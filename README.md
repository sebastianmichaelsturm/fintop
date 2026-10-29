# `fintop` 📈⚡

A high-performance, real-time terminal financial market tracker (inspired by `top`, `htop`, and peer tools [`ftop`](https://github.com/sebastianmichaelsturm/ftop)) that monitors major global indices and their constituent stocks in a modern Rust TUI.

Built with **Rust**, [`ratatui`](https://ratatui.rs/), [`crossterm`](https://crates.io/crates/crossterm), and [`tokio`](https://tokio.rs/).

---

## Features

- **Global Market Indices Overview**:
  - Displays all major indices grouped by regional hierarchy: **US Indices** (S&P 500, Nasdaq 100, Dow Jones, Russell 2000) $\to$ **European Indices** (DAX 40, FTSE 100, CAC 40, Euro Stoxx 50) $\to$ **Asian Indices** (Nikkei 225, Hang Seng).
  - Shows index level/price, daily absolute change, percentage change, and daily high/low range.
  - Sortable by Region Priority, Level/Price, Change %, Absolute Change, Name, or Symbol.
- **Constituent Stocks Explorer (Detail View)**:
  - Press `[Enter]` on any index to inspect its full constituent member stocks.
  - Comprehensive rosters included for all major indices (DAX 40, Dow Jones 30, Euro Stoxx 50, CAC 40, FTSE 100, Nasdaq 100, S&P 500, Nikkei 225).
  - **Largest Companies on Top by Default** (`Market Cap ▼`), sortable by Market Cap, Price, Change %, Absolute Change, Name, Symbol, or Volume.
- **High-Efficiency Lazy Loading & On-Demand Data Fetching**:
  - In overview mode, only queries the ~10 major index symbols in the background every 10 seconds.
  - Constituent quotes are queried on-demand only when an index is opened, saving network bandwidth and preventing API throttling.
  - In-memory 60-second TTL cache for instant re-entry.
  - Graceful fallback: built-in offline simulation mode (`--demo`) ensures fluid operation anywhere.
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
| `Enter` | Open selected index to explore constituent stocks |
| `Esc` / `Backspace` / `q` | Return from constituent view to indices overview |
| `q` | Quit `fintop` (from indices overview) |
| `s` | Cycle sort column |
| `d` | Toggle sort direction (Ascending ▲ / Descending ▼) |
| `/` | Live substring filter/search (press `Enter` to lock, `Esc` to clear) |
| `r` | Force immediate refresh now |
| `Space` | Pause / resume background ticks |
| `↑` / `k` | Move cursor up |
| `↓` / `j` | Move cursor down |

---

## License

MIT OR Apache-2.0
