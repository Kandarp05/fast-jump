const DEPTH_PENALTY: i64 = 3;
const CHILD_OVERRIDE_MARGIN: i64 = 25;
const ACRONYM_BONUS: i64 = 15;

pub fn apply_heuristics(path: &str, raw_score: i64, indices : &[usize]) -> i64 {
    let path_bytes = path.as_bytes();

    // bonus for a match at the boundary of an acronym
    let mut boundary_bonus = 0;
    let last_index = path_bytes.len().saturating_sub(1);

    for &idx in indices {
        if idx == 0 || idx == last_index{
            boundary_bonus += ACRONYM_BONUS;
        } else {
            let prev_char = path_bytes[idx - 1];
            if matches!(prev_char, b'/' | b'-' | b'_' | b' ') {
                boundary_bonus += ACRONYM_BONUS;
            }
        }
    }

    // compute the depth of the path
    // More the depth, more the penalty
    let depth = path_bytes.iter().filter(|&&b| b == b'/').count() as i64;
    let depth_penalty = (depth * DEPTH_PENALTY);

    // compute the final score and return it
    raw_score - depth_penalty + boundary_bonus
}

pub fn is_redundant(path: &str, score: i64, current_results: &[(i64, String)]) -> bool {
    for(existing_score, existing_path) in current_results {
        if path.starts_with(existing_path) {
            if score <= existing_score + CHILD_OVERRIDE_MARGIN {
                return true;
            }
        }
    }
    false
}