use std::sync::Arc;

use redis::Client;
use serde::{de::DeserializeOwned, Serialize};

use crate::{models::custom_api_errors::Result, utils::envs};

#[derive(Clone, Debug)]
pub struct Cache {
    pub client: Arc<Client>,
}

impl Cache {
    pub fn init() -> Self {
        let client = Client::open(envs::redis_url()).expect("Could not open Redis client");

        Self {
            client: Arc::new(client),
        }
    }

    async fn connection(&self) -> Result<redis::aio::Connection> {
        let conn = self.client.get_async_connection().await?;

        Ok(conn)
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query_async(&mut self.connection().await?)
            .await?;

        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<String> {
        let value: String = redis::cmd("GET")
            .arg(key)
            .query_async(&mut self.connection().await?)
            .await?;

        Ok(value)
    }

    pub async fn set_json<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let value = serde_json::to_string(value)?;

        redis::cmd("JSON.SET")
            .arg(key)
            .arg(value)
            .query_async(&mut self.connection().await?)
            .await?;

        Ok(())
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let value: String = redis::cmd("JSON.GET")
            .arg(key)
            .query_async(&mut self.connection().await?)
            .await?;

        let value: T = serde_json::from_str(&value)?;

        Ok(value)
    }

    pub async fn push_strings_to_set(&self, key: &str, values: Vec<&str>) -> Result<()> {
        redis::cmd("SADD")
            .arg(key)
            .arg(values)
            .query_async(&mut self.connection().await?)
            .await?;

        Ok(())
    }

    pub async fn remove_string_from_set(&self, key: &str, value: &str) -> Result<()> {
        redis::cmd("SREM")
            .arg(key)
            .arg(value)
            .query_async(&mut self.connection().await?)
            .await?;

        Ok(())
    }

    pub async fn get_string_set(&self, key: &str) -> Result<Vec<String>> {
        let values: Vec<String> = redis::cmd("SMEMBERS")
            .arg(key)
            .query_async(&mut self.connection().await?)
            .await?;

        Ok(values)
    }

    pub async fn key_exists(&self, key: &str) -> Result<bool> {
        let exists: bool = redis::cmd("EXISTS")
            .arg(key)
            .query_async(&mut self.connection().await.unwrap())
            .await?;

        Ok(exists)
    }

    pub fn session_users_key(session_id: &str) -> String {
        format!("session:{}:users", session_id)
    }
}
