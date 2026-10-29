use crate::filter::{matches_index, matches_stock};
use crate::model::{
    ConstituentStock, MarketIndex, SortColumn, SortDirection, TimeRange, TrendPoint,
};
use crate::provider::dataset::get_default_indices;
use crate::provider::yahoo::QuoteUpdate;
use crate::worker::{WorkerCommand, WorkerEvent};
use chrono::{DateTime, Local};
use ratatui::widgets::TableState;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{Receiver, Sender};

const CACHE_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewMode {
    Indices,
    Constituents { index_id: String },
}

pub struct CachedConstituents {
    pub updated_at: Instant,
    pub stocks: Vec<ConstituentStock>,
}

#[derive(Debug, Clone)]
pub struct TrendState {
    pub symbol: String,
    pub name: String,
    pub currency: String,
    pub current_price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub time_range: TimeRange,
    pub points: Vec<TrendPoint>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub window_offset_secs: i64,
}

impl TrendState {
    pub fn period_open(&self) -> Option<f64> {
        self.points.first().map(|p| p.price)
    }

    pub fn period_close(&self) -> Option<f64> {
        self.points.last().map(|p| p.price).or(Some(self.current_price))
    }

    pub fn period_min(&self) -> Option<f64> {
        self.points
            .iter()
            .map(|p| p.price)
            .reduce(f64::min)
            .or(Some(self.current_price))
    }

    pub fn period_max(&self) -> Option<f64> {
        self.points
            .iter()
            .map(|p| p.price)
            .reduce(f64::max)
            .or(Some(self.current_price))
    }

    pub fn period_change(&self) -> (f64, f64) {
        if let (Some(open), Some(close)) = (self.period_open(), self.period_close()) {
            if open > 0.0 {
                let diff = close - open;
                let pct = (diff / open) * 100.0;
                return (diff, pct);
            }
        }
        (self.change, self.change_percent)
    }
}

pub struct App {
    pub view_mode: ViewMode,
    pub indices: Vec<MarketIndex>,
    pub indices_table_state: TableState,
    pub stocks_table_state: TableState,
    pub sort_column_indices: SortColumn,
    pub sort_direction_indices: SortDirection,
    pub sort_column_stocks: SortColumn,
    pub sort_direction_stocks: SortDirection,
    pub filter_query: String,
    pub is_filtering: bool,
    pub is_paused: bool,
    pub status_message: String,
    pub is_live: bool,
    pub start_time: Instant,
    pub last_updated: Option<DateTime<Local>>,
    pub tx_worker: Sender<WorkerCommand>,
    pub rx_worker: Receiver<WorkerEvent>,
    pub stock_cache: HashMap<String, CachedConstituents>,
    pub trend_state: Option<TrendState>,
}

impl App {
    pub fn new(
        tx_worker: Sender<WorkerCommand>,
        rx_worker: Receiver<WorkerEvent>,
        demo_mode: bool,
    ) -> Self {
        let indices = get_default_indices();
        let mut indices_table_state = TableState::default();
        indices_table_state.select(Some(0));

        let mut stocks_table_state = TableState::default();
        stocks_table_state.select(Some(0));

        Self {
            view_mode: ViewMode::Indices,
            indices,
            indices_table_state,
            stocks_table_state,
            sort_column_indices: SortColumn::DefaultRegion,
            sort_direction_indices: SortDirection::Ascending,
            sort_column_stocks: SortColumn::MarketCap,
            sort_direction_stocks: SortDirection::Descending, // Highest market cap on top by default
            filter_query: String::new(),
            is_filtering: false,
            is_paused: false,
            status_message: if demo_mode {
                "Demo Mode".to_string()
            } else {
                "Connecting...".to_string()
            },
            is_live: !demo_mode,
            start_time: Instant::now(),
            last_updated: None,
            tx_worker,
            rx_worker,
            stock_cache: HashMap::new(),
            trend_state: None,
        }
    }

    pub fn process_worker_events(&mut self) {
        while let Ok(evt) = self.rx_worker.try_recv() {
            if self.is_paused {
                continue;
            }
            match evt {
                WorkerEvent::IndicesUpdated(quotes) => {
                    self.apply_indices_quotes(quotes);
                }
                WorkerEvent::StocksUpdated { index_id, quotes } => {
                    self.apply_stocks_quotes(&index_id, quotes);
                }
                WorkerEvent::TrendDataReceived {
                    symbol,
                    range,
                    offset_secs,
                    points,
                } => {
                    if let Some(ref mut trend) = self.trend_state {
                        if trend.symbol == symbol && trend.time_range == range && trend.window_offset_secs == offset_secs {
                            trend.points = points;
                            trend.is_loading = false;
                        }
                    }
                }
                WorkerEvent::FetchStatus {
                    is_live,
                    status_message,
                } => {
                    self.is_live = is_live;
                    self.status_message = status_message;
                }
            }
        }
    }

    fn apply_indices_quotes(&mut self, quotes: Vec<QuoteUpdate>) {
        for quote in quotes {
            if let Some(idx) = self.indices.iter_mut().find(|i| i.symbol == quote.symbol) {
                idx.current_price = quote.price;
                idx.change = quote.change;
                idx.change_percent = quote.change_percent;
                idx.day_high = quote.day_high;
                idx.day_low = quote.day_low;
            }
        }
        self.last_updated = Some(Local::now());
    }

    fn apply_stocks_quotes(&mut self, index_id: &str, quotes: Vec<QuoteUpdate>) {
        if let Some(idx) = self.indices.iter_mut().find(|i| i.id == index_id) {
            for quote in quotes {
                if let Some(stock) = idx.constituents.iter_mut().find(|s| s.symbol == quote.symbol) {
                    stock.current_price = quote.price;
                    stock.change = quote.change;
                    stock.change_percent = quote.change_percent;
                    stock.day_high = quote.day_high;
                    stock.day_low = quote.day_low;
                    if quote.volume > 0 {
                        stock.volume = quote.volume;
                    }
                    if stock.shares_outstanding > 0 {
                        stock.market_cap = (stock.current_price * stock.shares_outstanding as f64).round() as u64;
                    }
                }
            }
            // Update cache
            self.stock_cache.insert(
                index_id.to_string(),
                CachedConstituents {
                    updated_at: Instant::now(),
                    stocks: idx.constituents.clone(),
                },
            );
        }
        self.last_updated = Some(Local::now());
    }

    pub fn enter_index(&mut self) {
        if self.view_mode != ViewMode::Indices {
            return;
        }

        let sorted = self.sorted_indices();
        let selected_pos = self.indices_table_state.selected().unwrap_or(0);
        if let Some(selected_index) = sorted.get(selected_pos) {
            let index_id = selected_index.id.clone();
            let symbols: Vec<String> = selected_index
                .constituents
                .iter()
                .map(|s| s.symbol.clone())
                .collect();

            self.view_mode = ViewMode::Constituents {
                index_id: index_id.clone(),
            };

            // Reset stocks table cursor to 0 (top / highest market cap)
            self.stocks_table_state.select(Some(0));
            self.sort_column_stocks = SortColumn::MarketCap;
            self.sort_direction_stocks = SortDirection::Descending;
            self.filter_query.clear();
            self.is_filtering = false;

            // Check in-memory cache
            let needs_fetch = match self.stock_cache.get(&index_id) {
                Some(cached) => cached.updated_at.elapsed() > CACHE_TTL,
                None => true,
            };

            if needs_fetch && !self.is_paused {
                let _ = self.tx_worker.try_send(WorkerCommand::PollConstituents {
                    index_id,
                    symbols,
                });
            }
        }
    }

    pub fn exit_index(&mut self) {
        if self.view_mode != ViewMode::Indices {
            self.view_mode = ViewMode::Indices;
            self.filter_query.clear();
            self.is_filtering = false;
            let _ = self.tx_worker.try_send(WorkerCommand::CancelConstituents);
        }
    }

    pub fn trigger_refresh(&mut self) {
        let _ = self.tx_worker.try_send(WorkerCommand::ForceRefresh);
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    pub fn cycle_sort(&mut self) {
        match self.view_mode {
            ViewMode::Indices => {
                self.sort_column_indices = self.sort_column_indices.next_for_indices();
            }
            ViewMode::Constituents { .. } => {
                self.sort_column_stocks = self.sort_column_stocks.next_for_stocks();
            }
        }
    }

    pub fn toggle_sort_direction(&mut self) {
        match self.view_mode {
            ViewMode::Indices => {
                self.sort_direction_indices = self.sort_direction_indices.toggle();
            }
            ViewMode::Constituents { .. } => {
                self.sort_direction_stocks = self.sort_direction_stocks.toggle();
            }
        }
    }

    pub fn next_row(&mut self) {
        match self.view_mode {
            ViewMode::Indices => {
                let count = self.sorted_indices().len();
                if count == 0 {
                    return;
                }
                let current = self.indices_table_state.selected().unwrap_or(0);
                let next = (current + 1).min(count - 1);
                self.indices_table_state.select(Some(next));
            }
            ViewMode::Constituents { .. } => {
                let count = self.sorted_constituents().len();
                if count == 0 {
                    return;
                }
                let current = self.stocks_table_state.selected().unwrap_or(0);
                let next = (current + 1).min(count - 1);
                self.stocks_table_state.select(Some(next));
            }
        }
    }

    pub fn previous_row(&mut self) {
        match self.view_mode {
            ViewMode::Indices => {
                let current = self.indices_table_state.selected().unwrap_or(0);
                let prev = current.saturating_sub(1);
                self.indices_table_state.select(Some(prev));
            }
            ViewMode::Constituents { .. } => {
                let current = self.stocks_table_state.selected().unwrap_or(0);
                let prev = current.saturating_sub(1);
                self.stocks_table_state.select(Some(prev));
            }
        }
    }

    pub fn sorted_indices(&self) -> Vec<MarketIndex> {
        let mut list: Vec<MarketIndex> = self
            .indices
            .iter()
            .filter(|i| matches_index(i, &self.filter_query))
            .cloned()
            .collect();

        list.sort_by(|a, b| {
            let ordering = match self.sort_column_indices {
                SortColumn::DefaultRegion => a.region.cmp(&b.region).then_with(|| a.name.cmp(&b.name)),
                SortColumn::Price => a.current_price.total_cmp(&b.current_price),
                SortColumn::ChangePercent => a.change_percent.total_cmp(&b.change_percent),
                SortColumn::ChangeAbsolute => a.change.total_cmp(&b.change),
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Symbol => a.symbol.cmp(&b.symbol),
                _ => a.region.cmp(&b.region),
            };

            if self.sort_direction_indices == SortDirection::Descending {
                ordering.reverse()
            } else {
                ordering
            }
        });

        list
    }

    pub fn active_index(&self) -> Option<&MarketIndex> {
        if let ViewMode::Constituents { ref index_id } = self.view_mode {
            self.indices.iter().find(|i| i.id == *index_id)
        } else {
            None
        }
    }

    pub fn sorted_constituents(&self) -> Vec<ConstituentStock> {
        if let Some(index) = self.active_index() {
            let mut list: Vec<ConstituentStock> = index
                .constituents
                .iter()
                .filter(|s| matches_stock(s, &self.filter_query))
                .cloned()
                .collect();

            list.sort_by(|a, b| {
                let ordering = match self.sort_column_stocks {
                    SortColumn::MarketCap => a.market_cap.cmp(&b.market_cap),
                    SortColumn::Price => a.current_price.total_cmp(&b.current_price),
                    SortColumn::ChangePercent => a.change_percent.total_cmp(&b.change_percent),
                    SortColumn::ChangeAbsolute => a.change.total_cmp(&b.change),
                    SortColumn::Name => a.name.cmp(&b.name),
                    SortColumn::Symbol => a.symbol.cmp(&b.symbol),
                    SortColumn::Volume => a.volume.cmp(&b.volume),
                    _ => a.market_cap.cmp(&b.market_cap),
                };

                if self.sort_direction_stocks == SortDirection::Descending {
                    ordering.reverse()
                } else {
                    ordering
                }
            });

            list
        } else {
            Vec::new()
        }
    }

    pub fn open_trend_window(&mut self) {
        if let ViewMode::Constituents { .. } = self.view_mode {
            let sorted = self.sorted_constituents();
            let selected_pos = self.stocks_table_state.selected().unwrap_or(0);
            if let Some(stock) = sorted.get(selected_pos) {
                let symbol = stock.symbol.clone();
                let current_price = stock.current_price;
                let default_range = TimeRange::OneMonth;

                self.trend_state = Some(TrendState {
                    symbol: symbol.clone(),
                    name: stock.name.clone(),
                    currency: stock.currency.clone(),
                    current_price,
                    change: stock.change,
                    change_percent: stock.change_percent,
                    time_range: default_range,
                    points: Vec::new(),
                    is_loading: true,
                    error: None,
                    window_offset_secs: 0,
                });

                let _ = self.tx_worker.try_send(WorkerCommand::FetchTrend {
                    symbol,
                    range: default_range,
                    current_price,
                    offset_secs: 0,
                });
            }
        }
    }

    pub fn close_trend_window(&mut self) {
        self.trend_state = None;
    }

    pub fn set_trend_range(&mut self, range: TimeRange) {
        if let Some(ref mut trend) = self.trend_state {
            if trend.time_range != range {
                trend.time_range = range;
                trend.window_offset_secs = 0;
                trend.is_loading = true;
                let symbol = trend.symbol.clone();
                let current_price = trend.current_price;
                let _ = self.tx_worker.try_send(WorkerCommand::FetchTrend {
                    symbol,
                    range,
                    current_price,
                    offset_secs: 0,
                });
            }
        }
    }

    pub fn cycle_trend_range(&mut self) {
        if let Some(ref trend) = self.trend_state {
            let next_range = trend.time_range.next();
            self.set_trend_range(next_range);
        }
    }

    pub fn prev_trend_range(&mut self) {
        if let Some(ref trend) = self.trend_state {
            let prev_range = trend.time_range.prev();
            self.set_trend_range(prev_range);
        }
    }

    pub fn refresh_trend(&mut self) {
        if let Some(ref mut trend) = self.trend_state {
            trend.is_loading = true;
            let symbol = trend.symbol.clone();
            let range = trend.time_range;
            let current_price = trend.current_price;
            let offset_secs = trend.window_offset_secs;
            let _ = self.tx_worker.try_send(WorkerCommand::FetchTrend {
                symbol,
                range,
                current_price,
                offset_secs,
            });
        }
    }

    pub fn shift_window_backward(&mut self) {
        if let Some(ref mut trend) = self.trend_state {
            let shift = trend.time_range.window_shift_secs();
            trend.window_offset_secs -= shift;
            trend.is_loading = true;
            let symbol = trend.symbol.clone();
            let range = trend.time_range;
            let current_price = trend.current_price;
            let offset_secs = trend.window_offset_secs;
            let _ = self.tx_worker.try_send(WorkerCommand::FetchTrend {
                symbol,
                range,
                current_price,
                offset_secs,
            });
        }
    }

    pub fn shift_window_forward(&mut self) {
        if let Some(ref mut trend) = self.trend_state {
            if trend.window_offset_secs < 0 {
                let shift = trend.time_range.window_shift_secs();
                trend.window_offset_secs = (trend.window_offset_secs + shift).min(0);
                trend.is_loading = true;
                let symbol = trend.symbol.clone();
                let range = trend.time_range;
                let current_price = trend.current_price;
                let offset_secs = trend.window_offset_secs;
                let _ = self.tx_worker.try_send(WorkerCommand::FetchTrend {
                    symbol,
                    range,
                    current_price,
                    offset_secs,
                });
            }
        }
    }

    pub fn reset_window_offset(&mut self) {
        if let Some(ref mut trend) = self.trend_state {
            if trend.window_offset_secs != 0 {
                trend.window_offset_secs = 0;
                trend.is_loading = true;
                let symbol = trend.symbol.clone();
                let range = trend.time_range;
                let current_price = trend.current_price;
                let _ = self.tx_worker.try_send(WorkerCommand::FetchTrend {
                    symbol,
                    range,
                    current_price,
                    offset_secs: 0,
                });
            }
        }
    }
}
