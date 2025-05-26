use std::collections::BTreeMap;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task;
use actix_web::web;

use crate::models::{RandomWord, DictionaryError};
use crate::store::AppState;
use crate::utils::calculate_stats;

const REMOTE_URL: &str = "https://random-words-api.vercel.app/word"; 


/// Handler for Generating Dictionary
/// Uses a Semaphor to limit external api call to prevent too Many Request Issue
pub async fn generate_dictionary_handler(name: String, count: u32, app_data: web::Data<AppState>) -> Result<BTreeMap<char, usize>, DictionaryError> {
    println!("starting generate_dictionary_handler");
    let mut words: Vec<RandomWord> = Vec::with_capacity(count as usize);
    
    // send request for getting word
    let client = reqwest::Client::new();

    log::info!("triggering {} requests", count);
    let mut join_set = task::JoinSet::new();
    for i in 0..count {
        // acquire permit to spawn new task
        let permit = app_data.get_permit().await;
        let client = client.clone();

        join_set.spawn(async move {
            println!("spawning task : {i}");
            let _permit = permit;
            client.get(REMOTE_URL).send().await
        });
    }

    log::info!("start consuming response");
    while let Some(res) = join_set.join_next().await {
        match res {
            // success
            Ok(Ok(resp)) => {
                if !resp.status().is_success() {
                    eprintln!("A task returned error: with status : {}", resp.status());
                    // abort all pending tasks
                    join_set.shutdown().await;
                    return Err(
                        DictionaryError::RemoteReqFailed(
                            resp.text().await
                            .map_err(|_| DictionaryError::FailedToDeserialise)?
                        )
                    );
                }
                let word = resp.json::<RandomWord>().await
                    .map_err(|_| DictionaryError::FailedToDeserialise)?;

                words.push(word);
            },
            // Error from remote request
            Ok(Err(e)) => {
                eprintln!("A task returned error: {:?}", e);
                // abort all pending tasks
                join_set.shutdown().await;
                return Err(DictionaryError::RemoteReqFailed(e.to_string()));
            }
            // Join error
            Err(e) => {
                eprintln!("Task panicked or was aborted: {:?}", e);
                // abort all pending tasks
                join_set.shutdown().await;
                return Err(DictionaryError::JoinError(e.to_string()));
            }
        }
    }
    log::info!("finish consuming response");

    // sort the words 
    words.sort_by(|a, b| a.word.cmp(&b.word));

    // calculate stats
    let stats = calculate_stats(&words);

    log::info!("Starting writing to file");
    // write data to file
    let mut file = File::create(format!(".temp/{name}.txt")).await.map_err(|_| DictionaryError::FailedFileIO)?;
    let file_data = words.into_iter()
        .map(|word| format!(
            "{word}: {pronunciation}, {meaning}",
            word = word.word,
            pronunciation = word.pronunciation,
            meaning = word.definition,
        ))
        .collect::<Vec<String>>()
        .join("\n");
    
    file.write(&file_data.into_bytes()).await.map_err(|_| DictionaryError::FailedFileIO)?;
    log::info!("Finishing writing to file");

    Ok(stats)
}