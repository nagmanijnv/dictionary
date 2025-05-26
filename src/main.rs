use actix_web::{web, App, HttpServer};
use env_logger::Env;
use std::fs;
use std::path::Path;

use utils::{preload, get_value_from_env};

mod handlers;
mod models;
mod routes;
mod store;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    
    let dir_path = ".temp";
    // Create directory on startup
    if !Path::new(dir_path).exists() {
        fs::create_dir_all(dir_path)?;
        log::info!("Created directory: {}", dir_path);
    }

    // initialise the store
    let semaphore_limit = get_value_from_env("MAX_CONCURRENT_REQUESTS", 100);
    let app_state = store::AppState::init_store(semaphore_limit as usize);
    // preload the state data from existing file
    preload(&app_state);
    let store = web::Data::new(app_state);

    let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = get_value_from_env("PORT", 8888);
    HttpServer::new(move ||
        App::new()
        .wrap(actix_web::middleware::Logger::default())
        .app_data(store.clone())
        .configure(routes::service_config)
    )
    .bind((host, port))?
    .run()
    .await
}
