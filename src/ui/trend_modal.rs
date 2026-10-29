use crate::app::TrendState;
use crate::model::{format_price, TimeRange};
use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, BorderType, Borders, Chart, Clear, Dataset, GraphType, Paragraph},
    Frame,
};

pub fn render_trend_modal(frame: &mut Frame, trend: &TrendState, area: Rect) {
    let modal_area = centered_rect(86, 82, area);

    // Clear background to avoid bleed-through from underlying tables
    frame.render_widget(Clear, modal_area);

    let (change_abs, change_pct) = trend.period_change();
    let is_pos = change_abs >= 0.0;
    let change_color = if is_pos { Color::Green } else { Color::Red };
    let change_prefix = if is_pos { "+" } else { "" };

    let title_line = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            &trend.symbol,
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" — {} — ", trend.name), Style::default().fg(Color::White)),
        Span::styled(
            format!("[{} Trend]", trend.time_range.as_str()),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default()),
    ]);

    let border_style = Style::default().fg(Color::Cyan);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(title_line)
        .title_alignment(Alignment::Center);

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Metrics & Timeframe buttons
            Constraint::Min(8),    // Braille Chart
            Constraint::Length(1), // Shortcut help
        ])
        .split(inner);

    render_modal_header(frame, trend, change_abs, change_pct, change_color, change_prefix, chunks[0]);

    render_modal_chart(frame, trend, change_color, chunks[1]);

    render_modal_footer(frame, chunks[2]);
}

fn render_modal_header(
    frame: &mut Frame,
    trend: &TrendState,
    change_abs: f64,
    change_pct: f64,
    change_color: Color,
    change_prefix: &str,
    area: Rect,
) {
    let open_str = trend
        .period_open()
        .map(|p| format_price(p, &trend.currency))
        .unwrap_or_else(|| "-".to_string());
    let high_str = trend
        .period_max()
        .map(|p| format_price(p, &trend.currency))
        .unwrap_or_else(|| "-".to_string());
    let low_str = trend
        .period_min()
        .map(|p| format_price(p, &trend.currency))
        .unwrap_or_else(|| "-".to_string());
    let curr_str = format_price(trend.current_price, &trend.currency);

    let mut lines = Vec::new();

    // Line 1: Current Price & Period Return
    lines.push(Line::from(vec![
        Span::styled(" Price: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            curr_str,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled("   Period Return: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(
                "{}{:.2} ({}{:.2}%)",
                change_prefix, change_abs, change_prefix, change_pct
            ),
            Style::default().fg(change_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("   Currency: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&trend.currency, Style::default().fg(Color::Yellow)),
    ]));

    // Line 2: Stats (Open, High, Low)
    lines.push(Line::from(vec![
        Span::styled(" Open: ", Style::default().fg(Color::DarkGray)),
        Span::styled(open_str, Style::default().fg(Color::White)),
        Span::styled("   High: ", Style::default().fg(Color::DarkGray)),
        Span::styled(high_str, Style::default().fg(Color::Green)),
        Span::styled("   Low: ", Style::default().fg(Color::DarkGray)),
        Span::styled(low_str, Style::default().fg(Color::Red)),
        Span::styled("   Data Points: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", trend.points.len()),
            Style::default().fg(Color::White),
        ),
    ]));

    // Line 3: Timeframe Buttons
    let mut tf_spans = vec![Span::styled(" Timeframe: ", Style::default().fg(Color::DarkGray))];
    for (idx, r) in TimeRange::ALL.iter().enumerate() {
        let key_num = idx + 1;
        if *r == trend.time_range {
            tf_spans.push(Span::styled(
                format!(" [>{}:{}<] ", key_num, r.as_str()),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(30, 40, 60)),
            ));
        } else {
            tf_spans.push(Span::styled(
                format!(" [{}:{}] ", key_num, r.as_str()),
                Style::default().fg(Color::Gray),
            ));
        }
    }
    if trend.is_loading {
        tf_spans.push(Span::styled(
            " ⟳ Loading data...",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
        ));
    }
    lines.push(Line::from(tf_spans));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn render_modal_chart(
    frame: &mut Frame,
    trend: &TrendState,
    line_color: Color,
    area: Rect,
) {
    if trend.points.is_empty() {
        let msg = if trend.is_loading {
            "⟳ Fetching historical quotes from Yahoo Finance..."
        } else {
            "No historical quote data available for this range."
        };
        let p = Paragraph::new(msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        let centered = centered_rect(60, 30, area);
        frame.render_widget(p, centered);
        return;
    }

    let min_p = trend.period_min().unwrap_or(0.0);
    let max_p = trend.period_max().unwrap_or(100.0);
    let span_p = (max_p - min_p).abs();
    let margin = if span_p > 0.0001 { span_p * 0.03 } else { 1.0 };
    let y_bounds = [min_p - margin, max_p + margin];

    let data_coords: Vec<(f64, f64)> = trend
        .points
        .iter()
        .enumerate()
        .map(|(i, p)| (i as f64, p.price))
        .collect();

    let x_max = (trend.points.len().max(2) - 1) as f64;
    let x_bounds = [0.0, x_max];

    // X-Axis labels
    let n = trend.points.len();
    let x_labels = if n >= 3 {
        let first_ts = trend.points[0].timestamp;
        let mid_ts = trend.points[n / 2].timestamp;
        let last_ts = trend.points[n - 1].timestamp;
        vec![
            Span::raw(format_point_time(first_ts, trend.time_range)),
            Span::raw(format_point_time(mid_ts, trend.time_range)),
            Span::raw(format_point_time(last_ts, trend.time_range)),
        ]
    } else {
        vec![Span::raw("Start"), Span::raw("End")]
    };

    // Y-Axis labels
    let y_labels = vec![
        Span::raw(format_price(min_p, &trend.currency)),
        Span::raw(format_price((min_p + max_p) / 2.0, &trend.currency)),
        Span::raw(format_price(max_p, &trend.currency)),
    ];

    let dataset = Dataset::default()
        .name(format!("{} Trend", trend.time_range.as_str()))
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(line_color))
        .data(&data_coords);

    let x_axis = Axis::default()
        .title(Span::styled("Timeline", Style::default().fg(Color::DarkGray)))
        .style(Style::default().fg(Color::DarkGray))
        .bounds(x_bounds)
        .labels(x_labels);

    let y_axis = Axis::default()
        .title(Span::styled(
            format!("Price ({})", trend.currency),
            Style::default().fg(Color::DarkGray),
        ))
        .style(Style::default().fg(Color::DarkGray))
        .bounds(y_bounds)
        .labels(y_labels);

    let chart = Chart::new(vec![dataset])
        .x_axis(x_axis)
        .y_axis(y_axis);

    frame.render_widget(chart, area);
}

fn render_modal_footer(frame: &mut Frame, area: Rect) {
    let shortcuts = Line::from(vec![
        Span::styled(" [t/Tab/←/→] ", Style::default().fg(Color::Yellow).bold()),
        Span::styled("Cycle Range  ", Style::default().fg(Color::White)),
        Span::styled("[1-6] ", Style::default().fg(Color::Yellow).bold()),
        Span::styled("Direct Range  ", Style::default().fg(Color::White)),
        Span::styled("[r] ", Style::default().fg(Color::Yellow).bold()),
        Span::styled("Refresh  ", Style::default().fg(Color::White)),
        Span::styled("[Esc/q/Bksp] ", Style::default().fg(Color::Yellow).bold()),
        Span::styled("Close Window", Style::default().fg(Color::White)),
    ]);

    let p = Paragraph::new(shortcuts).alignment(Alignment::Center);
    frame.render_widget(p, area);
}

fn format_point_time(ts: i64, range: TimeRange) -> String {
    let dt = chrono::DateTime::from_timestamp(ts, 0)
        .map(|utc| utc.with_timezone(&Local));

    if let Some(local) = dt {
        match range {
            TimeRange::OneDay => local.format("%H:%M").to_string(),
            TimeRange::FiveDays => local.format("%d %b %H:%M").to_string(),
            TimeRange::OneMonth | TimeRange::SixMonths => local.format("%d %b").to_string(),
            TimeRange::OneYear => local.format("%b %Y").to_string(),
            TimeRange::FiveYears => local.format("%Y").to_string(),
        }
    } else {
        String::new()
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
