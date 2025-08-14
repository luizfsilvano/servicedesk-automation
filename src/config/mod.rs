use serde::Deserialize;
use std::io;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub environment: String,
    #[serde(rename = "ServiceDesk")]
    pub service_desk: ServiceDeskConfig,
    #[serde(rename = "TopDesk")]
    pub top_desk: TopDeskConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServiceDeskConfig {
    #[serde(rename = "SandboxUrl")]
    pub sandbox_url: String,
    #[serde(rename = "ProductionUrl")]
    pub production_url: String,
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(rename = "Password")]
    pub password: String,
    #[serde(rename = "UserID")]
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TopDeskConfig {
    #[serde(rename = "BaseUrl")]
    pub base_url: String,
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(rename = "Password")]
    pub password: String,
}

type Result<T> = std::result::Result<T, io::Error>;

pub fn load(path: &str) -> Result<AppSettings> {
    println!("Carregando configurações de: {}", path);
    let json_content = fs::read_to_string(path)?;
    let settings: AppSettings = serde_json::from_str(&json_content)?;
    println!("Configurações carregadas com sucesso.");
    Ok(settings)
}

