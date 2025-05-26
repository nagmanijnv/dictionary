use rayon::prelude::*;
use std::{fs, path::PathBuf};
use std::io::{BufRead, BufReader};
use std::collections::BTreeMap;

use crate::models::{DictionaryLocalState, DictionaryStatus, RandomWord};
use crate::store::AppState;

/// Preload the exisiting data into state
pub fn preload(state: &AppState) {
    println!("starting preloading flow");
    let data = preload_local_state(".temp");

    data.par_iter().for_each(|(name, dict_state)| {
        state.set_dict_data(name.to_string(), dict_state.clone());
    });
    println!("finishing preloading flow");
}

fn preload_local_state(dir: &str) -> Vec<(String, DictionaryLocalState)> {
    let mut final_data = Vec::new();
    let path = PathBuf::from(dir);
    if path.exists() && path.is_dir() {
        for entry in fs::read_dir(path).expect("Failed to read dir") {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                if let Ok(file) = fs::File::open(&file_path) {
                    let dict_name = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
                    println!("dict  name: {}", dict_name);
                    let reader = BufReader::new(file);

                    let mut entries = Vec::new();

                    for line_result in reader.lines() {
                        if let Ok(line) = line_result {
                            if let Some(entry) = parse_line_to_random_word(&line) {
                                entries.push(entry);
                            }
                        }
                    }

                    let dict_state = DictionaryLocalState {
                        status: DictionaryStatus::Completed,
                        stats: Some(calculate_stats(&entries))
                    };

                    final_data.push((dict_name.to_string(), dict_state));
                }
            }
        }
    }
    final_data
}

/// Parse line and converts it to RandomWord
fn parse_line_to_random_word(line: &str) -> Option<RandomWord> {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() == 2 {
        let new_parts: Vec<&str> = parts[1].splitn(2, ',').collect();
        Some(RandomWord {
            word: parts[0].trim().to_string(),
            pronunciation: new_parts[0].to_string(),
            definition: new_parts[1].to_string(),
        })
    } else {
        None
    }
}

/// Calculates statistics of a dictionary
/// counts number of words starting with each letter in the alphabet
pub fn calculate_stats(words: &Vec<RandomWord>) -> BTreeMap<char, usize> {
    let mut stats: BTreeMap<char, usize> = BTreeMap::new();
    for rword in words {
        if let Some(first_char) = rword.word.chars().next() {
            if first_char.is_ascii_alphabetic() {
                *stats.entry(first_char).or_insert(0) += 1;
            }
        }
    }
    stats
}

/// Fetch value from env based on key
pub fn get_value_from_env(key: &str, default: u16) -> u16 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(default) // fallback default
}