use anyhow::Result;
use config::{Config, Environment};
use deadpool_postgres::{Config as PoolConfig, Pool, Runtime};
use log::*;
use rustls::ClientConfig as RustlsClientConfig;
use serde::Deserialize;
use std::{fs::File, io::BufReader};
use tokio_postgres::NoTls;
use tokio_postgres_rustls::MakeRustlsConnect;

#[derive(Debug, Deserialize)]
struct Settings {
    pg: PoolConfig,
    db_ca_cert: Option<String>,
}

#[tokio::main]
async fn run(pool: Pool) -> Result<()> {
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

fn main() -> Result<()> {
    pretty_env_logger::init();

    let config = Config::builder()
        .add_source(Environment::default())
        .build()?;

    let settings: Settings = config.try_deserialize()?;

    debug!("settings: {:?}", settings);

    let pool = if let Some(ca_cert) = settings.db_ca_cert {
        let cert_file = File::open(ca_cert)?;
        let mut buf = BufReader::new(cert_file);
        let mut root_store = rustls::RootCertStore::empty();
        for cert in rustls_pemfile::certs(&mut buf) {
            root_store.add(cert?)?;
        }

        let tls_config = RustlsClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let tls = MakeRustlsConnect::new(tls_config);
        settings.pg.create_pool(Some(Runtime::Tokio1), tls)?
    } else {
        settings.pg.create_pool(Some(Runtime::Tokio1), NoTls)?
    };

    run(pool)
}
