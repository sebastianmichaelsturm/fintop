use crate::model::{ConstituentStock, CurrencyQuote, FuturesQuote, MarketIndex};

pub fn matches_index(index: &MarketIndex, query: &str) -> bool {
    if query.trim().is_empty() {
        return true;
    }
    let q = query.to_lowercase();
    index.name.to_lowercase().contains(&q)
        || index.symbol.to_lowercase().contains(&q)
        || index.clean_symbol().to_lowercase().contains(&q)
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

pub fn matches_currency(currency: &CurrencyQuote, query: &str) -> bool {
    if query.trim().is_empty() {
        return true;
    }
    let q = query.to_lowercase();
    currency.name.to_lowercase().contains(&q)
        || currency.symbol.to_lowercase().contains(&q)
        || currency.display_symbol.to_lowercase().contains(&q)
        || currency.base_currency.to_lowercase().contains(&q)
        || currency.quote_currency.to_lowercase().contains(&q)
}

pub fn matches_futures(future: &FuturesQuote, query: &str) -> bool {
    if query.trim().is_empty() {
        return true;
    }
    let q = query.to_lowercase();
    future.name.to_lowercase().contains(&q)
        || future.symbol.to_lowercase().contains(&q)
        || future.display_symbol.to_lowercase().contains(&q)
        || future.category.to_lowercase().contains(&q)
        || future.unit.to_lowercase().contains(&q)
}
