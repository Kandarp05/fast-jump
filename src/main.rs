mod app;
mod cli;
mod engine;
mod tui;

use crate::app::App;
use crate::engine::{EngineCommand, run_engine};
use crate::tui::Tui;
use clap::Parser;
use crossbeam_channel::unbounded;
use crossterm::{cursor::Show, execute, terminal::disable_raw_mode};
use std::thread;

const MAX_LIST_LENGTH: u16 = 20;

#[derive(Parser)]
struct Args {
    // The directory to search in
    dir: Option<String>,
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

    if let Err(err) = run() {
        eprintln!("There was an error: {}", err);
        std::process::exit(1);
    }
}

pub fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    // Setup the engine
    let (tx_cmd, rx_cmd) = unbounded();
    let (tx_res, rx_res) = unbounded();

    // Start the engine
    thread::spawn(move || {
        run_engine(rx_cmd, tx_res, args.dir, MAX_LIST_LENGTH);
    });

    // Initialize the App and the Terminal
    let mut app = App::new(rx_res, tx_cmd, MAX_LIST_LENGTH);
    let mut tui = Tui::init(MAX_LIST_LENGTH)?;

    app.run_event_loop(&mut tui)?;

    let _ = app.tx_cmd.send(EngineCommand::Quit);

    tui.terminal.clear()?;
    drop(tui);
    if let Some(path) = app.final_selection {
        println!("{}", path);
        let temp_file = std::env::temp_dir().join("fj_target");
        std::fs::write(&temp_file, path)?;
    }

    Ok(())
}
