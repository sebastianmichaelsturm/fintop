use crate::app::{App, ViewMode};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let key_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::White);

    let line = if app.is_filtering {
        Line::from(vec![
            Span::styled("Filter: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&app.filter_query, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
            Span::styled("  (Enter: apply, Esc: clear/exit)", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        match app.view_mode {
            ViewMode::Indices => Line::from(vec![
                Span::styled("[q]", key_style),
                Span::styled(" Quit  ", desc_style),
                Span::styled("[Enter]", key_style),
                Span::styled(" View Stocks  ", desc_style),
                Span::styled("[s]", key_style),
                Span::styled(" Sort  ", desc_style),
                Span::styled("[d]", key_style),
                Span::styled(format!(" Dir ({})  ", app.sort_direction_indices.indicator()), desc_style),
                Span::styled("[/]", key_style),
                Span::styled(" Filter  ", desc_style),
                Span::styled("[r]", key_style),
                Span::styled(" Refresh  ", desc_style),
                Span::styled("[Space]", key_style),
                Span::styled(if app.is_paused { " Resume  " } else { " Pause  " }, desc_style),
                Span::styled("[↑/↓, j/k]", key_style),
                Span::styled(" Navigate", desc_style),
            ]),
            ViewMode::Constituents { .. } => Line::from(vec![
                Span::styled("[Esc/q]", key_style),
                Span::styled(" Back to Indices  ", desc_style),
                Span::styled("[s]", key_style),
                Span::styled(" Sort  ", desc_style),
                Span::styled("[d]", key_style),
                Span::styled(format!(" Dir ({})  ", app.sort_direction_stocks.indicator()), desc_style),
                Span::styled("[/]", key_style),
                Span::styled(" Filter  ", desc_style),
                Span::styled("[r]", key_style),
                Span::styled(" Refresh  ", desc_style),
                Span::styled("[Space]", key_style),
                Span::styled(if app.is_paused { " Resume  " } else { " Pause  " }, desc_style),
                Span::styled("[↑/↓, j/k]", key_style),
                Span::styled(" Navigate", desc_style),
            ]),
        }
    };

    let footer = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(footer, area);
}
