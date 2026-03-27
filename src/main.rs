mod app;
mod cli;
mod engine;
mod tui;

use std::thread;

use clap::Parser;
use crossbeam_channel::unbounded;
use crossterm::{cursor::Show, execute, terminal::disable_raw_mode};

use crate::app::App;
use crate::engine::db::FrecencyDB;
use crate::engine::{EngineCommand, EngineResult, db, run_engine};
use crate::tui::Tui;

const MAX_LIST_LENGTH: u16 = 10;

#[derive(Parser)]
struct Args {
    // The directory to search in
    dir: Option<String>,

    #[arg(long)]
    headless: bool,

    #[arg(long)]
    query: Option<String>,
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

// 1. The Coordinator
pub fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    // Common Infrastructure Setup
    let (tx_cmd, rx_cmd) = unbounded();
    let (tx_res, rx_res) = unbounded();

    let db = db::load_db();
    let engine_db = db.clone();

    // Spawn the Engine (Shared by both modes)
    thread::spawn(move || {
        run_engine(rx_cmd, tx_res, args.dir, engine_db, MAX_LIST_LENGTH);
    });

    // Dispatch
    if args.headless {
        let query = args
            .query
            .ok_or_else(|| anyhow::anyhow!("--query is required in headless mode"))?;
        start_headless_session(query, tx_cmd, rx_res)
    } else {
        start_tui_session(tx_cmd, rx_res, db)
    }
}

fn start_tui_session(
    tx_cmd: crossbeam_channel::Sender<EngineCommand>,
    rx_res: crossbeam_channel::Receiver<EngineResult>,
    db: FrecencyDB,
) -> anyhow::Result<()> {
    // Initialize
    let mut app = App::new(rx_res, tx_cmd.clone(), MAX_LIST_LENGTH);
    let mut tui = Tui::init(MAX_LIST_LENGTH)?;

    tx_cmd.send(EngineCommand::Search(String::new()))?;

    // Run Loop
    app.run_event_loop(&mut tui)?;

    // Cleanup
    let _ = tx_cmd.send(EngineCommand::Quit);
    tui.terminal.clear()?;
    drop(tui);

    if let Some(path) = app.final_selection {
        // Update the frecency map with the selected path and save it
        db::update_and_save_db(db, path.clone());

        // Print the output and also save it to a temporary file
        println!("{}", path);
        let temp_file = std::env::temp_dir().join("fj_target");
        std::fs::write(&temp_file, path)?;
    }

    Ok(())
}

fn start_headless_session(
    query: String,
    tx_cmd: crossbeam_channel::Sender<EngineCommand>,
    rx_res: crossbeam_channel::Receiver<EngineResult>,
) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    // Execute
    tx_cmd.send(EngineCommand::Search(query))?;

    let mut count = 0;
    loop {
        match rx_res.recv() {
            Ok(EngineResult::Update(results)) => count = results.len(),
            Ok(EngineResult::Done) => break,
            Err(_) => break,
        }
    }

    // Report
    let duration = start.elapsed();
    println!("Search completed in {:.2?}", duration);
    println!("Found {} results", count);

    // Cleanup
    let _ = tx_cmd.send(EngineCommand::Quit);
    Ok(())
}
