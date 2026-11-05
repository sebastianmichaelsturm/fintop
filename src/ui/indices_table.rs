use crate::app::App;
use crate::model::{format_points, SortColumn};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub fn render_indices_table(
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

    let symbol_label = if app.sort_column_indices == SortColumn::Symbol {
        format!("Symbol {}", app.sort_direction_indices.indicator())
    } else {
        "Symbol".to_string()
    };
    let symbol_style = if app.sort_column_indices == SortColumn::Symbol {
        active_style
    } else {
        default_style
    };

    let name_label = if app.sort_column_indices == SortColumn::Name {
        format!("Index Name {}", app.sort_direction_indices.indicator())
    } else {
        "Index Name".to_string()
    };
    let name_style = if app.sort_column_indices == SortColumn::Name {
        active_style
    } else {
        default_style
    };

    let region_label = if app.sort_column_indices == SortColumn::DefaultRegion {
        format!("Region {}", app.sort_direction_indices.indicator())
    } else {
        "Region".to_string()
    };
    let region_style = if app.sort_column_indices == SortColumn::DefaultRegion {
        active_style
    } else {
        default_style
    };

    let points_label = if app.sort_column_indices == SortColumn::Price {
        format!("Points {}", app.sort_direction_indices.indicator())
    } else {
        "Points".to_string()
    };
    let points_style = if app.sort_column_indices == SortColumn::Price {
        active_style
    } else {
        default_style
    };

    let change_label = if app.sort_column_indices == SortColumn::ChangeAbsolute {
        format!("Change {}", app.sort_direction_indices.indicator())
    } else {
        "Change".to_string()
    };
    let change_style = if app.sort_column_indices == SortColumn::ChangeAbsolute {
        active_style
    } else {
        default_style
    };

    let pct_label = if app.sort_column_indices == SortColumn::ChangePercent {
        format!("Chg % {}", app.sort_direction_indices.indicator())
    } else {
        "Chg %".to_string()
    };
    let pct_style = if app.sort_column_indices == SortColumn::ChangePercent {
        active_style
    } else {
        default_style
    };

    let header_cells = if is_maximized {
        vec![
            Cell::from(symbol_label).style(symbol_style),
            Cell::from(name_label).style(name_style),
            Cell::from(region_label).style(region_style),
            Cell::from(points_label).style(points_style),
            Cell::from(change_label).style(change_style),
            Cell::from(pct_label).style(pct_style),
            Cell::from("Day Range (Low - High)").style(default_style),
        ]
    } else {
        vec![
            Cell::from(symbol_label).style(symbol_style),
            Cell::from(name_label).style(name_style),
            Cell::from(region_label).style(region_style),
            Cell::from(points_label).style(points_style),
            Cell::from(pct_label).style(pct_style),
        ]
    };
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let sorted = app.sorted_indices();

    let rows: Vec<Row> = sorted
        .iter()
        .map(|idx| {
            let (change_color, change_prefix) = if idx.change > 0.0001 {
                (Color::Green, "+")
            } else if idx.change < -0.0001 {
                (Color::Red, "")
            } else {
                (Color::DarkGray, "")
            };

            let range_str = if idx.day_low > 0.0 && idx.day_high > 0.0 {
                format!(
                    "{} - {}",
                    format_points(idx.day_low),
                    format_points(idx.day_high)
                )
            } else {
                "-".to_string()
            };

            let cells = if is_maximized {
                vec![
                    Cell::from(idx.clean_symbol().to_string()).style(
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(idx.name.clone()),
                    Cell::from(idx.region.as_str()).style(Style::default().fg(Color::LightBlue)),
                    Cell::from(format_points(idx.current_price))
                        .style(Style::default().fg(Color::Yellow)),
                    Cell::from(format!("{}{:.2}", change_prefix, idx.change))
                        .style(Style::default().fg(change_color)),
                    Cell::from(format!("{}{:.2}%", change_prefix, idx.change_percent)).style(
                        Style::default()
                            .fg(change_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(range_str).style(Style::default().fg(Color::DarkGray)),
                ]
            } else {
                vec![
                    Cell::from(idx.clean_symbol().to_string()).style(
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(idx.name.clone()),
                    Cell::from(idx.region.as_str()).style(Style::default().fg(Color::LightBlue)),
                    Cell::from(format_points(idx.current_price))
                        .style(Style::default().fg(Color::Yellow)),
                    Cell::from(format!("{}{:.2}%", change_prefix, idx.change_percent)).style(
                        Style::default()
                            .fg(change_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]
            };
            Row::new(cells).height(1)
        })
        .collect();

    let title = if is_focused {
        "[1] Global Indices [ACTIVE]"
    } else {
        "[1] Global Indices"
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

    let constraints = if is_maximized {
        vec![
            Constraint::Percentage(12),
            Constraint::Percentage(24),
            Constraint::Percentage(10),
            Constraint::Percentage(16),
            Constraint::Percentage(12),
            Constraint::Percentage(10),
            Constraint::Percentage(16),
        ]
    } else {
        vec![
            Constraint::Percentage(18),
            Constraint::Percentage(32),
            Constraint::Percentage(14),
            Constraint::Percentage(20),
            Constraint::Percentage(16),
        ]
    };

    let table = Table::new(rows, constraints)
        .header(header)
        .block(block)
        .row_highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 44, 52))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(if is_focused { "▶ " } else { "  " });

    frame.render_stateful_widget(table, area, &mut app.indices_table_state);
}
