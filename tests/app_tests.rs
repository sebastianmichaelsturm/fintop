use fintop::app::App;
use fintop::filter::{matches_index, matches_stock};
use fintop::model::{Region, SortColumn, SortDirection, TimeRange, TrendPoint};
use fintop::provider::dataset::get_default_indices;
use fintop::provider::mock::MockMarketEngine;
use tokio::sync::mpsc;

#[test]
fn test_indices_default_change_sorting() {
    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let app = App::new(tx_cmd, rx_evt, true);

    // Initial state: ChangePercent descending (+ to -)
    assert_eq!(app.sort_column_indices, SortColumn::ChangePercent);
    assert_eq!(app.sort_direction_indices, SortDirection::Descending);

    let sorted = app.sorted_indices();
    assert_eq!(sorted.len(), 27, "Must contain all 27 global benchmarks");

    for i in 0..sorted.len() - 1 {
        assert!(
            sorted[i].change_percent >= sorted[i + 1].change_percent,
            "Index {} ({:.2}%) has lower change % than next {} ({:.2}%)",
            sorted[i].symbol,
            sorted[i].change_percent,
            sorted[i + 1].symbol,
            sorted[i + 1].change_percent
        );
    }
}

#[test]
fn test_indices_region_sorting() {
    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let mut app = App::new(tx_cmd, rx_evt, true);

    app.sort_column_indices = SortColumn::DefaultRegion;
    app.sort_direction_indices = SortDirection::Ascending;

    let sorted = app.sorted_indices();
    assert_eq!(sorted.len(), 27, "Must contain all 27 global benchmarks");

    // Verify all 4 regions are present
    assert!(sorted.iter().any(|i| i.region == Region::UnitedStates));
    assert!(sorted.iter().any(|i| i.region == Region::Europe));
    assert!(sorted.iter().any(|i| i.region == Region::Asia));
    assert!(sorted.iter().any(|i| i.region == Region::RestOfWorld));

    let mut last_region = Region::UnitedStates;
    for idx in &sorted {
        assert!(idx.region >= last_region, "Index {:?} region out of order", idx.name);
        last_region = idx.region;
    }
}

#[test]
fn test_constituents_default_change_sorting() {
    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let mut app = App::new(tx_cmd, rx_evt, true);

    // Enter first index
    app.enter_index();

    assert_eq!(app.sort_column_stocks, SortColumn::ChangePercent);
    assert_eq!(app.sort_direction_stocks, SortDirection::Descending);

    let stocks = app.sorted_constituents();
    assert!(!stocks.is_empty());

    // Assert highest change % on top (descending order + to -)
    for i in 0..stocks.len() - 1 {
        assert!(
            stocks[i].change_percent >= stocks[i + 1].change_percent,
            "Stock {} ({:.2}%) has lower change % than next {} ({:.2}%)",
            stocks[i].symbol,
            stocks[i].change_percent,
            stocks[i + 1].symbol,
            stocks[i + 1].change_percent
        );
    }
}

#[test]
fn test_clean_symbol_no_caret() {
    let indices = get_default_indices();
    for idx in &indices {
        let clean = idx.clean_symbol();
        assert!(!clean.starts_with('^'), "Symbol '{}' should not start with '^'", clean);
        assert!(!clean.is_empty(), "Clean symbol for {} should not be empty", idx.name);
    }

    // Specific ticker checks
    let sp500 = indices.iter().find(|i| i.id == "sp500").unwrap();
    assert_eq!(sp500.clean_symbol(), "SPX");

    let dax = indices.iter().find(|i| i.id == "dax40").unwrap();
    assert_eq!(dax.clean_symbol(), "DAX");

    let dji = indices.iter().find(|i| i.id == "dowjones").unwrap();
    assert_eq!(dji.clean_symbol(), "DJI");

    let vix = indices.iter().find(|i| i.id == "vix").unwrap();
    assert_eq!(vix.clean_symbol(), "VIX");

    let smi = indices.iter().find(|i| i.id == "smi20").unwrap();
    assert_eq!(smi.clean_symbol(), "SMI");

    let mib = indices.iter().find(|i| i.id == "ftsemib").unwrap();
    assert_eq!(mib.clean_symbol(), "MIB");

    let ssec = indices.iter().find(|i| i.id == "shanghai").unwrap();
    assert_eq!(ssec.clean_symbol(), "SSEC");

    let nifty = indices.iter().find(|i| i.id == "nifty50").unwrap();
    assert_eq!(nifty.clean_symbol(), "NIFTY");

    let kospi = indices.iter().find(|i| i.id == "kospi").unwrap();
    assert_eq!(kospi.clean_symbol(), "KOSPI");

    let tsx = indices.iter().find(|i| i.id == "tsx").unwrap();
    assert_eq!(tsx.clean_symbol(), "TSX");

    let ibov = indices.iter().find(|i| i.id == "bovespa").unwrap();
    assert_eq!(ibov.clean_symbol(), "IBOV");
}

#[test]
fn test_nvidia_market_cap_and_dynamic_update() {
    let indices = get_default_indices();
    let sp500 = indices.iter().find(|i| i.id == "sp500").unwrap();
    let nvda = sp500.constituents.iter().find(|s| s.symbol == "NVDA").unwrap();

    assert!(nvda.market_cap >= 5_000_000_000_000, "Nvidia market cap should be >= $5.0T, got {}", nvda.market_cap);
    assert!(nvda.shares_outstanding > 20_000_000_000, "Nvidia shares outstanding should be > 20B");
}

#[test]
fn test_sorting_columns_and_directions() {
    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let mut app = App::new(tx_cmd, rx_evt, true);

    // Initial state: ChangePercent, Descending
    assert_eq!(app.sort_column_indices, SortColumn::ChangePercent);
    assert_eq!(app.sort_direction_indices, SortDirection::Descending);

    // Initial sort is descending by change percent (+ to -)
    let sorted = app.sorted_indices();
    for i in 0..sorted.len() - 1 {
        assert!(
            sorted[i].change_percent >= sorted[i + 1].change_percent,
            "Indices not sorted descending by change percent"
        );
    }

    // Toggle direction to Ascending (- to +)
    app.toggle_sort_direction();
    assert_eq!(app.sort_direction_indices, SortDirection::Ascending);

    let asc_sorted = app.sorted_indices();
    for i in 0..asc_sorted.len() - 1 {
        assert!(
            asc_sorted[i].change_percent <= asc_sorted[i + 1].change_percent,
            "Indices not sorted ascending by change percent"
        );
    }

    // Cycle sort column
    app.cycle_sort();
    assert_eq!(app.sort_column_indices, SortColumn::Price);
}

#[test]
fn test_filter_matching() {
    let indices = get_default_indices();
    let dax = indices.iter().find(|i| i.id == "dax40").unwrap();

    assert!(matches_index(dax, "dax"));
    assert!(matches_index(dax, "DAX 40"));
    assert!(matches_index(dax, "europe"));
    assert!(!matches_index(dax, "nonexistent"));

    let sap = dax.constituents.iter().find(|s| s.symbol == "SAP.DE").unwrap();
    assert!(matches_stock(sap, "sap"));
    assert!(matches_stock(sap, "SAP SE"));
    assert!(!matches_stock(sap, "tesla"));
}

#[test]
fn test_constituent_roster_counts() {
    let indices = get_default_indices();

    let dax = indices.iter().find(|i| i.id == "dax40").unwrap();
    assert_eq!(dax.constituents.len(), 40, "DAX 40 must have exactly 40 constituents");

    let dow = indices.iter().find(|i| i.id == "dowjones").unwrap();
    assert_eq!(dow.constituents.len(), 30, "Dow Jones must have exactly 30 constituents");

    let stoxx50 = indices.iter().find(|i| i.id == "stoxx50").unwrap();
    assert_eq!(stoxx50.constituents.len(), 50, "Euro Stoxx 50 must have exactly 50 constituents");

    let stoxx600 = indices.iter().find(|i| i.id == "stoxx600").unwrap();
    assert_eq!(stoxx600.constituents.len(), 50, "Euro Stoxx 600 must have constituents populated");
    assert_eq!(stoxx600.clean_symbol(), "SXXP");

    let nasdaq = indices.iter().find(|i| i.id == "nasdaq100").unwrap();
    assert_eq!(nasdaq.constituents.len(), 50, "Nasdaq sample constituents must be populated");

    let vix = indices.iter().find(|i| i.id == "vix").unwrap();
    assert_eq!(vix.constituents.len(), 6, "VIX proxies must be populated");

    let smi = indices.iter().find(|i| i.id == "smi20").unwrap();
    assert_eq!(smi.constituents.len(), 12, "SMI constituents must be populated");

    let ssec = indices.iter().find(|i| i.id == "shanghai").unwrap();
    assert_eq!(ssec.constituents.len(), 10, "Shanghai constituents must be populated");

    let kospi = indices.iter().find(|i| i.id == "kospi").unwrap();
    assert_eq!(kospi.constituents.len(), 10, "KOSPI constituents must be populated");

    let tsx = indices.iter().find(|i| i.id == "tsx").unwrap();
    assert_eq!(tsx.constituents.len(), 10, "TSX constituents must be populated");

    let bovespa = indices.iter().find(|i| i.id == "bovespa").unwrap();
    assert_eq!(bovespa.constituents.len(), 10, "Bovespa constituents must be populated");

    let ipc = indices.iter().find(|i| i.id == "ipcmexico").unwrap();
    assert_eq!(ipc.constituents.len(), 10, "IPC Mexico constituents must be populated");
}

#[test]
fn test_time_range_navigation_and_params() {
    assert_eq!(TimeRange::OneDay.next(), TimeRange::FiveDays);
    assert_eq!(TimeRange::FiveDays.next(), TimeRange::OneMonth);
    assert_eq!(TimeRange::OneMonth.next(), TimeRange::SixMonths);
    assert_eq!(TimeRange::SixMonths.next(), TimeRange::OneYear);
    assert_eq!(TimeRange::OneYear.next(), TimeRange::FiveYears);
    assert_eq!(TimeRange::FiveYears.next(), TimeRange::OneDay);

    assert_eq!(TimeRange::OneDay.prev(), TimeRange::FiveYears);
    assert_eq!(TimeRange::FiveYears.prev(), TimeRange::OneYear);

    assert_eq!(TimeRange::from_key_number('1'), Some(TimeRange::OneDay));
    assert_eq!(TimeRange::from_key_number('3'), Some(TimeRange::OneMonth));
    assert_eq!(TimeRange::from_key_number('6'), Some(TimeRange::FiveYears));
    assert_eq!(TimeRange::from_key_number('9'), None);

    assert_eq!(TimeRange::OneDay.yahoo_params(), ("1d", "5m"));
    assert_eq!(TimeRange::OneMonth.yahoo_params(), ("1mo", "1d"));
    assert_eq!(TimeRange::FiveYears.yahoo_params(), ("5y", "1wk"));
}

#[test]
fn test_mock_trend_generation() {
    let mut engine = MockMarketEngine::new();
    let current_price = 215.38;
    let points = engine.generate_trend(current_price, TimeRange::OneMonth, 0);

    assert!(!points.is_empty(), "Points should not be empty");
    assert_eq!(points.len(), 30, "OneMonth should generate 30 points");

    // Timestamps must be sorted ascending
    for i in 0..points.len() - 1 {
        assert!(points[i].timestamp < points[i + 1].timestamp, "Timestamps out of chronological order");
        assert!(points[i].price > 0.0, "Price must be strictly positive");
    }

    // Last point must match current_price
    let last = points.last().unwrap();
    assert_eq!(last.price, current_price);
}

#[test]
fn test_open_and_close_trend_modal() {
    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let mut app = App::new(tx_cmd, rx_evt, true);

    // Initial state: no modal
    assert!(app.trend_state.is_none());

    // Enter index view
    app.enter_index();
    let selected_stock = app.sorted_constituents()[0].clone();

    // Open trend window
    app.open_trend_window();
    assert!(app.trend_state.is_some());

    let trend = app.trend_state.as_ref().unwrap();
    assert_eq!(trend.symbol, selected_stock.symbol);
    assert_eq!(trend.current_price, selected_stock.current_price);
    assert_eq!(trend.time_range, TimeRange::OneMonth); // Default is 1M

    // Cycle timeframe
    app.cycle_trend_range();
    assert_eq!(app.trend_state.as_ref().unwrap().time_range, TimeRange::SixMonths);

    // Direct jump
    app.set_trend_range(TimeRange::OneDay);
    assert_eq!(app.trend_state.as_ref().unwrap().time_range, TimeRange::OneDay);

    // Close modal
    app.close_trend_window();
    assert!(app.trend_state.is_none());
}

#[test]
fn test_trend_state_period_metrics() {
    let points = vec![
        TrendPoint { timestamp: 1000, price: 100.0 },
        TrendPoint { timestamp: 2000, price: 120.0 },
        TrendPoint { timestamp: 3000, price: 90.0 },
        TrendPoint { timestamp: 4000, price: 110.0 },
    ];

    let state = fintop::app::TrendState {
        symbol: "TEST".to_string(),
        name: "Test Stock".to_string(),
        currency: "USD".to_string(),
        current_price: 110.0,
        change: 5.0,
        change_percent: 4.76,
        time_range: TimeRange::OneMonth,
        points,
        is_loading: false,
        error: None,
        window_offset_secs: 0,
    };

    assert_eq!(state.period_open(), Some(100.0));
    assert_eq!(state.period_close(), Some(110.0));
    assert_eq!(state.period_min(), Some(90.0));
    assert_eq!(state.period_max(), Some(120.0));

    let (diff, pct) = state.period_change();
    assert_eq!(diff, 10.0);
    assert_eq!(pct, 10.0); // (110 - 100) / 100 * 100 = 10%
}

#[test]
fn test_window_shift_navigation() {
    use fintop::worker::WorkerCommand;

    let (tx_cmd, mut rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let mut app = App::new(tx_cmd, rx_evt, true);

    app.enter_index();
    app.open_trend_window();

    // Drain prior worker commands
    while rx_cmd.try_recv().is_ok() {}

    let trend = app.trend_state.as_mut().unwrap();
    trend.time_range = TimeRange::OneDay; // 1D resolution: window_shift_secs is 3600 (1 hour)
    assert_eq!(trend.window_offset_secs, 0);

    // Shift window backward into past: should shift by -3600s
    app.shift_window_backward();
    assert_eq!(app.trend_state.as_ref().unwrap().window_offset_secs, -3600);

    if let Ok(cmd) = rx_cmd.try_recv() {
        match cmd {
            WorkerCommand::FetchTrend { offset_secs, range, .. } => {
                assert_eq!(offset_secs, -3600);
                assert_eq!(range, TimeRange::OneDay);
            }
            _ => panic!("Expected FetchTrend command with -3600 offset"),
        }
    }

    // Shift backward again: should be -7200s (2 hours)
    app.shift_window_backward();
    assert_eq!(app.trend_state.as_ref().unwrap().window_offset_secs, -7200);

    // Shift forward: should return to -3600s
    app.shift_window_forward();
    assert_eq!(app.trend_state.as_ref().unwrap().window_offset_secs, -3600);

    // Reset offset: resets to 0 (latest live window)
    app.reset_window_offset();
    assert_eq!(app.trend_state.as_ref().unwrap().window_offset_secs, 0);
}

#[test]
fn test_index_points_formatting() {
    use fintop::model::format_points;

    assert_eq!(format_points(5808.12), "5,808.12");
    assert_eq!(format_points(19450.30), "19,450.30");
    assert_eq!(format_points(42141.54), "42,141.54");
    assert_eq!(format_points(38981.75), "38,981.75");
    assert_eq!(format_points(0.0), "0.00");
    assert_eq!(format_points(825.40), "825.40");

    // Must not contain any currency symbols
    for val in [5808.12, 19450.30, 42141.54] {
        let formatted = format_points(val);
        assert!(!formatted.contains('$'));
        assert!(!formatted.contains('€'));
        assert!(!formatted.contains('£'));
        assert!(!formatted.contains('¥'));
    }

    // Sort column naming for indices
    assert_eq!(SortColumn::Price.as_str_for_indices(), "Points");
    assert_eq!(SortColumn::Price.as_str(), "Price");
}

#[test]
fn test_multi_currency_formatting() {
    use fintop::model::{format_market_cap, format_price};

    // JPY and KRW without decimal fractions
    assert_eq!(format_price(2680.0, "JPY"), "¥2,680");
    assert_eq!(format_price(59200.0, "KRW"), "₩59,200");
    assert_eq!(format_price(198000.0, "KRW"), "₩198,000");

    // Other currencies with 2 decimal places
    assert_eq!(format_price(84.50, "CHF"), "CHF 84.50");
    assert_eq!(format_price(168.50, "CAD"), "C$168.50");
    assert_eq!(format_price(144.50, "AUD"), "A$144.50");
    assert_eq!(format_price(1560.00, "CNY"), "¥1,560.00");
    assert_eq!(format_price(2715.00, "INR"), "₹2,715.00");
    assert_eq!(format_price(1060.00, "TWD"), "NT$1,060.00");
    assert_eq!(format_price(39.20, "SGD"), "S$39.20");
    assert_eq!(format_price(36.80, "BRL"), "R$36.80");
    assert_eq!(format_price(59.80, "MXN"), "Mex$59.80");

    // Market cap abbreviations
    assert_eq!(format_market_cap(353_000_000_000_000, "KRW"), "₩353.00T");
    assert_eq!(format_market_cap(27_500_000_000_000, "TWD"), "NT$27.50T");
    assert_eq!(format_market_cap(238_000_000_000, "CAD"), "C$238.0B");
}

#[test]
fn test_default_datasets_populated() {
    use fintop::provider::dataset::{get_default_currencies, get_default_futures, get_default_stocks};

    let stocks = get_default_stocks();
    assert_eq!(stocks.len(), 14, "Must contain 14 global benchmark stocks");
    assert!(stocks.iter().any(|s| s.symbol == "NVDA"));
    assert!(stocks.iter().any(|s| s.symbol == "AAPL"));
    assert!(stocks.iter().any(|s| s.symbol == "MSFT"));
    assert!(stocks.iter().any(|s| s.symbol == "ASML"));
    assert!(stocks.iter().any(|s| s.symbol == "SAP.DE"));

    let currencies = get_default_currencies();
    assert_eq!(currencies.len(), 11, "Must contain 11 currency/crypto pairs");
    assert!(currencies.iter().any(|c| c.symbol == "EURUSD=X"));
    assert!(currencies.iter().any(|c| c.symbol == "USDJPY=X"));
    assert!(currencies.iter().any(|c| c.symbol == "BTC-USD"));
    assert!(currencies.iter().any(|c| c.symbol == "ETH-USD"));

    let futures = get_default_futures();
    assert_eq!(futures.len(), 11, "Must contain 11 futures contracts");
    assert!(futures.iter().any(|f| f.symbol == "GC=F" && f.category == "Metals"));
    assert!(futures.iter().any(|f| f.symbol == "CL=F" && f.category == "Energy"));
    assert!(futures.iter().any(|f| f.symbol == "ES=F" && f.category == "Indices"));
    assert!(futures.iter().any(|f| f.symbol == "ZN=F" && f.category == "Bonds"));
}

#[test]
fn test_crypto_currencies_included_and_formatted() {
    use fintop::model::format_rate;
    use fintop::provider::dataset::get_default_currencies;

    let currencies = get_default_currencies();
    let btc = currencies.iter().find(|c| c.symbol == "BTC-USD").unwrap();
    assert_eq!(btc.clean_symbol(), "BTC/USD");
    assert_eq!(btc.base_currency, "BTC");
    assert_eq!(format_rate(68850.0, &btc.base_currency), "$68,850.00");

    let eth = currencies.iter().find(|c| c.symbol == "ETH-USD").unwrap();
    assert_eq!(eth.clean_symbol(), "ETH/USD");
    assert_eq!(eth.base_currency, "ETH");

    let sol = currencies.iter().find(|c| c.symbol == "SOL-USD").unwrap();
    assert_eq!(sol.clean_symbol(), "SOL/USD");
    assert_eq!(sol.base_currency, "SOL");
}

#[test]
fn test_dashboard_section_switching() {
    use fintop::model::DashboardSection;

    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let mut app = App::new(tx_cmd, rx_evt, true);

    // Initial state: Indices
    assert_eq!(app.active_section, DashboardSection::Indices);
    assert_eq!(app.active_section.title(), "[1] Global Indices");

    // Cycle forward: Indices -> Stocks -> Currencies -> Futures -> Indices
    app.cycle_section();
    assert_eq!(app.active_section, DashboardSection::Stocks);
    assert_eq!(app.active_section.title(), "[2] Top Stocks");

    app.cycle_section();
    assert_eq!(app.active_section, DashboardSection::Currencies);
    assert_eq!(app.active_section.title(), "[3] Currencies & Crypto");

    app.cycle_section();
    assert_eq!(app.active_section, DashboardSection::Futures);
    assert_eq!(app.active_section.title(), "[4] Futures & Commodities");

    app.cycle_section();
    assert_eq!(app.active_section, DashboardSection::Indices);

    // Cycle backward
    app.prev_section();
    assert_eq!(app.active_section, DashboardSection::Futures);

    app.prev_section();
    assert_eq!(app.active_section, DashboardSection::Currencies);

    // Direct jump
    app.set_section(DashboardSection::Stocks);
    assert_eq!(app.active_section, DashboardSection::Stocks);

    app.set_section(DashboardSection::Indices);
    assert_eq!(app.active_section, DashboardSection::Indices);
}

#[test]
fn test_dashboard_row_navigation() {
    use fintop::model::DashboardSection;

    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let mut app = App::new(tx_cmd, rx_evt, true);

    // Test row navigation on Indices
    app.set_section(DashboardSection::Indices);
    assert_eq!(app.indices_table_state.selected(), Some(0));
    app.next_row();
    assert_eq!(app.indices_table_state.selected(), Some(1));
    app.previous_row();
    assert_eq!(app.indices_table_state.selected(), Some(0));

    // Test row navigation on Stocks
    app.set_section(DashboardSection::Stocks);
    assert_eq!(app.stocks_table_state.selected(), Some(0));
    app.next_row();
    assert_eq!(app.stocks_table_state.selected(), Some(1));
    app.previous_row();
    assert_eq!(app.stocks_table_state.selected(), Some(0));

    // Test row navigation on Currencies
    app.set_section(DashboardSection::Currencies);
    assert_eq!(app.currencies_table_state.selected(), Some(0));
    app.next_row();
    assert_eq!(app.currencies_table_state.selected(), Some(1));
    app.previous_row();
    assert_eq!(app.currencies_table_state.selected(), Some(0));

    // Test row navigation on Futures
    app.set_section(DashboardSection::Futures);
    assert_eq!(app.futures_table_state.selected(), Some(0));
    app.next_row();
    assert_eq!(app.futures_table_state.selected(), Some(1));
    app.previous_row();
    assert_eq!(app.futures_table_state.selected(), Some(0));
}

#[test]
fn test_currencies_and_futures_sorting() {
    use fintop::model::{DashboardSection, SortColumn, SortDirection};

    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let mut app = App::new(tx_cmd, rx_evt, true);

    // Currencies sorting
    app.set_section(DashboardSection::Currencies);
    assert_eq!(app.sort_column_currencies, SortColumn::ChangePercent);
    assert_eq!(app.sort_direction_currencies, SortDirection::Descending);

    let currs = app.sorted_currencies();
    for i in 0..currs.len() - 1 {
        assert!(
            currs[i].change_percent >= currs[i + 1].change_percent,
            "Currencies not sorted descending by change percent"
        );
    }

    app.cycle_sort();
    assert_eq!(app.sort_column_currencies, SortColumn::Price);
    app.toggle_sort_direction();
    assert_eq!(app.sort_direction_currencies, SortDirection::Ascending);

    // Futures sorting
    app.set_section(DashboardSection::Futures);
    assert_eq!(app.sort_column_futures, SortColumn::ChangePercent);
    assert_eq!(app.sort_direction_futures, SortDirection::Descending);

    let futs = app.sorted_futures();
    for i in 0..futs.len() - 1 {
        assert!(
            futs[i].change_percent >= futs[i + 1].change_percent,
            "Futures not sorted descending by change percent"
        );
    }

    app.cycle_sort();
    assert_eq!(app.sort_column_futures, SortColumn::Price);
    app.toggle_sort_direction();
    assert_eq!(app.sort_direction_futures, SortDirection::Ascending);
}

#[test]
fn test_filter_currencies_and_futures() {
    use fintop::filter::{matches_currency, matches_futures};
    use fintop::provider::dataset::{get_default_currencies, get_default_futures};

    let currencies = get_default_currencies();
    let btc = currencies.iter().find(|c| c.symbol == "BTC-USD").unwrap();
    assert!(matches_currency(btc, "btc"));
    assert!(matches_currency(btc, "Bitcoin"));
    assert!(matches_currency(btc, "USD"));
    assert!(!matches_currency(btc, "Yen"));

    let eur = currencies.iter().find(|c| c.symbol == "EURUSD=X").unwrap();
    assert!(matches_currency(eur, "eur"));
    assert!(matches_currency(eur, "Euro"));

    let futures = get_default_futures();
    let gold = futures.iter().find(|f| f.symbol == "GC=F").unwrap();
    assert!(matches_futures(gold, "gold"));
    assert!(matches_futures(gold, "metals"));
    assert!(matches_futures(gold, "GC"));
    assert!(!matches_futures(gold, "Crude"));

    let oil = futures.iter().find(|f| f.symbol == "CL=F").unwrap();
    assert!(matches_futures(oil, "oil"));
    assert!(matches_futures(oil, "Energy"));
    assert!(matches_futures(oil, "WTI"));
}

#[test]
fn test_mock_engine_ticks_all_asset_classes() {
    use fintop::provider::dataset::{get_default_currencies, get_default_futures};

    let mut engine = MockMarketEngine::new();
    let mut currency = get_default_currencies()[0].clone();
    let prev_rate = currency.rate;
    assert!(prev_rate > 0.0);
    engine.tick_currency(&mut currency);
    assert!(currency.rate > 0.0);
    assert_ne!(currency.rate, 0.0);
    assert!(currency.day_high >= currency.rate);
    assert!(currency.day_low <= currency.rate);

    let mut future = get_default_futures()[0].clone();
    engine.tick_futures(&mut future);
    assert!(future.current_price > 0.0);
    assert!(future.day_high >= future.current_price);
    assert!(future.day_low <= future.current_price);
}

#[test]
fn test_open_trend_modal_for_different_assets() {
    use fintop::app::ViewMode;
    use fintop::model::DashboardSection;

    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let mut app = App::new(tx_cmd, rx_evt, true);

    // Initial state: Dashboard
    assert_eq!(app.view_mode, ViewMode::Dashboard);

    // 1. Open trend on Stocks
    app.set_section(DashboardSection::Stocks);
    app.open_trend_window();
    assert!(app.trend_state.is_some());
    let trend = app.trend_state.as_ref().unwrap();
    let top_stock = app.sorted_top_stocks()[0].symbol.clone();
    assert_eq!(trend.symbol, top_stock);
    app.close_trend_window();
    assert!(app.trend_state.is_none());

    // 2. Open trend on Currencies
    app.set_section(DashboardSection::Currencies);
    app.open_trend_window();
    assert!(app.trend_state.is_some());
    app.close_trend_window();

    // 3. Open trend on Futures
    app.set_section(DashboardSection::Futures);
    app.open_trend_window();
    assert!(app.trend_state.is_some());
    app.close_trend_window();
}

#[test]
fn test_toggle_maximize() {
    use fintop::app::ViewMode;
    use fintop::model::DashboardSection;

    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let mut app = App::new(tx_cmd, rx_evt, true);

    assert_eq!(app.view_mode, ViewMode::Dashboard);

    // Maximize Indices
    app.set_section(DashboardSection::Indices);
    app.toggle_maximize();
    assert_eq!(app.view_mode, ViewMode::Maximized(DashboardSection::Indices));

    // Toggle back
    app.toggle_maximize();
    assert_eq!(app.view_mode, ViewMode::Dashboard);

    // Maximize Futures
    app.set_section(DashboardSection::Futures);
    app.toggle_maximize();
    assert_eq!(app.view_mode, ViewMode::Maximized(DashboardSection::Futures));

    // Toggle back
    app.toggle_maximize();
    assert_eq!(app.view_mode, ViewMode::Dashboard);
}
