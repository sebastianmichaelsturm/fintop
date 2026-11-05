pub mod constituents_table;
pub mod currencies_table;
pub mod dashboard;
pub mod footer;
pub mod futures_table;
pub mod header;
pub mod indices_table;
pub mod stocks_table;
pub mod trend_modal;

use crate::app::{App, ViewMode};
use constituents_table::render_constituents_table;
use dashboard::{render_dashboard, render_maximized};
use footer::render_footer;
use header::render_header;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use trend_modal::render_trend_modal;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header / Metrics
            Constraint::Min(5),    // Table / Dashboard
            Constraint::Length(2), // Help / Status bar
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);

    match app.view_mode {
        ViewMode::Dashboard => render_dashboard(frame, app, chunks[1]),
        ViewMode::Maximized(sec) => render_maximized(frame, app, sec, chunks[1]),
        ViewMode::Constituents { .. } => render_constituents_table(frame, app, chunks[1]),
    }

    render_footer(frame, app, chunks[2]);

    if let Some(ref trend) = app.trend_state {
        render_trend_modal(frame, trend, frame.area());
    }
}
