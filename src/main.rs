mod app;
mod cli;
mod engine;

use crate::app::App;
use crate::engine::{EngineCommand, run_engine};
use clap::Parser;
use crossbeam_channel::unbounded;
use crossterm::cursor::Show;
use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};
use std::thread;

const MAX_LIST_LENGTH: u16 = 10;

#[derive(Parser)]
struct Args {
    // The directory to search in
    dir: Option<String>,
}

struct TerminalGuard;

impl TerminalGuard {
    fn init() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), Show);
    }
}

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let mut stdout = std::io::stdout();
        let _ = disable_raw_mode();
        let _ = execute!(stdout, Show);
        original_hook(panic_info);
    }));
}

fn main() {
    install_panic_hook();

    if let Err(err) = exec_app() {
        eprintln!("There was an error: {}", err);
        std::process::exit(1);
    }
}

fn exec_app() -> anyhow::Result<()> {
    let args = Args::parse();
    let (tx_cmd, rx_cmd) = unbounded();
    let (tx_res, rx_res) = unbounded();

    thread::spawn(move || {
        run_engine(rx_cmd, tx_res, args.dir);
    });
    let mut app = App::new(rx_res, tx_cmd, MAX_LIST_LENGTH);
    let _guard = TerminalGuard::init()?;

    // setup terminal
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(1 + MAX_LIST_LENGTH),
        },
    )?;

    let res = tui_loop(&mut app, &mut terminal);

    let _ = app.tx_cmd.send(EngineCommand::Quit);

    if let Err(err) = res {
        eprintln!("Error running fj: {:?}", err);
    }

    if let Some(path) = app.final_selection {
        println!("{}", path);
        let temp_file = std::env::temp_dir().join("fj_target");
        let _ = std::fs::write(&temp_file, path);
    }

    Ok(())
}

fn tui_loop(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| cli::render::draw(f, app))?;

        let mut received_new_res = false;
        while let Ok(new_res) = app.rx_res.try_recv() {
            app.results = new_res;

            if app.selected_i >= app.results.len() {
                app.selected_i = app.results.len().saturating_sub(1);
            }
            received_new_res = true;
        }

        if received_new_res {
            continue;
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            cli::events::handle_events(app)?;
        }

        if app.should_exit {
            break;
        }
    }

    Ok(())
}
