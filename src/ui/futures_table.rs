use crate::app::App;
use crate::model::{format_futures_price, SortColumn};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub fn render_futures_table(frame: &mut Frame, app: &mut App, area: Rect, is_focused: bool, is_maximized: bool) {
    let active_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let default_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    let symbol_label = if app.sort_column_futures == SortColumn::Symbol {
        format!("Contract {}", app.sort_direction_futures.indicator())
    } else {
        "Contract".to_string()
    };
    let symbol_style = if app.sort_column_futures == SortColumn::Symbol { active_style } else { default_style };

    let name_label = if app.sort_column_futures == SortColumn::Name {
        format!("Underlying {}", app.sort_direction_futures.indicator())
    } else {
        "Underlying".to_string()
    };
    let name_style = if app.sort_column_futures == SortColumn::Name { active_style } else { default_style };

    let price_label = if app.sort_column_futures == SortColumn::Price {
        format!("Price {}", app.sort_direction_futures.indicator())
    } else {
        "Price".to_string()
    };
    let price_style = if app.sort_column_futures == SortColumn::Price { active_style } else { default_style };

    let pct_label = if app.sort_column_futures == SortColumn::ChangePercent {
        format!("Chg % {}", app.sort_direction_futures.indicator())
    } else {
        "Chg %".to_string()
    };
    let pct_style = if app.sort_column_futures == SortColumn::ChangePercent { active_style } else { default_style };

    let header_cells = vec![
        Cell::from(symbol_label).style(symbol_style),
        Cell::from(name_label).style(name_style),
        Cell::from("Category").style(default_style),
        Cell::from(price_label).style(price_style),
        Cell::from(pct_label).style(pct_style),
        Cell::from("Unit").style(default_style),
    ];
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let sorted = app.sorted_futures();

    let rows: Vec<Row> = sorted
        .iter()
        .map(|f| {
            let (change_color, change_prefix) = if f.change > 0.0001 {
                (Color::Green, "+")
            } else if f.change < -0.0001 {
                (Color::Red, "")
            } else {
                (Color::DarkGray, "")
            };

            let cells = vec![
                Cell::from(f.clean_symbol().to_string()).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from(f.name.clone()),
                Cell::from(f.category.clone()).style(Style::default().fg(Color::LightBlue)),
                Cell::from(format_futures_price(f.current_price, &f.unit)).style(Style::default().fg(Color::Yellow)),
                Cell::from(format!("{}{:.2}%", change_prefix, f.change_percent))
                    .style(Style::default().fg(change_color).add_modifier(Modifier::BOLD)),
                Cell::from(f.unit.clone()).style(Style::default().fg(Color::DarkGray)),
            ];
            Row::new(cells).height(1)
        })
        .collect();

    let title = if is_focused {
        "[4] Futures & Commodities [ACTIVE]"
    } else {
        "[4] Futures & Commodities"
    };

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(title)
        .title_style(if is_focused { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) })
        .borders(if is_maximized { Borders::NONE } else { Borders::ALL })
        .border_style(border_style);

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(16),
            Constraint::Percentage(28),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(14),
            Constraint::Percentage(10),
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

    frame.render_stateful_widget(table, area, &mut app.futures_table_state);
}
