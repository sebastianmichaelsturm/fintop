use serde::Deserialize;
use std::time::Duration;
use tokio::task::JoinSet;

#[derive(Deserialize, Debug)]
struct ChartResponse {
    chart: ChartBody,
}

#[derive(Deserialize, Debug)]
struct ChartBody {
    result: Option<Vec<ChartResult>>,
}

#[derive(Deserialize, Debug)]
struct ChartResult {
    meta: ChartMeta,
}

#[derive(Deserialize, Debug)]
struct ChartMeta {
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: Option<f64>,
    #[serde(rename = "chartPreviousClose")]
    chart_previous_close: Option<f64>,
    #[serde(rename = "previousClose")]
    previous_close: Option<f64>,
    #[serde(rename = "regularMarketDayHigh")]
    regular_market_day_high: Option<f64>,
    #[serde(rename = "regularMarketDayLow")]
    regular_market_day_low: Option<f64>,
    #[serde(rename = "regularMarketVolume")]
    regular_market_volume: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct QuoteUpdate {
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub day_high: f64,
    pub day_low: f64,
    pub volume: u64,
}

pub fn create_http_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(5))
        .build()
}

pub async fn fetch_single_quote(client: &reqwest::Client, symbol: &str) -> Result<QuoteUpdate, String> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1d",
        symbol
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request error for {}: {}", symbol, e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP error {}: {}", resp.status(), symbol));
    }

    let parsed: ChartResponse = resp
        .json()
        .await
        .map_err(|e| format!("JSON decode error for {}: {}", symbol, e))?;

    let result = parsed
        .chart
        .result
        .and_then(|mut r| if !r.is_empty() { Some(r.remove(0)) } else { None })
        .ok_or_else(|| format!("Empty chart result for {}", symbol))?;

    let meta = result.meta;
    let price = meta.regular_market_price.unwrap_or(0.0);
    let prev_close = meta.chart_previous_close.or(meta.previous_close).unwrap_or(price);

    let change = if prev_close > 0.0 {
        price - prev_close
    } else {
        0.0
    };
    let change_percent = if prev_close > 0.0 {
        (change / prev_close) * 100.0
    } else {
        0.0
    };

    Ok(QuoteUpdate {
        symbol: symbol.to_string(),
        price,
        change,
        change_percent,
        day_high: meta.regular_market_day_high.unwrap_or(price),
        day_low: meta.regular_market_day_low.unwrap_or(price),
        volume: meta.regular_market_volume.unwrap_or(0),
    })
}

pub async fn fetch_quotes_batch(
    client: &reqwest::Client,
    symbols: &[String],
    max_concurrency: usize,
) -> Vec<QuoteUpdate> {
    let mut results = Vec::new();
    let mut join_set = JoinSet::new();
    let mut iter = symbols.iter().cloned();

    // Spawn initial pool
    for _ in 0..max_concurrency {
        if let Some(sym) = iter.next() {
            let cl = client.clone();
            join_set.spawn(async move { fetch_single_quote(&cl, &sym).await });
        }
    }

    // Process completions and refill
    while let Some(res) = join_set.join_next().await {
        if let Ok(Ok(quote)) = res {
            results.push(quote);
        }
        if let Some(next_sym) = iter.next() {
            let cl = client.clone();
            join_set.spawn(async move { fetch_single_quote(&cl, &next_sym).await });
        }
    }

    results
}
