use crate::model::{TimeRange, TrendPoint};
use crate::provider::mock::MockMarketEngine;
use crate::provider::yahoo::{create_http_client, fetch_historical_trend, fetch_quotes_batch, QuoteUpdate};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::interval;

#[derive(Debug)]
pub enum WorkerCommand {
    PollIndices(Vec<String>),
    PollConstituents {
        index_id: String,
        symbols: Vec<String>,
    },
    CancelConstituents,
    FetchTrend {
        symbol: String,
        range: TimeRange,
        current_price: f64,
        offset_secs: i64,
    },
    ForceRefresh,
}

#[derive(Debug)]
pub enum WorkerEvent {
    IndicesUpdated(Vec<QuoteUpdate>),
    TopStocksUpdated(Vec<QuoteUpdate>),
    CurrenciesUpdated(Vec<QuoteUpdate>),
    FuturesUpdated(Vec<QuoteUpdate>),
    StocksUpdated {
        index_id: String,
        quotes: Vec<QuoteUpdate>,
    },
    TrendDataReceived {
        symbol: String,
        range: TimeRange,
        offset_secs: i64,
        points: Vec<TrendPoint>,
    },
    FetchStatus {
        is_live: bool,
        status_message: String,
    },
}

pub struct MarketWorker {
    rx_cmd: Receiver<WorkerCommand>,
    tx_evt: Sender<WorkerEvent>,
    demo_mode: bool,
    interval_secs: u64,
    index_symbols: Vec<String>,
    stock_symbols: Vec<String>,
    currency_symbols: Vec<String>,
    futures_symbols: Vec<String>,
    active_constituents: Option<(String, Vec<String>)>,
    mock_engine: MockMarketEngine,
}

impl MarketWorker {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rx_cmd: Receiver<WorkerCommand>,
        tx_evt: Sender<WorkerEvent>,
        demo_mode: bool,
        interval_secs: u64,
        index_symbols: Vec<String>,
        stock_symbols: Vec<String>,
        currency_symbols: Vec<String>,
        futures_symbols: Vec<String>,
    ) -> Self {
        Self {
            rx_cmd,
            tx_evt,
            demo_mode,
            interval_secs: interval_secs.max(1),
            index_symbols,
            stock_symbols,
            currency_symbols,
            futures_symbols,
            active_constituents: None,
            mock_engine: MockMarketEngine::new(),
        }
    }

    pub async fn run(mut self) {
        let http_client = if !self.demo_mode {
            create_http_client().ok()
        } else {
            None
        };

        let mut timer = interval(Duration::from_secs(self.interval_secs));
        // First tick completes immediately
        timer.tick().await;

        // Perform initial fetch on start
        self.do_fetch(http_client.as_ref()).await;

        loop {
            tokio::select! {
                Some(cmd) = self.rx_cmd.recv() => {
                    match cmd {
                        WorkerCommand::PollIndices(syms) => {
                            self.index_symbols = syms;
                        }
                        WorkerCommand::PollConstituents { index_id, symbols } => {
                            self.active_constituents = Some((index_id.clone(), symbols.clone()));
                            // Immediately fetch constituents on-demand
                            self.fetch_active_constituents(&index_id, &symbols, http_client.as_ref()).await;
                        }
                        WorkerCommand::CancelConstituents => {
                            self.active_constituents = None;
                        }
                        WorkerCommand::FetchTrend { symbol, range, current_price, offset_secs } => {
                            self.fetch_trend(&symbol, range, current_price, offset_secs, http_client.as_ref()).await;
                        }
                        WorkerCommand::ForceRefresh => {
                            self.do_fetch(http_client.as_ref()).await;
                        }
                    }
                }
                _ = timer.tick() => {
                    self.do_fetch(http_client.as_ref()).await;
                }
            }
        }
    }

    async fn do_fetch(&mut self, http_client: Option<&reqwest::Client>) {
        if let Some((ref index_id, ref symbols)) = self.active_constituents.clone() {
            // In constituent view: fetch active constituent stocks on-demand
            self.fetch_active_constituents(index_id, symbols, http_client).await;
        } else {
            // In dashboard view: fetch symbols across all 4 asset classes
            self.fetch_dashboard(http_client).await;
        }
    }

    async fn fetch_dashboard(&mut self, http_client: Option<&reqwest::Client>) {
        let mut all_symbols = Vec::new();
        all_symbols.extend(self.index_symbols.clone());
        all_symbols.extend(self.stock_symbols.clone());
        all_symbols.extend(self.currency_symbols.clone());
        all_symbols.extend(self.futures_symbols.clone());

        if let Some(client) = http_client {
            let quotes = fetch_quotes_batch(client, &all_symbols, 12).await;
            if !quotes.is_empty() {
                let mut index_quotes = Vec::new();
                let mut stock_quotes = Vec::new();
                let mut currency_quotes = Vec::new();
                let mut futures_quotes = Vec::new();

                for q in quotes {
                    if self.index_symbols.contains(&q.symbol) {
                        index_quotes.push(q);
                    } else if self.stock_symbols.contains(&q.symbol) {
                        stock_quotes.push(q);
                    } else if self.currency_symbols.contains(&q.symbol) {
                        currency_quotes.push(q);
                    } else if self.futures_symbols.contains(&q.symbol) {
                        futures_quotes.push(q);
                    }
                }

                if !index_quotes.is_empty() {
                    let _ = self.tx_evt.send(WorkerEvent::IndicesUpdated(index_quotes)).await;
                }
                if !stock_quotes.is_empty() {
                    let _ = self.tx_evt.send(WorkerEvent::TopStocksUpdated(stock_quotes)).await;
                }
                if !currency_quotes.is_empty() {
                    let _ = self.tx_evt.send(WorkerEvent::CurrenciesUpdated(currency_quotes)).await;
                }
                if !futures_quotes.is_empty() {
                    let _ = self.tx_evt.send(WorkerEvent::FuturesUpdated(futures_quotes)).await;
                }

                let _ = self.tx_evt.send(WorkerEvent::FetchStatus {
                    is_live: true,
                    status_message: "Live (Yahoo Finance)".to_string(),
                }).await;
                return;
            }
        }

        // Fallback or demo simulation
        let index_symbols = self.index_symbols.clone();
        let mut sim_indices = Vec::new();
        for sym in &index_symbols {
            sim_indices.push(self.create_simulated_quote(sym));
        }
        let _ = self.tx_evt.send(WorkerEvent::IndicesUpdated(sim_indices)).await;

        let stock_symbols = self.stock_symbols.clone();
        let mut sim_stocks = Vec::new();
        for sym in &stock_symbols {
            sim_stocks.push(self.create_simulated_quote(sym));
        }
        let _ = self.tx_evt.send(WorkerEvent::TopStocksUpdated(sim_stocks)).await;

        let currency_symbols = self.currency_symbols.clone();
        let mut sim_currencies = Vec::new();
        for sym in &currency_symbols {
            sim_currencies.push(self.create_simulated_quote(sym));
        }
        let _ = self.tx_evt.send(WorkerEvent::CurrenciesUpdated(sim_currencies)).await;

        let futures_symbols = self.futures_symbols.clone();
        let mut sim_futures = Vec::new();
        for sym in &futures_symbols {
            sim_futures.push(self.create_simulated_quote(sym));
        }
        let _ = self.tx_evt.send(WorkerEvent::FuturesUpdated(sim_futures)).await;

        let _ = self.tx_evt.send(WorkerEvent::FetchStatus {
            is_live: false,
            status_message: if self.demo_mode { "Demo Mode".to_string() } else { "Offline (Simulated)".to_string() },
        }).await;
    }

    async fn fetch_active_constituents(
        &mut self,
        index_id: &str,
        symbols: &[String],
        http_client: Option<&reqwest::Client>,
    ) {
        if let Some(client) = http_client {
            let quotes = fetch_quotes_batch(client, symbols, 12).await;
            if !quotes.is_empty() {
                let _ = self.tx_evt.send(WorkerEvent::StocksUpdated {
                    index_id: index_id.to_string(),
                    quotes,
                }).await;
                let _ = self.tx_evt.send(WorkerEvent::FetchStatus {
                    is_live: true,
                    status_message: "Live (Yahoo Finance)".to_string(),
                }).await;
                return;
            }
        }

        // Fallback or demo simulation for constituent stocks
        let mut simulated = Vec::new();
        for sym in symbols {
            simulated.push(self.create_simulated_quote(sym));
        }
        let _ = self.tx_evt.send(WorkerEvent::StocksUpdated {
            index_id: index_id.to_string(),
            quotes: simulated,
        }).await;
        let _ = self.tx_evt.send(WorkerEvent::FetchStatus {
            is_live: false,
            status_message: if self.demo_mode { "Demo Mode".to_string() } else { "Offline (Simulated)".to_string() },
        }).await;
    }

    fn create_simulated_quote(&mut self, symbol: &str) -> QuoteUpdate {
        let mut dummy = crate::model::ConstituentStock {
            symbol: symbol.to_string(),
            name: String::new(),
            currency: "USD".to_string(),
            current_price: 150.0,
            change: 0.0,
            change_percent: 0.0,
            day_high: 155.0,
            day_low: 145.0,
            volume: 1_000_000,
            market_cap: 100_000_000_000,
            shares_outstanding: 1_000_000_000,
        };
        self.mock_engine.tick_stock(&mut dummy);
        QuoteUpdate {
            symbol: symbol.to_string(),
            price: dummy.current_price,
            change: dummy.change,
            change_percent: dummy.change_percent,
            day_high: dummy.day_high,
            day_low: dummy.day_low,
            volume: dummy.volume,
        }
    }

    async fn fetch_trend(
        &mut self,
        symbol: &str,
        range: TimeRange,
        current_price: f64,
        offset_secs: i64,
        http_client: Option<&reqwest::Client>,
    ) {
        if let Some(client) = http_client {
            if let Ok(points) = fetch_historical_trend(client, symbol, range, offset_secs).await {
                let _ = self.tx_evt.send(WorkerEvent::TrendDataReceived {
                    symbol: symbol.to_string(),
                    range,
                    offset_secs,
                    points,
                }).await;
                return;
            }
        }

        // Fallback or demo simulation
        let points = self.mock_engine.generate_trend(current_price, range, offset_secs);
        let _ = self.tx_evt.send(WorkerEvent::TrendDataReceived {
            symbol: symbol.to_string(),
            range,
            offset_secs,
            points,
        }).await;
    }
}
