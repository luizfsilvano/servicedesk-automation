use serde::{Serialize, Deserialize};
use reqwest::{Client, cookie::Jar, Url};
use std::sync::Arc;
use thirtyfour::SessionId;
use crate::config::AppSettings;
use thiserror::Error;

#[derive(Serialize)]
pub struct LoginPayload {
    user_name: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct UserInfoEntry {
    pub key: String,
    pub value: serde_json::Value,
    #[serde(rename = "valueCaption")]
    pub value_caption: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub info: Vec<UserInfoEntry>,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub user: User,
}

#[derive(Debug, Error)]
pub enum AuthError {

    #[error("Falha na autenticação: {0}")]
    AuthenticationFailed(String), // Erros genéricos de falha de autenticação

    #[error("Falha ao fazer login. Por favor, faça login manualmente no navegador para inicializar a sessão. Detalhes: {0}")]
    ManualLoginRequired(String),

    #[error("Erro de requisição HTTP: {0}")]
    ReqwestError(#[from] reqwest::Error), // Erros do reqwest (convertidos automaticamente)

    #[error("Erro de IO: {0}")]
    IoError(#[from] std::io::Error), // Erros de I/O (se a leitura do corpo da resposta falhar, por exemplo)

    #[error("Erro de parseamento JSON: {0}")]
    JsonParseError(#[from] serde_json::Error), // Erros de desserialização JSON

    #[error("Campo de informação de usuário não encontrado ou inválido: {0}")]
    UserInfoMissing(String), // Se user_group_id, first_name, email não forem encontrados
}
// Gerenciar autenticação com o SD
pub struct AuthHandler {
    config: AppSettings,
    http_client: Client,
    pub session_id: String,
    pub goc_session: String,
    pub user_group_id: u32,
    pub user_name: String,
    pub user_email: String,
    base_url: String,
}

impl AuthHandler {
    pub fn new(config: AppSettings) -> Self {
        let base_url = if config.environment == "Sandbox" {
            config.service_desk.sandbox_url.clone()
        } else {
            config.service_desk.production_url.clone()
        };

        let cookie_jar = Arc::new(Jar::default());
        let http_client = Client::builder()
            .cookie_store(true)
            .cookie_provider(cookie_jar)
            .build()
            .expect("Falha ao construir o cliente HTTP");

        AuthHandler {
            config,
            http_client,
            session_id: String::new(),
            goc_session: String::new(),
            user_group_id: 0,
            user_name: String::new(),
            user_email: String::new(),
            base_url,
        }
    }

    // Métódo pra fazer lógin assíncrono
    pub async fn login_service_desk(&mut self) -> Result<(), AuthError> {
        let url = format!("{}/api/v1/login?user", self.base_url);
        let payload = LoginPayload {
            user_name: self.config.service_desk.username.clone(),
            password: self.config.service_desk.password.clone(),
        };

        println!("Tentando autenticar no Service Desk com a URL: {}", url);

        let response = self.http_client.post(&url)
            .json(&payload)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let error_details = response.text().await?;
            let lowercased_details = error_details.to_lowercase();

            // Verifica a mensagem de erro específica
            if lowercased_details.contains("informações de acesso incorretas.") &&
                lowercased_details.contains("as palavras devem ser escritas na caixa correta.") &&
                lowercased_details.contains("certifique-se de que a tecla caps lock não esteja ligada.") {
                return Err(AuthError::ManualLoginRequired(error_details));
            } else {
                return Err(AuthError::AuthenticationFailed(error_details));
            }
        }

        let json_response: LoginResponse = response.json().await?;

        let info_map: std::collections::HashMap<String, serde_json::Value> = json_response.user.info.into_iter()
            .map(|entry| (entry.key, entry.value))
            .collect();

        if let Some(user_groups_val) = info_map.get("user_groups") {
            if let Some(user_groups_array) = user_groups_val.as_array() {
                if let Some(first_group) = user_groups_array.get(0) {
                    if let Some(id) = first_group.get("id") {
                        self.user_group_id = id.as_u64().unwrap_or(0) as u32;
                    }
                }
            }
        }
        if self.user_group_id == 0 {
            println!("Nenhum grupo de atendimento encontrado no retorno.");
            return Err(AuthError::UserInfoMissing("Falha ao capturar o ID do Grupo de Atendimento.".to_string()));
        }

        self.user_name = info_map.get("first_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Nome não encontrado")
            .to_string();

        self.user_email = info_map.get("email_address")
            .and_then(|v| v.as_str())
            .unwrap_or("E-mail não encontrado")
            .to_string();

        println!("Login realizado com sucesso!");
        Ok(())
    }
}