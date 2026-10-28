pub mod constituents_table;
pub mod footer;
pub mod header;
pub mod indices_table;

use crate::app::{App, ViewMode};
use constituents_table::render_constituents_table;
use footer::render_footer;
use header::render_header;
use indices_table::render_indices_table;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header / Metrics
            Constraint::Min(5),    // Table (Indices or Constituent Stocks)
            Constraint::Length(2), // Help / Status bar
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);

    match app.view_mode {
        ViewMode::Indices => render_indices_table(frame, app, chunks[1]),
        ViewMode::Constituents { .. } => render_constituents_table(frame, app, chunks[1]),
    }

    render_footer(frame, app, chunks[2]);
}
