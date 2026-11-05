use crate::app::App;
use crate::model::{format_rate, SortColumn};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub fn render_currencies_table(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    is_focused: bool,
    is_maximized: bool,
) {
    let active_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let default_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let symbol_label = if app.sort_column_currencies == SortColumn::Symbol {
        format!("Pair {}", app.sort_direction_currencies.indicator())
    } else {
        "Pair".to_string()
    };
    let symbol_style = if app.sort_column_currencies == SortColumn::Symbol {
        active_style
    } else {
        default_style
    };

    let name_label = if app.sort_column_currencies == SortColumn::Name {
        format!(
            "Currency Name {}",
            app.sort_direction_currencies.indicator()
        )
    } else {
        "Currency Name".to_string()
    };
    let name_style = if app.sort_column_currencies == SortColumn::Name {
        active_style
    } else {
        default_style
    };

    let rate_label = if app.sort_column_currencies == SortColumn::Price {
        format!("Rate {}", app.sort_direction_currencies.indicator())
    } else {
        "Rate".to_string()
    };
    let rate_style = if app.sort_column_currencies == SortColumn::Price {
        active_style
    } else {
        default_style
    };

    let pct_label = if app.sort_column_currencies == SortColumn::ChangePercent {
        format!("Chg % {}", app.sort_direction_currencies.indicator())
    } else {
        "Chg %".to_string()
    };
    let pct_style = if app.sort_column_currencies == SortColumn::ChangePercent {
        active_style
    } else {
        default_style
    };

    let header_cells = vec![
        Cell::from(symbol_label).style(symbol_style),
        Cell::from(name_label).style(name_style),
        Cell::from(rate_label).style(rate_style),
        Cell::from(pct_label).style(pct_style),
        Cell::from("Day Range (Lo - Hi)").style(default_style),
    ];
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let sorted = app.sorted_currencies();

    let rows: Vec<Row> = sorted
        .iter()
        .map(|c| {
            let (change_color, change_prefix) = if c.change > 0.000001 {
                (Color::Green, "+")
            } else if c.change < -0.000001 {
                (Color::Red, "")
            } else {
                (Color::DarkGray, "")
            };

            let range_str = if c.day_low > 0.0 && c.day_high > 0.0 {
                format!(
                    "{} - {}",
                    format_rate(c.day_low, &c.base_currency),
                    format_rate(c.day_high, &c.base_currency)
                )
            } else {
                "-".to_string()
            };

            let cells = vec![
                Cell::from(c.clean_symbol().to_string()).style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(c.name.clone()),
                Cell::from(format_rate(c.rate, &c.base_currency))
                    .style(Style::default().fg(Color::Yellow)),
                Cell::from(format!("{}{:.2}%", change_prefix, c.change_percent)).style(
                    Style::default()
                        .fg(change_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(range_str).style(Style::default().fg(Color::DarkGray)),
            ];
            Row::new(cells).height(1)
        })
        .collect();

    let title = if is_focused {
        "[3] Currencies & Crypto [ACTIVE]"
    } else {
        "[3] Currencies & Crypto"
    };

    let border_style = if is_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(title)
        .title_style(if is_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        })
        .borders(if is_maximized {
            Borders::NONE
        } else {
            Borders::ALL
        })
        .border_style(border_style);

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(18),
            Constraint::Percentage(32),
            Constraint::Percentage(18),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
        ],
    )
    .header(header)
    .block(block)
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(40, 44, 52))
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(if is_focused { "▶ " } else { "  " });

    frame.render_stateful_widget(table, area, &mut app.currencies_table_state);
}
