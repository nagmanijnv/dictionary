use actix_web::{error, HttpResponse, http::{header::ContentType, StatusCode},};
use derive_more::Display;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::error::Error;
use serde_json::json;


#[derive(Deserialize)]
pub struct RandomWord {
    pub word: String,
    pub pronunciation: String,
    pub definition: String,
}

#[derive(Deserialize)]
pub struct DictGenerateRequest {
    pub dict_name: String,
    pub word_count: u32,
}

#[derive(Clone)]
pub enum DictionaryStatus {
    Completed,
    InProgress,
    Failed(String),
}

#[derive(Clone)]
pub struct DictionaryLocalState {
    pub status: DictionaryStatus,
    pub stats: Option<BTreeMap<char, usize>>
}

impl Default for DictionaryLocalState {
    fn default() -> Self {
        Self {
            status: DictionaryStatus::InProgress,
            stats: None
        }
    }
}

impl DictionaryLocalState {
    pub fn set_status(mut self, status: DictionaryStatus) -> Self{
        self.status = status;
        self
    }

    pub fn set_stats(mut self, stats: BTreeMap<char, usize>) -> Self {
        self.stats = Some(stats);
        self
    }
}

#[derive(Debug, Display,)]
pub enum DictionaryError {
    #[display("request failed, due to : {}", _0)]
    RemoteReqFailed(String),
    #[display("request failed, due to : {}", _0)]
    JoinError(String),
    #[display("request failed due to deserialisation issue")]
    FailedToDeserialise,
    #[display("request failed due internal issue")]
    FailedFileIO,
    #[display("A dictionary already exist with status: {}", _0)]
    EntryExist(String),
    #[display("Dictionary does not exist, reason: {}", _0)]
    NotFound(String)
}

impl Error for DictionaryError {}

impl error::ResponseError for DictionaryError {
    fn error_response(&self) -> HttpResponse {
        let msg = match self {
            DictionaryError::RemoteReqFailed(e) => DictionaryError::RemoteReqFailed(e.into()).to_string(),
            DictionaryError::JoinError(e) => DictionaryError::JoinError(e.into()).to_string(),
            DictionaryError::FailedToDeserialise => DictionaryError::FailedToDeserialise.to_string(),
            DictionaryError::FailedFileIO => DictionaryError::FailedFileIO.to_string(),
            DictionaryError::EntryExist(e) => DictionaryError::EntryExist(e.into()).to_string(),
            DictionaryError::NotFound(e) => DictionaryError::NotFound(e.into()).to_string(), 
        };

        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(json!({"message": msg, "error": true}))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            DictionaryError::RemoteReqFailed(_)
            | DictionaryError::JoinError(_)
            | DictionaryError::FailedToDeserialise
            | DictionaryError::FailedFileIO => StatusCode::INTERNAL_SERVER_ERROR,
            DictionaryError::EntryExist(_) => StatusCode::CONFLICT,
            DictionaryError::NotFound(_) => StatusCode::NOT_FOUND,
        }
    }
}

