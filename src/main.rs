mod config;

fn main() {
    let base_path = std::env::current_dir().expect("Falha ao obter  diretório atual");
    let config_path = base_path.join("Data").join("Configs").join("appsettings.json");
    let config_path_str = config_path.to_str().expect("Caminho de configuração inválido");

    match config::load(config_path_str) {
        Ok(settings) => {
            println!("Configurações carregadas com sucesso:");
            println!("Ambiente: {}", settings.environment);
            println!("ServiceDesk URL: {:?}", settings.service_desk.production_url);
            println!("TopDesk Base URL: {:?}", settings.top_desk.base_url);
        }
        Err(e) => {
            eprintln!("Erro ao carregar configurações: {}", e);
        }
    }
}
