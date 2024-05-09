use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub app: AppSettings,
}

#[derive(serde::Deserialize)]
pub struct AppSettings {
    pub port: u16,
    pub host: String,
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. \
                Use either `local` or `production`.",
                other
            )),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

// Get Configuration
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // we have `base` `local` and `production`
    // base holds the `port` and database settings
    // `local` and `production` hold different hosts
    // Introduce an `APP_ENVIRONMENT` env to toggle the hosts

    // Get the projects root path
    let base_path = std::env::current_dir().expect("Failed to get current directory path");

    // Get the `/configuration` directory path
    let configuration_directory = base_path.join("configuration");

    // Get the environment
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse `APP_ENVIRONMENT`");
    let environment_file_name = format!("{}.yaml", environment.as_str());

    // Based on the environment we should load the correct file.
    // Initialize configuration reader
    let settings = config::Config::builder()
        // Add base config file from `base.yaml`
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        // Add environment config file from `(local or production).yaml`
        .add_source(config::File::from(
            configuration_directory.join(environment_file_name),
        ))
        .build()?;

    // Try to convert the configuration file into a `Settings` struct instance
    settings.try_deserialize::<Settings>()
}
