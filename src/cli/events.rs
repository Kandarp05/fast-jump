use crate::app::App;
use crossterm::event::{self, Event, KeyEventKind, KeyCode};
use tui_input::backend::crossterm::EventHandler;
use crate::engine::EngineCommand;

pub fn handle_events(app: &mut App) -> anyhow::Result<()> {
    let event = event::read()?;
    if let Event::Key(key) = event {
        let prev_query = app.input.value().to_string();
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Up => app.move_up(),
                KeyCode::Down => app.move_down(),
                KeyCode::Enter => {
                    if !app.results.is_empty() && app.selected_i < app.results.len() {
                        app.final_selection = Some(app.results[app.selected_i].clone());
                    }
                    app.should_exit = true;
                }
                _ => {
                    app.input.handle_event(&Event::Key(key));
                }
            }
        }

        let new_query = app.input.value().to_string();
        if new_query != prev_query {
            let _ = app.tx_cmd.send(EngineCommand::Search(new_query));
            app.selected_i = 0;
        }
    }

    Ok(())
}