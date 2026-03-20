mod app;
mod cli;

use clap::Parser;
use crossterm::{
    execute,
    cursor::Show,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};
use crate::app::App;

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
    let mut app = App::new(args, MAX_LIST_LENGTH);

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

    disable_raw_mode()?;
    terminal.clear()?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error running fj: {:?}", err);
    }

    // 5. Output the selection to stdout for the shell wrapper
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
        cli::events::handle_events(app)?;
        if app.should_exit {
            break;
        }
    }

    Ok(())
}
