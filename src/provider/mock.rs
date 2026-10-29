use crate::model::{ConstituentStock, MarketIndex};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MockMarketEngine {
    seed: u64,
}

impl MockMarketEngine {
    pub fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(123456789);
        Self { seed }
    }

    fn next_rand(&mut self) -> f64 {
        // Simple linear congruential generator for deterministic, dependency-free random floats
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = (self.seed >> 32) as u32;
        (val as f64) / (u32::MAX as f64)
    }

    pub fn tick_index(&mut self, index: &mut MarketIndex) {
        // Random drift between -0.3% and +0.3%
        let factor = (self.next_rand() - 0.49) * 0.003;
        let new_price = (index.current_price * (1.0 + factor)).max(0.01);
        index.current_price = (new_price * 100.0).round() / 100.0;

        let base = if index.previous_close > 0.0 { index.previous_close } else { index.current_price };
        index.change = (index.current_price - base * 100.0).round() / 100.0;
        index.change_percent = ((index.change / base) * 100.0 * 100.0).round() / 100.0;

        if index.current_price > index.day_high {
            index.day_high = index.current_price;
        }
        if index.current_price < index.day_low || index.day_low == 0.0 {
            index.day_low = index.current_price;
        }
    }

    pub fn tick_stock(&mut self, stock: &mut ConstituentStock) {
        // Random drift between -0.5% and +0.5%
        let factor = (self.next_rand() - 0.49) * 0.005;
        let new_price = (stock.current_price * (1.0 + factor)).max(0.01);
        stock.current_price = (new_price * 100.0).round() / 100.0;

        let prev = stock.current_price - stock.change;
        let base = if prev > 0.0 { prev } else { stock.current_price };
        stock.change = (stock.current_price - base * 100.0).round() / 100.0;
        stock.change_percent = ((stock.change / base) * 100.0 * 100.0).round() / 100.0;

        if stock.current_price > stock.day_high {
            stock.day_high = stock.current_price;
        }
        if stock.current_price < stock.day_low || stock.day_low == 0.0 {
            stock.day_low = stock.current_price;
        }
        // Small volume increment
        stock.volume = stock.volume.saturating_add((self.next_rand() * 5000.0) as u64);
        if stock.shares_outstanding > 0 {
            stock.market_cap = (stock.current_price * stock.shares_outstanding as f64).round() as u64;
        }
    }
}

impl Default for MockMarketEngine {
    fn default() -> Self {
        Self::new()
    }
}
