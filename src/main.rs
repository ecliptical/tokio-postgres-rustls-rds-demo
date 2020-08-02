use config::{
    Config,
    Environment,
};

use deadpool_postgres::{
    Config as PoolConfig,
    Pool,
};

use log::*;
use rustls::ClientConfig as RustlsClientConfig;
use serde::Deserialize;
use std::{
    fs::File,
    io::BufReader,
};

use tokio_postgres::NoTls;
use tokio_postgres_rustls::MakeRustlsConnect;

#[derive(Debug, Deserialize)]
struct Settings {
    pg: PoolConfig,
    db_ca_cert: Option<String>,
}

#[tokio::main]
async fn run(pool: Pool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let stmt = client
        .prepare("SELECT * FROM information_schema.information_schema_catalog_name")
        .await?;

    let rows = client.query(&stmt, &[]).await?;
    for row in rows {
        let val: String = row.try_get(0)?;
        info!("{}", val);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let mut config = Config::new();
    config.merge(Environment::new())?;

    let settings: Settings = config.try_into()?;

    debug!("settings: {:?}", settings);

    let pool = if let Some(ca_cert) = settings.db_ca_cert {
        let mut tls_config = RustlsClientConfig::new();
        let cert_file = File::open(&ca_cert)?;
        let mut buf = BufReader::new(cert_file);
        tls_config.root_store.add_pem_file(&mut buf).map_err(|_| {
            anyhow::anyhow!("failed to read database root certificate: {}", ca_cert)
        })?;

        let tls = MakeRustlsConnect::new(tls_config);
        settings.pg.create_pool(tls)?
    } else {
        settings.pg.create_pool(NoTls)?
    };

    run(pool)
}
