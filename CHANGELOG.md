# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-11-05

### Added
- **Multi-Asset Starting Page (2x2 Dashboard Grid)**:
  - **Top Stocks**: Global mega-caps (NVDA, AAPL, MSFT, GOOGL, AMZN, META, TSLA, AVGO, BRK-B, LLY, ASML, SAP, TSM, NOVO-B) with dynamic market cap calculation.
  - **Market Indices**: 26 global benchmark indices covering US, Europe, Asia, and Rest of World (S&P 500, Nasdaq 100, Dow Jones, Russell 2000, DAX 40, FTSE 100, Euro Stoxx 50/600, CAC 40, Nikkei 225, Hang Seng, etc.).
  - **Currencies & Crypto**: Major Forex currency pairs (EUR/USD, USD/JPY, GBP/USD, USD/CHF, AUD/USD, USD/CAD, EUR/GBP, EUR/CHF) and benchmark cryptocurrencies (BTC-USD, ETH-USD, SOL-USD) with day high/low ranges.
  - **Futures**: Commodities and financial futures spanning Metals (Gold GC=F, Silver SI=F, Copper HG=F), Energy (Crude Oil CL=F, Brent BZ=F, Natural Gas NG=F), Index Futures (E-mini S&P ES=F, E-mini Nasdaq NQ=F), and Bonds (10-Year T-Note ZN=F, 30-Year T-Bond ZB=F, German Bund FGBL.DE).
- **Quadrant Navigation & Maximizing**:
  - Direct quadrant focus with hotkeys `1` (Stocks), `2` (Indices), `3` (Currencies), `4` (Futures).
  - Quick panel cycling via `Tab` / `BackTab`.
  - Maximize active section into full-screen view with `m` or `f` (toggle back with `m`/`f`/`Esc`).
  - Index constituent drill-down via `Enter` from the Indices section.
- **Default Change-Percent Sorting**:
  - All four dashboard panels (Stocks, Indices, Currencies, Futures) and index constituent views sort by `Change %` descending (`+` to `-`) by default.
  - Multi-column sorting (`s`) and direction toggling (`S`) remain available.
- **Interactive Asset Trend Inspection**:
  - Modal trend chart available across all asset classes with selectable timeframes (`1D`, `5D`, `1M`, `6M`, `YTD`, `1Y`, `5Y`, `MAX`) and step-by-step window panning (`h`/`l`).

### Changed
- Standardized codebase formatting with `cargo fmt`.
- Market indices display values cleanly in index points rather than currency symbols.
- Dynamic market cap computation based on live prices and actual shares outstanding.

---

## [0.1.0] - 2026-10-20

### Added
- Initial release of `fintop` modern terminal user interface (TUI) for financial markets built with Ratatui and Tokio.
- Real-time market data worker with Yahoo Finance API integration and fallback mock tick engine.
- Interactive filtering with `/`, live ticker updates, and customizable refresh intervals.
