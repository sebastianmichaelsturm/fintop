use crate::model::{ConstituentStock, MarketIndex};

pub fn matches_index(index: &MarketIndex, query: &str) -> bool {
    if query.trim().is_empty() {
        return true;
    }
    let q = query.to_lowercase();
    index.name.to_lowercase().contains(&q)
        || index.symbol.to_lowercase().contains(&q)
        || index.region.as_str().to_lowercase().contains(&q)
        || index.currency.to_lowercase().contains(&q)
}

pub fn matches_stock(stock: &ConstituentStock, query: &str) -> bool {
    if query.trim().is_empty() {
        return true;
    }
    let q = query.to_lowercase();
    stock.name.to_lowercase().contains(&q)
        || stock.symbol.to_lowercase().contains(&q)
        || stock.currency.to_lowercase().contains(&q)
}
