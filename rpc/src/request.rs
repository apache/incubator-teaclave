use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::prelude::v1::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request<T> {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(flatten)]
    pub message: T,
}

impl<T> Request<T> {
    pub fn new(message: T) -> Self {
        Request {
            metadata: HashMap::<String, String>::default(),
            message,
        }
    }
}

impl<T> Request<T> {
    pub fn map<F, U>(self, f: F) -> Request<U>
    where
        F: FnOnce(T) -> U,
    {
        let message = f(self.message);

        Request {
            metadata: self.metadata,
            message,
        }
    }
}
