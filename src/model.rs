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
    pub shares_outstanding: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    OneDay,
    FiveDays,
    OneMonth,
    SixMonths,
    OneYear,
    FiveYears,
}

impl TimeRange {
    pub const ALL: [TimeRange; 6] = [
        TimeRange::OneDay,
        TimeRange::FiveDays,
        TimeRange::OneMonth,
        TimeRange::SixMonths,
        TimeRange::OneYear,
        TimeRange::FiveYears,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            TimeRange::OneDay => "1D",
            TimeRange::FiveDays => "5D",
            TimeRange::OneMonth => "1M",
            TimeRange::SixMonths => "6M",
            TimeRange::OneYear => "1Y",
            TimeRange::FiveYears => "5Y",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TimeRange::OneDay => "1 Day (5m)",
            TimeRange::FiveDays => "5 Days (15m)",
            TimeRange::OneMonth => "1 Month (1d)",
            TimeRange::SixMonths => "6 Months (1d)",
            TimeRange::OneYear => "1 Year (1d)",
            TimeRange::FiveYears => "5 Years (1wk)",
        }
    }

    pub fn yahoo_params(&self) -> (&'static str, &'static str) {
        match self {
            TimeRange::OneDay => ("1d", "5m"),
            TimeRange::FiveDays => ("5d", "15m"),
            TimeRange::OneMonth => ("1mo", "1d"),
            TimeRange::SixMonths => ("6mo", "1d"),
            TimeRange::OneYear => ("1y", "1d"),
            TimeRange::FiveYears => ("5y", "1wk"),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            TimeRange::OneDay => TimeRange::FiveDays,
            TimeRange::FiveDays => TimeRange::OneMonth,
            TimeRange::OneMonth => TimeRange::SixMonths,
            TimeRange::SixMonths => TimeRange::OneYear,
            TimeRange::OneYear => TimeRange::FiveYears,
            TimeRange::FiveYears => TimeRange::OneDay,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            TimeRange::OneDay => TimeRange::FiveYears,
            TimeRange::FiveDays => TimeRange::OneDay,
            TimeRange::OneMonth => TimeRange::FiveDays,
            TimeRange::SixMonths => TimeRange::OneMonth,
            TimeRange::OneYear => TimeRange::SixMonths,
            TimeRange::FiveYears => TimeRange::OneYear,
        }
    }

    pub fn from_key_number(num: char) -> Option<Self> {
        match num {
            '1' => Some(TimeRange::OneDay),
            '2' => Some(TimeRange::FiveDays),
            '3' => Some(TimeRange::OneMonth),
            '4' => Some(TimeRange::SixMonths),
            '5' => Some(TimeRange::OneYear),
            '6' => Some(TimeRange::FiveYears),
            _ => None,
        }
    }

    pub fn step_label(&self) -> &'static str {
        match self {
            TimeRange::OneDay => "1h",
            TimeRange::FiveDays => "4h",
            TimeRange::OneMonth => "1d",
            TimeRange::SixMonths => "1w",
            TimeRange::OneYear => "1mo",
            TimeRange::FiveYears => "3mo",
        }
    }

    pub fn approx_duration_secs(&self) -> i64 {
        match self {
            TimeRange::OneDay => 86_400,
            TimeRange::FiveDays => 5 * 86_400,
            TimeRange::OneMonth => 30 * 86_400,
            TimeRange::SixMonths => 182 * 86_400,
            TimeRange::OneYear => 365 * 86_400,
            TimeRange::FiveYears => 5 * 365 * 86_400,
        }
    }

    pub fn window_shift_secs(&self) -> i64 {
        match self {
            TimeRange::OneDay => 3_600,
            TimeRange::FiveDays => 14_400,
            TimeRange::OneMonth => 86_400,
            TimeRange::SixMonths => 7 * 86_400,
            TimeRange::OneYear => 30 * 86_400,
            TimeRange::FiveYears => 91 * 86_400,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrendPoint {
    pub timestamp: i64,
    pub price: f64,
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

    pub fn as_str_for_indices(&self) -> &'static str {
        match self {
            SortColumn::Price => "Points",
            _ => self.as_str(),
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

pub fn format_points(points: f64) -> String {
    format_with_commas(points)
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
