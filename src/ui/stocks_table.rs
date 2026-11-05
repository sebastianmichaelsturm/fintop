use crate::app::App;
use crate::model::{format_market_cap, format_price, SortColumn};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub fn render_stocks_table(frame: &mut Frame, app: &mut App, area: Rect, is_focused: bool, is_maximized: bool) {
    let active_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let default_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    let symbol_label = if app.sort_column_stocks == SortColumn::Symbol {
        format!("Symbol {}", app.sort_direction_stocks.indicator())
    } else {
        "Symbol".to_string()
    };
    let symbol_style = if app.sort_column_stocks == SortColumn::Symbol { active_style } else { default_style };

    let name_label = if app.sort_column_stocks == SortColumn::Name {
        format!("Name {}", app.sort_direction_stocks.indicator())
    } else {
        "Name".to_string()
    };
    let name_style = if app.sort_column_stocks == SortColumn::Name { active_style } else { default_style };

    let price_label = if app.sort_column_stocks == SortColumn::Price {
        format!("Price {}", app.sort_direction_stocks.indicator())
    } else {
        "Price".to_string()
    };
    let price_style = if app.sort_column_stocks == SortColumn::Price { active_style } else { default_style };

    let pct_label = if app.sort_column_stocks == SortColumn::ChangePercent {
        format!("Chg % {}", app.sort_direction_stocks.indicator())
    } else {
        "Chg %".to_string()
    };
    let pct_style = if app.sort_column_stocks == SortColumn::ChangePercent { active_style } else { default_style };

    let cap_label = if app.sort_column_stocks == SortColumn::MarketCap {
        format!("Mkt Cap {}", app.sort_direction_stocks.indicator())
    } else {
        "Mkt Cap".to_string()
    };
    let cap_style = if app.sort_column_stocks == SortColumn::MarketCap { active_style } else { default_style };

    let header_cells = vec![
        Cell::from(symbol_label).style(symbol_style),
        Cell::from(name_label).style(name_style),
        Cell::from(price_label).style(price_style),
        Cell::from(pct_label).style(pct_style),
        Cell::from(cap_label).style(cap_style),
    ];
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let sorted = app.sorted_top_stocks();

    let rows: Vec<Row> = sorted
        .iter()
        .map(|stock| {
            let (change_color, change_prefix) = if stock.change > 0.0001 {
                (Color::Green, "+")
            } else if stock.change < -0.0001 {
                (Color::Red, "")
            } else {
                (Color::DarkGray, "")
            };

            let cells = vec![
                Cell::from(stock.symbol.clone()).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from(stock.name.clone()),
                Cell::from(format_price(stock.current_price, &stock.currency)).style(Style::default().fg(Color::Yellow)),
                Cell::from(format!("{}{:.2}%", change_prefix, stock.change_percent))
                    .style(Style::default().fg(change_color).add_modifier(Modifier::BOLD)),
                Cell::from(format_market_cap(stock.market_cap, &stock.currency))
                    .style(Style::default().fg(Color::LightCyan)),
            ];
            Row::new(cells).height(1)
        })
        .collect();

    let title = if is_focused {
        "[2] Top Stocks [ACTIVE]"
    } else {
        "[2] Top Stocks"
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

    frame.render_stateful_widget(table, area, &mut app.stocks_table_state);
}
