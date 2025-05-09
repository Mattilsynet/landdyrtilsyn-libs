use secrecy::ExposeSecret;
use secrecy::SecretString;
use sqlx::{
    Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::env;

pub type DbPool = Pool<Postgres>;

struct DatabaseSettings {
    username: String,
    password: SecretString,
    host: String,
    port: u16,
    database_name: String,
    database_connection_name: String,
}

impl DatabaseSettings {
    fn local_connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )
    }
    fn cloud_connection_string(&self) -> String {
        format!(
            "postgres://?host=/cloudsql/{host}&port={port}&dbname={db_name}&user={username}&password={password}",
            host = self.database_connection_name,
            port = self.port,
            db_name = self.database_name,
            username = self.username,
            password = self.password.expose_secret()
        )
    }
}

pub async fn get_database_pool() -> Result<DbPool, sqlx::Error> {
    let env = env::var("APP_APPLICATION__ENVIRONMENT").unwrap_or_else(|_| "local".into());
    let username = env::var("DATABASE_USER").expect("DATABASE_USER must be set");
    let password = env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD must be set");
    let database_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");
    let connection_string = if env == "local" {
        let host = env::var("DATABASE_HOST").expect("DATABASE_HOST must be set");
        let port = env::var("DATABASE_PORT")
            .expect("DATABASE_PORT must be set")
            .parse::<u16>()
            .expect("DATABASE_PORT must be a number");
        let settings = DatabaseSettings {
            username,
            password: password.into(),
            host,
            port,
            database_name,
            database_connection_name: "".into(),
        };
        settings.local_connection_string()
    } else {
        let database_connection_name =
            env::var("DATABASE_CONNECTION_NAME").expect("INSTANCE_CONNECTION_NAME must be set");
        let settings = DatabaseSettings {
            username,
            password: password.into(),
            host: "".into(),
            port: 5432,
            database_name,
            database_connection_name,
        };
        settings.cloud_connection_string()
    };
    if env == "local" {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await
    } else {
        let opt = connection_string.parse::<PgConnectOptions>()?;
        Ok(PgPoolOptions::new()
            .max_connections(5)
            .connect_with(opt)
            .await
            .expect("Failed to connect to Postgres"))
    }
}
