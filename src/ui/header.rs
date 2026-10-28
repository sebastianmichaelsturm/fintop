use crate::app::{App, ViewMode};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let elapsed = app.start_time.elapsed().as_secs();
    let uptime_str = format!("{:02}:{:02}", elapsed / 60, elapsed % 60);

    let mut title_spans = vec![
        Span::styled(
            "fintop",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];

    if app.is_paused {
        title_spans.push(Span::styled(
            "[PAUSED]",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    } else if app.is_live {
        title_spans.push(Span::styled(
            "[LIVE]",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ));
    } else {
        title_spans.push(Span::styled(
            "[DEMO]",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
    }

    match app.view_mode {
        ViewMode::Indices => {
            title_spans.push(Span::raw(" — "));
            title_spans.push(Span::styled(
                "Global Market Indices",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            title_spans.push(Span::raw(" (US → Europe → Asia → Global)"));
        }
        ViewMode::Constituents { .. } => {
            if let Some(idx) = app.active_index() {
                title_spans.push(Span::raw(" — "));
                title_spans.push(Span::styled(
                    format!("Index: {} ({})", idx.name, idx.symbol),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ));
                title_spans.push(Span::raw(format!(
                    " — {} Constituents [Currency: {}]",
                    idx.constituents.len(),
                    idx.currency
                )));
            }
        }
    }

    let active_sort_str = match app.view_mode {
        ViewMode::Indices => format!(
            "{} {}",
            app.sort_column_indices.as_str(),
            app.sort_direction_indices.indicator()
        ),
        ViewMode::Constituents { .. } => format!(
            "{} {}",
            app.sort_column_stocks.as_str(),
            app.sort_direction_stocks.indicator()
        ),
    };

    let total_constituents: usize = app.indices.iter().map(|i| i.constituents.len()).sum();

    let mut metrics_spans = vec![
        Span::raw("Indices: "),
        Span::styled(format!("{}", app.indices.len()), Style::default().fg(Color::Green)),
        Span::raw("  |  Tracked Stocks: "),
        Span::styled(format!("{}", total_constituents), Style::default().fg(Color::Yellow)),
        Span::raw("  |  Sort: "),
        Span::styled(active_sort_str, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
    ];

    if !app.filter_query.is_empty() {
        metrics_spans.push(Span::raw("  |  Filter: "));
        metrics_spans.push(Span::styled(
            format!("\"{}\"", app.filter_query),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
    }

    metrics_spans.push(Span::raw("  |  Status: "));
    metrics_spans.push(Span::styled(&app.status_message, Style::default().fg(Color::DarkGray)));

    metrics_spans.push(Span::raw("  |  Uptime: "));
    metrics_spans.push(Span::styled(uptime_str, Style::default().fg(Color::DarkGray)));

    if let Some(updated) = app.last_updated {
        metrics_spans.push(Span::raw("  |  Updated: "));
        metrics_spans.push(Span::styled(
            updated.format("%H:%M:%S").to_string(),
            Style::default().fg(Color::White),
        ));
    }

    let header_text = vec![Line::from(title_spans), Line::from(metrics_spans)];

    let header_widget = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(header_widget, area);
}
