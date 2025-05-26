use actix_web::{delete, get, post, web, HttpResponse};
use actix_web::http::header;
use futures::StreamExt;
use std::path::PathBuf;
use tokio::task;
use tokio::fs::{remove_file, File};
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::handlers::generate_dictionary_handler;
use crate::models::{DictGenerateRequest, DictionaryError, DictionaryLocalState, DictionaryStatus};
use crate::store::AppState;


#[post("/api/v1/dict/generate")]
async fn generate_dictionary(
    req_body: web::Json<DictGenerateRequest>,
    app_data: web::Data<AppState>,
) -> Result<HttpResponse, DictionaryError> {
    let payload = req_body.into_inner();

    // check if dictionary already exist
    match app_data.get_entry(&payload.dict_name) {
        Some(data) => {
            let msg = match data.status {
                DictionaryStatus::Completed => "completed",
                DictionaryStatus::Failed(_) => "failed",
                DictionaryStatus::InProgress => "in-progress",
            };

            return Err(DictionaryError::EntryExist(msg.to_string()));
        },
        None => {
            // create entry into local store
            app_data.set_dict_data(payload.dict_name.clone(), DictionaryLocalState::default());
            let cloned_app_data = app_data.clone();
            task::spawn(async move {
                match generate_dictionary_handler(
                    payload.dict_name.clone(),
                    payload.word_count.clone(),
                    app_data,
                ).await {
                    Ok(stats) => {
                        let dict_data = DictionaryLocalState::default()
                            .set_status(DictionaryStatus::Completed)
                            .set_stats(stats);

                        cloned_app_data.set_dict_data(payload.dict_name.clone(), dict_data);
                    },
                    Err(e) => {
                        println!("some error occured: {}", e.to_string());
                        cloned_app_data.update_dict_status(&payload.dict_name, DictionaryStatus::Failed(e.to_string()));
                    }
                }
            });

            Ok(
                HttpResponse::Accepted().json(
                    serde_json::json!({
                        "message": "Dictionary generation started",
                        "status": true,
                    })
                )
            )           
        }
    }

}

#[get("/api/v1/dict/{dict_name}/status")]
async fn get_dictionary_status(
    name: web::Path<String>,
    app_data: web::Data<AppState>,
) -> HttpResponse {
    let dict_name = name.into_inner();

    match app_data.get_dict_status(&dict_name) {
        Some(status) => {
            let msg = match status {
                DictionaryStatus::Completed => "Completed",
                DictionaryStatus::Failed(_) => "Failed",
                DictionaryStatus::InProgress => "InProgress",
            };

            HttpResponse::Ok().json(
                serde_json::json!({
                    "message": "Dictionary exist",
                    "status": msg,
                })
            )
        },
        None => {
            HttpResponse::NotFound().json(
                serde_json::json!({
                    "message": "Dictionary does not exist",
                    "status": false,
                })
            )
        }
    }
}

#[delete("/api/v1/dict/{dict_name}")]
async fn delete_dictionary(
    name: web::Path<String>,
    app_data: web::Data<AppState>,
) -> Result<HttpResponse, DictionaryError> {
    let dict_name = name.into_inner();

    match app_data.delete_entry(&dict_name) {
        Some(status) => {
            match status {
                DictionaryStatus::Completed => {
                    let file_path = PathBuf::from(format!(".temp/{}.txt", dict_name));
                    let _ = remove_file(file_path).await
                        .map_err(|e| DictionaryError::NotFound(e.to_string()))?;
                },
                _ => {
                    println!("file does not exist for these statuses");
                }
            }
            
            Ok(
                HttpResponse::Ok().json(
                    serde_json::json!({
                        "message": "deleted successfully",
                        "status": true
                    })
                )
            )
        },
        None => Ok(HttpResponse::NotFound().json(
            serde_json::json!({
                "message": "dictionary does not exist, failed to delete",
                "status": false
            })
        ))
    }
}

#[get("/api/v1/dict/{dict_name}/download")]
async fn download_dictionary(
    name: web::Path<String>,
    app_data: web::Data<AppState>,
) -> Result<HttpResponse, DictionaryError> {

    let dict_name = name.into_inner();

    // check if dictionary exist of not
    match app_data.get_dict_status(&dict_name) {
        Some(status) => {
            match status {
                DictionaryStatus::Completed => {
                    let file_path = PathBuf::from(format!(".temp/{}.txt", dict_name));

                    let file = File::open(file_path).await
                        .map_err(|e| DictionaryError::NotFound(e.to_string()))?;
                    let stream = FramedRead::new(file, BytesCodec::new())
                        .map(|res| res.map(|bytes_mut| bytes_mut.freeze()));

                    Ok(
                        HttpResponse::Ok()
                        .insert_header((header::CONTENT_TYPE, "text/plain"))
                        .insert_header((header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}.txt\"", dict_name)))
                        .streaming(stream)
                    )
                }
                DictionaryStatus::Failed(e) => Err(DictionaryError::NotFound(format!("status Failed: {e}"))),
                DictionaryStatus::InProgress => Err(DictionaryError::NotFound("status Inprogress, still being generated".to_string())),
            }
            
        },
        None => Err(DictionaryError::NotFound("no status found".to_string()))
    }
}

#[get("/api/v1/dict/{dict_name}/statistics")]
async fn get_dictionary_statistics(
    name: web::Path<String>,
    app_data: web::Data<AppState>,
) -> HttpResponse {
    let dict_name = name.into_inner();

    match app_data.get_entry(&dict_name) {
        Some(data) => {
            let msg = match data.status {
                DictionaryStatus::Completed => serde_json::json!({
                    "message": "Dictionary exist",
                    "stats": data.stats,
                }),
                DictionaryStatus::Failed(_) => serde_json::json!({
                    "message": "Dictionary exist, but in failed state",
                    "stats": data.stats,
                }),
                DictionaryStatus::InProgress => serde_json::json!({
                    "message": "Dictionary is still being generated",
                    "stats": data.stats,
                }),
            };

            HttpResponse::Ok().json(msg)
        },
        None => {
            HttpResponse::NotFound().json(
                serde_json::json!({
                    "message": "Dictionary does not exist",
                    "status": false,
                })
            )
        }
    }
}

pub fn service_config(cfg: &mut web::ServiceConfig) {
    cfg.service(generate_dictionary);
    cfg.service(get_dictionary_status);
    cfg.service(delete_dictionary);
    cfg.service(download_dictionary);
    cfg.service(get_dictionary_statistics);
}
