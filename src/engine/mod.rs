mod walker;
mod score;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use crossbeam_channel::{Receiver, Sender};

pub enum EngineCommand {
    Search(String),
    Quit,
}

pub fn run_engine(
    rx_cmd : Receiver<EngineCommand>,
    tx_result : Sender<Vec<String>>,
    search_space : Option<String>,
) {
    let mut current_kill_switch : Option<Arc<AtomicBool>> = None;
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
                    let dir = search_space.clone().unwrap_or_else(|| "./".to_string());
                    thread::spawn(move || {
                        walker::search_disk(query, tx_res_clone, kill_switch, dir);
                    });
                },
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