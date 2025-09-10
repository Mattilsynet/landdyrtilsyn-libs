//! Konfigurasjon og oppsett av databaseforbindelse (connection pool).
//!
//! Leser nødvendige miljøvariabler og oppretter en gjenbruksbar `sqlx::Pool<Postgres)`
//! som kan injiseres i andre deler av systemet.

use secrecy::ExposeSecret;
use secrecy::SecretString;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Pool, Postgres};
use std::env;

/// Typealias for prosjektets database-pool.
///
/// Bruk denne for å dele pool mellom kall/funksjoner i biblioteket og
/// nedstrøms applikasjoner.
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

/// Oppretter og returnerer en databaseforbindelses-pool.
///
/// Leser nødvendige miljøvariabler:
/// - `DATABASE_USER`
/// - `DATABASE_PASSWORD`
/// - `DATABASE_HOST`
/// - `DATABASE_PORT`
/// - `DATABASE_NAME`
///
/// # Panics
/// Denne funksjonen kan panikke dersom en eller flere av miljøvariablene over
/// ikke er satt. Det skyldes eksplisitte `expect(...)`-kall ved uthenting av
/// variablene.
///
/// # Errors
/// Returnerer `sqlx::Error` dersom opprettelse av pool feiler, for eksempel ved
/// ugyldig tilkoblingsstreng, nettverksfeil eller avvist tilkobling fra databasen.
///
/// # Examples
/// ```rust
/// # async fn example() -> Result<(), sqlx::Error> {
/// use lib_sql::database_config::get_database_pool;
///
/// let pool = get_database_pool().await?;
/// // Bruk `pool`
/// # Ok(())
/// # }
/// ```
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
            database_connection_name: String::new(),
        };
        settings.local_connection_string()
    } else {
        let database_connection_name =
            env::var("DATABASE_CONNECTION_NAME").expect("INSTANCE_CONNECTION_NAME must be set");
        let settings = DatabaseSettings {
            username,
            password: password.into(),
            host: String::new(),
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
