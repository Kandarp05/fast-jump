mod score;
mod walker;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use crossbeam_channel::{Receiver, Sender};

pub enum EngineCommand {
    Search(String),
    Quit,
}

pub fn run_engine(
    rx_cmd: Receiver<EngineCommand>,
    tx_result: Sender<Vec<String>>,
    search_space: Option<String>,
    max_list_size: u16,
) {
    let mut current_kill_switch: Option<Arc<AtomicBool>> = None;
    loop {
        if let Ok(cmd) = rx_cmd.recv() {
            match cmd {
                EngineCommand::Search(query) => {
                    kill_active_search(&mut current_kill_switch);

                    // Return empty if the query is empty
                    if query.is_empty() {
                        let _ = tx_result.send(vec![]);
                        continue;
                    }

                    let dir = resolve_search_directory(&search_space);
                    let tx_res_clone = tx_result.clone();
                    current_kill_switch = Some(spawn_search(
                        tx_res_clone,
                        max_list_size,
                        query,
                        dir,
                    ));
                }
                EngineCommand::Quit => {
                    kill_active_search(&mut current_kill_switch);
                    break;
                }
            }
        }
    }
}

fn spawn_search(
    tx_result: Sender<Vec<String>>,
    max_list_size: u16,
    query: String,
    dir: String,
) -> Arc<AtomicBool> {
    // New kill switch for the new search
    let kill_switch = Arc::new(AtomicBool::new(false));
    let kill_switch_clone = Arc::clone(&kill_switch);
    thread::spawn(move || {
        walker::search_disk(query, tx_result, kill_switch_clone, dir, max_list_size);
    });

    kill_switch
}

fn resolve_search_directory(search_space: &Option<String>) -> String {
    // If no search space is provided, use the home directory
    let dir: String = search_space.clone().unwrap_or_else(|| {
        dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    });
    dir
}

fn kill_active_search(current_kill_switch: &mut Option<Arc<AtomicBool>>) {
    // Kill all the prev worker threads
    if let Some(ks) = current_kill_switch.take() {
        ks.store(true, Ordering::Relaxed);
    }
}
