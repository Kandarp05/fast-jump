mod score;
mod walker;

use crossbeam_channel::{Receiver, Sender};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

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
                    // Kill all the prev worker threads
                    if let Some(ks) = current_kill_switch.take() {
                        ks.store(true, Ordering::Relaxed);
                    }

                    // Return empty if the query is empty
                    if query.is_empty() {
                        let _ = tx_result.send(vec![]);
                        continue;
                    }

                    // New kill switch for the new search
                    let kill_switch = Arc::new(AtomicBool::new(false));
                    current_kill_switch = Some(Arc::clone(&kill_switch));
                    let tx_res_clone = tx_result.clone();

                    // If no search space is provided, use the home directory
                    let dir: String = search_space.clone().unwrap_or_else(|| {
                        dirs::home_dir()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| ".".to_string())
                    });

                    thread::spawn(move || {
                        walker::search_disk(query, tx_res_clone, kill_switch, dir, max_list_size);
                    });
                }
                EngineCommand::Quit => {
                    if let Some(ks) = current_kill_switch.take() {
                        ks.store(true, Ordering::Relaxed);
                    }
                    break;
                }
            }
        }
    }
}
