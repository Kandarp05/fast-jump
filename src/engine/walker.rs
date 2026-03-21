use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crossbeam_channel::Sender;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ignore::WalkBuilder;

pub fn search_disk(
    query: String,
    tx_res: Sender<Vec<String>>,
    kill_switch: Arc<AtomicBool>,
) {
    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(i64, String)> = Vec::new();

    // FIXME: Hardcoded home dir
    let start_dir = dirs::home_dir().unwrap();
    let walker = WalkBuilder::new(&start_dir)
        .hidden(true)
        .parents(true)
        .ignore(true)
        .build();

    for result in walker {
        if kill_switch.load(Ordering::Relaxed) {
            return;
        }

        if let Ok(entry) = result {
            if entry.file_type().map_or(false, |f| f.is_dir()) {
                let path = entry.path().to_string_lossy().to_string();

                if let Some(score) = matcher.fuzzy_match(&path, &query) {
                    if score > 10 {
                        results.push((score, path));
                        results.sort_by(|a, b| b.0.cmp(&a.0));
                        results.truncate(5);
                        let top_5: Vec<String> = results
                            .iter()
                            .map(|(_, p)| p.clone())
                            .collect();

                        let _ = tx_res.send(top_5);
                    }
                }
            }
        }
    }
}