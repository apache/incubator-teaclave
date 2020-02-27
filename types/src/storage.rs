use anyhow;
use serde::{Deserialize, Serialize};
use serde_json;
use std::prelude::v1::*;
use uuid::Uuid;

pub trait Storable: Serialize + for<'de> Deserialize<'de> {
    fn key_prefix() -> &'static str;

    fn uuid(&self) -> Uuid;

    fn key_string(&self) -> String {
        format!("{}-{}", Self::key_prefix(), self.uuid().to_string())
    }

    fn key(&self) -> Vec<u8> {
        self.key_string().into_bytes()
    }

    fn match_prefix(key: &str) -> bool {
        key.starts_with(Self::key_prefix())
    }

    fn to_vec(&self) -> anyhow::Result<Vec<u8>> {
        let bytes = serde_json::to_vec(self)?;
        Ok(bytes)
    }

    fn from_slice(bytes: &[u8]) -> anyhow::Result<Self> {
        let obj = serde_json::from_slice(bytes)?;
        Ok(obj)
    }

    fn external_id(&self) -> String {
        self.key_string()
    }
}
