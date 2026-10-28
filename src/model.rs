use ratatui::style::Color;

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
}

#[derive(Debug, Clone)]
pub struct MarketIndex {
    pub id: String,
    pub symbol: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    DefaultRegion,
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
            SortColumn::Price => SortColumn::ChangePercent,
            SortColumn::ChangePercent => SortColumn::Name,
            SortColumn::Name => SortColumn::Symbol,
            SortColumn::Symbol => SortColumn::Volume,
            SortColumn::Volume => SortColumn::Price,
            _ => SortColumn::Price,
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

pub fn format_change_values(change: f64, pct: f64) -> (String, Color) {
    if change > 0.0001 {
        (format!("+{:.2} (+{:.2}%)", change, pct), Color::Green)
    } else if change < -0.0001 {
        (format!("{:.2} ({:.2}%)", change, pct), Color::Red)
    } else {
        ("0.00 (0.00%)".to_string(), Color::DarkGray)
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
