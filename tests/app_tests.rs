use fintop::app::App;
use fintop::filter::{matches_index, matches_stock};
use fintop::model::{Region, SortColumn, SortDirection, TimeRange, TrendPoint};
use fintop::provider::dataset::get_default_indices;
use fintop::provider::mock::MockMarketEngine;
use tokio::sync::mpsc;

#[test]
fn test_indices_default_region_sorting() {
    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let app = App::new(tx_cmd, rx_evt, true);

    let sorted = app.sorted_indices();
    assert!(!sorted.is_empty());

    // Verify region hierarchy: US first, then Europe, then Asia
    let first_index = &sorted[0];
    assert_eq!(first_index.region, Region::UnitedStates);

    let mut last_region = Region::UnitedStates;
    for idx in &sorted {
        assert!(idx.region >= last_region, "Index {:?} region out of order", idx.name);
        last_region = idx.region;
    }
}

#[test]
fn test_constituents_default_market_cap_sorting() {
    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let mut app = App::new(tx_cmd, rx_evt, true);

    // Enter first index
    app.enter_index();

    let stocks = app.sorted_constituents();
    assert!(!stocks.is_empty());

    // Assert highest market cap on top (descending order)
    for i in 0..stocks.len() - 1 {
        assert!(
            stocks[i].market_cap >= stocks[i + 1].market_cap,
            "Stock {} ({}) has lower market cap than next {} ({})",
            stocks[i].symbol,
            stocks[i].market_cap,
            stocks[i + 1].symbol,
            stocks[i + 1].market_cap
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

    // Initial state: DefaultRegion, Ascending
    assert_eq!(app.sort_column_indices, SortColumn::DefaultRegion);
    assert_eq!(app.sort_direction_indices, SortDirection::Ascending);

    // Cycle sort
    app.cycle_sort();
    assert_eq!(app.sort_column_indices, SortColumn::ChangePercent);

    // Toggle direction
    app.toggle_sort_direction();
    assert_eq!(app.sort_direction_indices, SortDirection::Descending);

    let sorted = app.sorted_indices();
    for i in 0..sorted.len() - 1 {
        assert!(
            sorted[i].change_percent >= sorted[i + 1].change_percent,
            "Indices not sorted descending by change percent"
        );
    }
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
