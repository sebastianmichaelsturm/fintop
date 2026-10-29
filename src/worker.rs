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
    active_constituents: Option<(String, Vec<String>)>,
    mock_engine: MockMarketEngine,
}

impl MarketWorker {
    pub fn new(
        rx_cmd: Receiver<WorkerCommand>,
        tx_evt: Sender<WorkerEvent>,
        demo_mode: bool,
        interval_secs: u64,
        initial_index_symbols: Vec<String>,
    ) -> Self {
        Self {
            rx_cmd,
            tx_evt,
            demo_mode,
            interval_secs: interval_secs.max(1),
            index_symbols: initial_index_symbols,
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
            // In indices view: fetch only index symbols (high efficiency)
            self.fetch_indices(http_client).await;
        }
    }

    async fn fetch_indices(&mut self, http_client: Option<&reqwest::Client>) {
        if let Some(client) = http_client {
            let quotes = fetch_quotes_batch(client, &self.index_symbols, 8).await;
            if !quotes.is_empty() {
                let _ = self.tx_evt.send(WorkerEvent::IndicesUpdated(quotes)).await;
                let _ = self.tx_evt.send(WorkerEvent::FetchStatus {
                    is_live: true,
                    status_message: "Live (Yahoo Finance)".to_string(),
                }).await;
                return;
            }
        }

        // Fallback or demo simulation
        let symbols = self.index_symbols.clone();
        let mut simulated = Vec::new();
        for sym in &symbols {
            simulated.push(self.create_simulated_quote(sym));
        }
        let _ = self.tx_evt.send(WorkerEvent::IndicesUpdated(simulated)).await;
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
