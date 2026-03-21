mod app;
mod cli;
mod engine;

use clap::Parser;
use crossbeam_channel::unbounded;
use crossterm::{event, terminal::{disable_raw_mode, enable_raw_mode}};
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};
use crate::app::App;
use std::thread;
use crate::engine::{run_engine, EngineCommand};

const MAX_LIST_LENGTH : u16 = 10;

#[derive(Parser)]
struct Args {
    // The directory to search in
    dir: Option<String>
}

fn main() -> anyhow::Result<()> {
    // The PANIC Hook
    // TODO: Remove this
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // If we crash, always restore the terminal before printing the error!
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    let args = Args::parse();
    let (tx_cmd, rx_cmd) = unbounded();
    let (tx_res, rx_res) = unbounded();

    thread::spawn(move || {
        run_engine(rx_cmd, tx_res, args.dir);
    });
    let mut app = App::new(rx_res, tx_cmd, MAX_LIST_LENGTH);

    // setup terminal
    enable_raw_mode()?;
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(1 + MAX_LIST_LENGTH),
        }
    )?;

    let res = run_app(&mut app, &mut terminal);

    let _ = app.tx_cmd.send(EngineCommand::Quit);
    disable_raw_mode()?;
    terminal.clear()?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error running fj: {:?}", err);
    }
    
    if let Some(path) = app.final_selection {
        println!("{}", path);
    }
    Ok(())
}

fn run_app(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>
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
