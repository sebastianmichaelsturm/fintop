use crate::app::App;
use crate::model::{format_market_cap, format_price, format_volume, SortColumn};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub fn render_constituents_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let active_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let default_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let symbol_label = if app.sort_column_stocks == SortColumn::Symbol {
        format!("Symbol {}", app.sort_direction_stocks.indicator())
    } else {
        "Symbol".to_string()
    };
    let symbol_style = if app.sort_column_stocks == SortColumn::Symbol {
        active_style
    } else {
        default_style
    };

    let name_label = if app.sort_column_stocks == SortColumn::Name {
        format!("Company Name {}", app.sort_direction_stocks.indicator())
    } else {
        "Company Name".to_string()
    };
    let name_style = if app.sort_column_stocks == SortColumn::Name {
        active_style
    } else {
        default_style
    };

    let mcap_label = if app.sort_column_stocks == SortColumn::MarketCap {
        format!("Market Cap {}", app.sort_direction_stocks.indicator())
    } else {
        "Market Cap".to_string()
    };
    let mcap_style = if app.sort_column_stocks == SortColumn::MarketCap {
        active_style
    } else {
        default_style
    };

    let price_label = if app.sort_column_stocks == SortColumn::Price {
        format!("Price {}", app.sort_direction_stocks.indicator())
    } else {
        "Price".to_string()
    };
    let price_style = if app.sort_column_stocks == SortColumn::Price {
        active_style
    } else {
        default_style
    };

    let change_label = if app.sort_column_stocks == SortColumn::ChangeAbsolute {
        format!("Change {}", app.sort_direction_stocks.indicator())
    } else {
        "Change".to_string()
    };
    let change_style = if app.sort_column_stocks == SortColumn::ChangeAbsolute {
        active_style
    } else {
        default_style
    };

    let pct_label = if app.sort_column_stocks == SortColumn::ChangePercent {
        format!("Change % {}", app.sort_direction_stocks.indicator())
    } else {
        "Change %".to_string()
    };
    let pct_style = if app.sort_column_stocks == SortColumn::ChangePercent {
        active_style
    } else {
        default_style
    };

    let vol_label = if app.sort_column_stocks == SortColumn::Volume {
        format!("Volume {}", app.sort_direction_stocks.indicator())
    } else {
        "Volume".to_string()
    };
    let vol_style = if app.sort_column_stocks == SortColumn::Volume {
        active_style
    } else {
        default_style
    };

    let header_cells = vec![
        Cell::from(symbol_label).style(symbol_style),
        Cell::from(name_label).style(name_style),
        Cell::from(mcap_label).style(mcap_style),
        Cell::from(price_label).style(price_style),
        Cell::from(change_label).style(change_style),
        Cell::from(pct_label).style(pct_style),
        Cell::from("Day High").style(default_style),
        Cell::from("Day Low").style(default_style),
        Cell::from(vol_label).style(vol_style),
    ];
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let sorted = app.sorted_constituents();

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
                Cell::from(stock.symbol.clone()).style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(stock.name.clone()),
                Cell::from(format_market_cap(stock.market_cap, &stock.currency)).style(
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(format_price(stock.current_price, &stock.currency))
                    .style(Style::default().fg(Color::Yellow)),
                Cell::from(format!("{}{:.2}", change_prefix, stock.change))
                    .style(Style::default().fg(change_color)),
                Cell::from(format!("{}{:.2}%", change_prefix, stock.change_percent)).style(
                    Style::default()
                        .fg(change_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(format_price(stock.day_high, &stock.currency))
                    .style(Style::default().fg(Color::DarkGray)),
                Cell::from(format_price(stock.day_low, &stock.currency))
                    .style(Style::default().fg(Color::DarkGray)),
                Cell::from(format_volume(stock.volume))
                    .style(Style::default().fg(Color::LightCyan)),
            ];
            Row::new(cells).height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(10), // Symbol
            Constraint::Percentage(22), // Company Name
            Constraint::Percentage(12), // Market Cap
            Constraint::Percentage(12), // Price
            Constraint::Percentage(10), // Change
            Constraint::Percentage(10), // Change %
            Constraint::Percentage(9),  // Day High
            Constraint::Percentage(9),  // Day Low
            Constraint::Percentage(8),  // Volume
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::NONE))
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(40, 44, 52))
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, area, &mut app.stocks_table_state);
}
