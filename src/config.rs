use serde::Deserialize;
use serde_json::from_slice;
use async_std::fs::read;
use crate::prelude::*;
use crate::tweets::TwitterAuth;

#[derive(Deserialize)]
pub struct Config {
	pub keywords: Vec<String>,
	pub auth: TwitterAuth,
}

pub async fn read_config() -> Result<Config> {
	let data = read("config.json").await?;
	Ok(from_slice(&data)?)
}