#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Region {
    UnitedStates = 0,
    Europe = 1,
    Asia = 2,
    RestOfWorld = 3,
}

impl Region {
    pub fn as_str(&self) -> &'static str {
        match self {
            Region::UnitedStates => "US",
            Region::Europe => "Europe",
            Region::Asia => "Asia",
            Region::RestOfWorld => "Global",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstituentStock {
    pub symbol: String,
    pub name: String,
    pub currency: String,
    pub current_price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub day_high: f64,
    pub day_low: f64,
    pub volume: u64,
    pub market_cap: u64,
}

#[derive(Debug, Clone)]
pub struct MarketIndex {
    pub id: String,
    pub symbol: String,          // API ticker symbol (e.g. "^GSPC")
    pub display_symbol: String,  // Clean UI display symbol (e.g. "SPX" / "DAX")
    pub name: String,
    pub region: Region,
    pub currency: String,
    pub current_price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub day_high: f64,
    pub day_low: f64,
    pub previous_close: f64,
    pub constituents: Vec<ConstituentStock>,
}

impl MarketIndex {
    pub fn clean_symbol(&self) -> &str {
        if !self.display_symbol.is_empty() {
            &self.display_symbol
        } else {
            self.symbol.strip_prefix('^').unwrap_or(&self.symbol)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    DefaultRegion,
    MarketCap,
    Price,
    ChangePercent,
    ChangeAbsolute,
    Name,
    Symbol,
    Volume,
}

impl SortColumn {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortColumn::DefaultRegion => "Region Hierarchy",
            SortColumn::MarketCap => "Market Cap",
            SortColumn::Price => "Price",
            SortColumn::ChangePercent => "Change %",
            SortColumn::ChangeAbsolute => "Change Abs",
            SortColumn::Name => "Name",
            SortColumn::Symbol => "Symbol",
            SortColumn::Volume => "Volume",
        }
    }

    pub fn next_for_indices(&self) -> Self {
        match self {
            SortColumn::DefaultRegion => SortColumn::ChangePercent,
            SortColumn::ChangePercent => SortColumn::Price,
            SortColumn::Price => SortColumn::Name,
            SortColumn::Name => SortColumn::Symbol,
            SortColumn::Symbol => SortColumn::DefaultRegion,
            _ => SortColumn::DefaultRegion,
        }
    }

    pub fn next_for_stocks(&self) -> Self {
        match self {
            SortColumn::MarketCap => SortColumn::Price,
            SortColumn::Price => SortColumn::ChangePercent,
            SortColumn::ChangePercent => SortColumn::Name,
            SortColumn::Name => SortColumn::Symbol,
            SortColumn::Symbol => SortColumn::Volume,
            SortColumn::Volume => SortColumn::MarketCap,
            _ => SortColumn::MarketCap,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            SortDirection::Ascending => "▲",
            SortDirection::Descending => "▼",
        }
    }
}

pub fn currency_symbol(curr: &str) -> &'static str {
    match curr.to_uppercase().as_str() {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        "CHF" => "CHF ",
        "HKD" => "HK$",
        _ => "",
    }
}

pub fn format_price(price: f64, currency: &str) -> String {
    let sym = currency_symbol(currency);
    if currency.eq_ignore_ascii_case("JPY") {
        let rounded = price.round() as i64;
        let int_str = rounded.to_string();
        let mut result = String::new();
        let len = int_str.len();
        for (i, ch) in int_str.chars().enumerate() {
            if i > 0 && (len - i).is_multiple_of(3) {
                result.push(',');
            }
            result.push(ch);
        }
        format!("{}{}", sym, result)
    } else {
        let formatted = format_with_commas(price);
        format!("{}{}", sym, formatted)
    }
}

fn format_with_commas(val: f64) -> String {
    let integer_part = val.trunc() as i64;
    let frac_part = (val.fract() * 100.0).round().abs() as u32;
    let int_str = integer_part.to_string();
    let is_negative = int_str.starts_with('-');
    let digits = if is_negative { &int_str[1..] } else { &int_str };

    let mut result = String::new();
    let len = digits.len();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(ch);
    }
    if is_negative {
        format!("-{}.{:02}", result, frac_part)
    } else {
        format!("{}.{:02}", result, frac_part)
    }
}

pub fn format_market_cap(market_cap: u64, currency: &str) -> String {
    let sym = currency_symbol(currency);
    if market_cap >= 1_000_000_000_000 {
        format!("{}{:.2}T", sym, market_cap as f64 / 1_000_000_000_000.0)
    } else if market_cap >= 1_000_000_000 {
        format!("{}{:.1}B", sym, market_cap as f64 / 1_000_000_000.0)
    } else if market_cap >= 1_000_000 {
        format!("{}{:.0}M", sym, market_cap as f64 / 1_000_000.0)
    } else if market_cap > 0 {
        format!("{}{}", sym, market_cap)
    } else {
        "-".to_string()
    }
}

pub fn format_volume(volume: u64) -> String {
    if volume >= 1_000_000_000 {
        format!("{:.2}B", volume as f64 / 1_000_000_000.0)
    } else if volume >= 1_000_000 {
        format!("{:.2}M", volume as f64 / 1_000_000.0)
    } else if volume >= 1_000 {
        format!("{:.1}K", volume as f64 / 1_000.0)
    } else if volume > 0 {
        format!("{}", volume)
    } else {
        "-".to_string()
    }
}
