pub mod app;
pub mod filter;
pub mod model;
pub mod provider;
pub mod ui;
pub mod worker;

use app::{App, ViewMode};
use clap::Parser;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, stdout},
    time::Duration,
};
use tokio::sync::mpsc;
use worker::MarketWorker;

#[derive(Parser, Debug)]
#[command(
    name = "fintop",
    about = "A real-time top-like terminal dashboard for major market indices and constituent stocks",
    version = "0.1.0"
)]
pub struct Args {
    /// Run in offline demo/simulation mode without live network queries
    #[arg(short, long, default_value_t = false)]
    pub demo: bool,

    /// Background refresh interval in seconds
    #[arg(short, long, default_value_t = 10)]
    pub interval: u64,
}

fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Panic hook ensures clean terminal reset even on panic
    setup_panic_hook();

    // Initialize tokio multi-thread runtime for background data worker
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // Create communication channels between UI and Worker
    let (tx_cmd, rx_cmd) = mpsc::channel(32);
    let (tx_evt, rx_evt) = mpsc::channel(64);

    let mut app = App::new(tx_cmd.clone(), rx_evt, args.demo);

    let index_symbols: Vec<String> = app.indices.iter().map(|i| i.symbol.clone()).collect();
    let worker = MarketWorker::new(rx_cmd, tx_evt, args.demo, args.interval, index_symbols);

    // Spawn worker in tokio runtime
    rt.spawn(worker.run());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout_handle = stdout();
    execute!(stdout_handle, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout_handle);
    let mut terminal = Terminal::new(backend)?;

    let res = run_loop(&mut terminal, &mut app);

    // Teardown terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("fintop error: {err}");
    }

    Ok(())
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        // Drain any incoming market data from background worker
        app.process_worker_events();

        // Render current state
        terminal.draw(|f| ui::draw(f, app))?;

        // Poll keyboard input at ~30 FPS (33ms)
        if event::poll(Duration::from_millis(33))? {
            if let CEvent::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.is_filtering {
                        match key.code {
                            KeyCode::Esc => {
                                app.filter_query.clear();
                                app.is_filtering = false;
                            }
                            KeyCode::Enter => {
                                app.is_filtering = false;
                            }
                            KeyCode::Backspace => {
                                app.filter_query.pop();
                            }
                            KeyCode::Char(c) => {
                                app.filter_query.push(c);
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => {
                                match app.view_mode {
                                    ViewMode::Constituents { .. } => {
                                        app.exit_index();
                                    }
                                    ViewMode::Indices => {
                                        return Ok(());
                                    }
                                }
                            }
                            KeyCode::Esc | KeyCode::Backspace => {
                                if app.view_mode != ViewMode::Indices {
                                    app.exit_index();
                                }
                            }
                            KeyCode::Enter => {
                                if app.view_mode == ViewMode::Indices {
                                    app.enter_index();
                                }
                            }
                            KeyCode::Char('/') => {
                                app.is_filtering = true;
                            }
                            KeyCode::Char('s') => {
                                app.cycle_sort();
                            }
                            KeyCode::Char('d') => {
                                app.toggle_sort_direction();
                            }
                            KeyCode::Char(' ') => {
                                app.toggle_pause();
                            }
                            KeyCode::Char('r') => {
                                app.trigger_refresh();
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.previous_row();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.next_row();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
