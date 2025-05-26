use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::models::{DictionaryLocalState, DictionaryStatus};

#[derive(Clone)]
pub struct AppState {
    map: Arc<DashMap<String, DictionaryLocalState>>,
    permits: Arc<Semaphore>,
}

impl AppState {
    pub fn init_store(limit: usize) -> AppState {
        AppState {
            map: Arc::new(DashMap::new()),
            permits: Arc::new(Semaphore::new(limit)),
        }
    }

    pub async fn get_permit(&self) -> OwnedSemaphorePermit {
        self.permits.clone().acquire_owned().await.unwrap()
    }

    pub fn get_entry(&self, dict_name: &str) -> Option<DictionaryLocalState> {
        self.map.get(dict_name).map(|item| item.clone())
    }

    pub fn get_dict_status(&self, dict_name: &str) -> Option<DictionaryStatus> {
        self.map.get(dict_name).map(|entry| entry.status.clone())
    }

    pub fn set_dict_data(&self, dict_name: String, data: DictionaryLocalState) {
        self.map.insert(dict_name, data);
    }

    pub fn update_dict_status(&self, dict_name: &str, status: DictionaryStatus) {
        if let Some(mut data) = self.map.get_mut(dict_name) {
            data.status = status;
        }
    }

    pub fn delete_entry(&self, dict_name: &str) -> Option<DictionaryStatus> {
        let entry = self.map.remove(dict_name);
        entry.map(|item| item.1.status)
    }

}