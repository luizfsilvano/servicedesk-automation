mod config;
mod handlers;

#[tokio::main]
async fn main() {
    let base_path = std::env::current_dir().expect("Falha ao obter  diretório atual");
    let config_path = base_path.join("Data").join("Configs").join("appsettings.json");
    let config_path_str = config_path.to_str().expect("Caminho de configuração inválido");

    match config::load(config_path_str) {
        Ok(settings) => {
            println!("Configurações carregadas com sucesso:");
            println!("Ambiente: {}", settings.environment);
            println!("ServiceDesk URL: {:?}", settings.service_desk.production_url);
            println!("TopDesk Base URL: {:?}", settings.top_desk.base_url);

            let mut auth_handler = handlers::auth_handler::AuthHandler::new(settings.clone());

            // Tentativa do login no SD
            match auth_handler.login_service_desk().await {
                Ok(_) => {
                    println!("Login no Service Desk realizado com sucesso!");
                    println!("User Name: {}", auth_handler.user_name);
                    println!("User Email: {}", auth_handler.user_email);
                    println!("User Group ID: {}", auth_handler.user_group_id);
                    println!("goc_session: {}", auth_handler.goc_session);
                    println!("session_id: {}", auth_handler.session_id);
                }
                Err(e) => {
                    match e {
                        handlers::auth_handler::AuthError::ManualLoginRequired(details) => {
                            eprintln!("ERRO CRÍTICO DE LOGIN NO SERVICE DESK:");
                            eprintln!("Parece que o Service Desk exige um login manual inicial no navegador para liberar a sessão da API.");
                            eprintln!("Por favor, acesse o Service Desk no navegador, faça login e tente novamente.");
                            eprintln!("Detalhes da API: {}", details);
                        },
                        _ => { // Captura todos os outros tipos de AuthError
                            eprintln!("Erro ao fazer login no Service Desk: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Erro ao carregar configurações: {}", e);
        }
    }
}
