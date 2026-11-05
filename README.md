# `fintop` 📈

A high-performance, real-time terminal financial market tracker (inspired by `top`, `htop`, and peer tools [`ftop`](https://github.com/sebastianmichaelsturm/ftop)) that monitors **Stocks**, **Indices**, **Currencies**, and **Futures** in a modern Rust TUI.

Built with **Rust**, [`ratatui`](https://ratatui.rs/), [`crossterm`](https://crates.io/crates/crossterm), and [`tokio`](https://tokio.rs/).

---

## Features

- **Multi-Asset Starting Page (4-Quadrant Grid Dashboard)**:
  - **[1] Global Market Indices**: Major benchmarks across the US, Europe, Asia-Pacific, and Americas (S&P 500, Nasdaq 100, Dow Jones, DAX 40, FTSE 100, Nikkei 225, Hang Seng, etc.).
  - **[2] Top Benchmark Stocks**: Mega-cap market leaders (NVIDIA, Apple, Microsoft, Alphabet, Amazon, Meta, Tesla, ASML, SAP, TSMC, etc.).
  - **[3] Currencies & Crypto**: G10 Forex pairs (EUR/USD, USD/JPY, GBP/USD, USD/CHF, AUD/USD, USD/CAD, etc.) and leading digital assets (Bitcoin `BTC-USD`, Ethereum `ETH-USD`, Solana `SOL-USD`).
  - **[4] Futures & Commodities**: Key contracts across Energy (WTI, Brent, Nat Gas), Metals (Gold, Silver, Copper), Index Futures (E-mini S&P, Nasdaq, Dow), and Bonds (10Y Note, 30Y Bond).
- **Interactive Panel Focus & Maximization**:
  - Switch active panel focus with `Tab` / `BackTab` or numbers `[1]` to `[4]`.
  - Press `m` to maximize any focused panel into full-screen view; press `Esc` or `m` to return to the 4-panel dashboard.
- **Constituent Stocks Explorer & Interactive Trend Charts**:
  - Press `[Enter]` on any index to inspect its full constituent member stocks.
  - Press `[Enter]` on any stock, currency pair, or future contract to open the **Interactive Multi-Timeframe Trend Chart** (1D, 5D, 1M, 6M, 1Y, 5Y with panning).
- **High-Efficiency Background Worker**:
  - Live real-time market data via concurrent Yahoo Finance queries.
  - Built-in offline simulation mode (`--demo`) with deterministic price and volatility engine.
  - 60-second TTL caching for index constituent rosters.
- **Top-Like Controls**:
  - Interactive sorting (`s` to cycle column, `d` to toggle direction).
  - Instant live filter/search (`/`).
  - Pause / resume updates (`Space`).
  - Immediate manual refresh (`r`).
  - Clean panic-safe terminal restoration.

---

## Installation & Build

```bash
# Clone the repository
git clone https://github.com/sebastianmichaelsturm/fintop.git
cd fintop

# Build release binary
cargo build --release

# Run fintop
./target/release/fintop
```

---

## Usage

```bash
# Start in live market mode (default 10s refresh interval)
fintop

# Start in offline simulation/demo mode
fintop --demo

# Set custom refresh interval in seconds
fintop --interval 5
```

---

## Keyboard Controls

| Key | Description |
|---|---|
| `Tab` / `BackTab` | Cycle active panel focus forward / backward |
| `1` / `2` / `3` / `4` | Directly jump to Indices (`1`), Stocks (`2`), Currencies (`3`), Futures (`4`) |
| `m` / `f` | Toggle Maximize / Full-Screen mode for the active panel |
| `Enter` | Open selected index constituents OR open interactive Trend Chart |
| `Esc` / `Backspace` | Return to 4-panel dashboard from Constituent or Maximized view |
| `q` | Return to dashboard (from subviews) OR Quit `fintop` (from dashboard) |
| `s` | Cycle sort column for active panel |
| `d` | Toggle sort direction (Ascending ▲ / Descending ▼) |
| `/` | Live substring filter/search (press `Enter` to lock, `Esc` to clear) |
| `r` | Force immediate refresh now |
| `Space` | Pause / resume background ticks |
| `↑` / `k` | Move cursor up |
| `↓` / `j` | Move cursor down |

---

## License

MIT
