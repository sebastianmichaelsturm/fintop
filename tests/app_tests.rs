use fintop::app::App;
use fintop::filter::{matches_index, matches_stock};
use fintop::model::{Region, SortColumn, SortDirection};
use fintop::provider::dataset::get_default_indices;
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
fn test_constituents_default_highest_price_sorting() {
    let (tx_cmd, _rx_cmd) = mpsc::channel(32);
    let (_tx_evt, rx_evt) = mpsc::channel(64);
    let mut app = App::new(tx_cmd, rx_evt, true);

    // Enter first index
    app.enter_index();

    let stocks = app.sorted_constituents();
    assert!(!stocks.is_empty());

    // Assert highest price on top (descending order)
    for i in 0..stocks.len() - 1 {
        assert!(
            stocks[i].current_price >= stocks[i + 1].current_price,
            "Stock {} ({}) is lower price than next {} ({})",
            stocks[i].symbol,
            stocks[i].current_price,
            stocks[i + 1].symbol,
            stocks[i + 1].current_price
        );
    }
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

    let nasdaq = indices.iter().find(|i| i.id == "nasdaq100").unwrap();
    assert_eq!(nasdaq.constituents.len(), 50, "Nasdaq sample constituents must be populated");
}
