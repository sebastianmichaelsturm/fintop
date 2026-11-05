use crate::app::{App, DashboardSection};
use crate::ui::currencies_table::render_currencies_table;
use crate::ui::futures_table::render_futures_table;
use crate::ui::indices_table::render_indices_table;
use crate::ui::stocks_table::render_stocks_table;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub fn render_dashboard(frame: &mut Frame, app: &mut App, area: Rect) {
    // Split into left and right columns (50% / 50%)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Split left column into Top (Indices) and Bottom (Currencies)
    let left_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(cols[0]);

    // Split right column into Top (Stocks) and Bottom (Futures)
    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(cols[1]);

    let focused = app.active_section;

    // Top-Left: Indices
    render_indices_table(
        frame,
        app,
        left_rows[0],
        focused == DashboardSection::Indices,
        false,
    );

    // Top-Right: Top Stocks
    render_stocks_table(
        frame,
        app,
        right_rows[0],
        focused == DashboardSection::Stocks,
        false,
    );

    // Bottom-Left: Currencies
    render_currencies_table(
        frame,
        app,
        left_rows[1],
        focused == DashboardSection::Currencies,
        false,
    );

    // Bottom-Right: Futures
    render_futures_table(
        frame,
        app,
        right_rows[1],
        focused == DashboardSection::Futures,
        false,
    );
}

pub fn render_maximized(
    frame: &mut Frame,
    app: &mut App,
    section: DashboardSection,
    area: Rect,
) {
    match section {
        DashboardSection::Indices => {
            render_indices_table(frame, app, area, true, true);
        }
        DashboardSection::Stocks => {
            render_stocks_table(frame, app, area, true, true);
        }
        DashboardSection::Currencies => {
            render_currencies_table(frame, app, area, true, true);
        }
        DashboardSection::Futures => {
            render_futures_table(frame, app, area, true, true);
        }
    }
}
