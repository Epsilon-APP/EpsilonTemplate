pub struct Config {
    pub registry_username: String,
    pub registry_password: String,
    pub registry_host: String,

    pub api_host: String,
}

impl Config {
    pub fn new(
        default_registry_username: &str,
        default_registry_password: &str,
        default_registry_host: &str,
        default_api_host: &str,
    ) -> Config {
        let registry_username = std::env::var("REGISTRY_USERNAME")
            .unwrap_or_else(|_| default_registry_username.to_string());
        let registry_password = std::env::var("REGISTRY_PASSWORD")
            .unwrap_or_else(|_| default_registry_password.to_string());
        let registry_host =
            std::env::var("REGISTRY_HOST").unwrap_or_else(|_| default_registry_host.to_string());
        let api_host = std::env::var("API_HOST").unwrap_or_else(|_| default_api_host.to_string());

        Config {
            registry_username,
            registry_password,
            registry_host,
            api_host,
        }
    }
}
