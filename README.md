# Connecting Securely to Amazon RDS for PostgreSQL

This project demonstrates how to use [Tokio Postgres](https://crates.io/crates/tokio-postgres) with [Rustls](https://crates.io/crates/rustls) to connect to [Amazon RDS for PostgreSQL](https://aws.amazon.com/rds/postgresql/) over TLS.

The trick? Configure your Rustls-backed Tokio Postgres client to use the AWS-issued RDS CA certificate, which can be downloaded [directly from Amazon](https://truststore.pki.rds.amazonaws.com/global/global-bundle.pem). See [Using SSL/TLS to Encrypt a Connection to a DB Instance](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/UsingWithRDS.SSL.html) for additional details.

## RDS Setup

Ensure you have access to an _RDS for Postgres_ database. If not, you may be able to create one for free in the AWS Console.

### Database Instance

Now for the actual database instance:

1. In the _Databases_ tab, click _Create database_
2. Choose _Standard Create_ and pick the _PostgreSQL_ engine type
3. Pick the latest _Version_ (e.g., PostgreSQL 16.1-R2)
4. If available, pick the _Free tier_ template
5. Fill out _Credentials Settings_ (i.e., Master username and password)
6. Under _Connectivity_, check _Yes_ under _Public access_. This will allow you to connect to your instance remotely from your computer.
7. Scroll down to the last top-level section named _Additional configuration_ and enter `postgres` in the _Initial database name_ field right under _Database options_.
8. Click _Create database_

### Security Group

Unless you previously set up and configured your Security Group, the default one that was created for your database instance won't let you connect remotely. To address this:

1. Open the details of your newly created database instance
2. In the _Connectivity and security_ tab, _Security_ section, click the first (active) security group under _VPC security groups_ (it should be named something like `default (sg-0123abcd)`).
3. Open its _Inbound rules_ tab and click _Edit inbound rules_
4. Ensure your development machine has access to TCP port 5432; e.g., add your public IP address as the _Source_ for a _PostgreSQL_ type rule. *BE CAUTIOUS* -- this has implications on the security of your newly created database instance and any other AWS assets that may be protected by this security group!

## Testing Connectivity

Once your new database instance becomes available, find its public hostname:

1. Open its details
2. In the _Connectivity and security_ tab, _Endpoint & port_ section, copy the _Endpoint_ value (i.e., its fully-qualified domain name). It should look something like `database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com`.

> In the examples that follow, please substitute the PG.HOST parameter value with your own database instance hostname.

By default, the RDS instance you created requires secure connections (i.e., its default parameter group's `rds.force_ssl` parameter is set to `1`). To connect using TLS, add the `DB_CA_CERT` parameter with the path to the RDS CA certificate:

```bash
env PG.DBNAME=postgres PG.HOST=database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com PG.USER=postgres PG.PASSWORD=xxxxxxxx DB_CA_CERT=ca-certificates/global-bundle.pem RUST_LOG=debug cargo run
```

You should see output similar to:

```
    Finished dev [unoptimized + debuginfo] target(s) in 0.08s
     Running `target/debug/tokio-postgres-rustls-rds-demo`
 DEBUG tokio_postgres_rustls_rds_demo > settings: Settings { pg: Config { url: None, user: Some("postgres"), password: Some("xxxxxxxx"), dbname: Some("postgres"), options: None, application_name: None, ssl_mode: None, host: Some("database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com"), hosts: None, hostaddr: None, hostaddrs: None, port: None, ports: None, connect_timeout: None, keepalives: None, keepalives_idle: None, target_session_attrs: None, channel_binding: None, load_balance_hosts: None, manager: None, pool: None }, db_ca_cert: Some("ca-certificates/global-bundle.pem") }
 DEBUG rustls::client::hs             > No cached session for DnsName("database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com")
 DEBUG rustls::client::hs             > Not resuming any session
 DEBUG rustls::client::hs             > Using ciphersuite TLS13_AES_256_GCM_SHA384
 DEBUG rustls::client::tls13          > Not resuming
 DEBUG rustls::client::tls13          > TLS1.3 encrypted extensions: []
 DEBUG rustls::client::hs             > ALPN protocol is None
 DEBUG rustls::client::tls13          > Got CertificateRequest CertificateRequestPayloadTls13 { context: , extensions: [SignatureAlgorithms([ECDSA_NISTP256_SHA256, RSA_PSS_SHA256, RSA_PKCS1_SHA256, ECDSA_NISTP384_SHA384, RSA_PSS_SHA384, RSA_PKCS1_SHA384, RSA_PSS_SHA512, RSA_PKCS1_SHA512, RSA_PKCS1_SHA1]), AuthorityNames([DistinguishedName(3081a5310b300906035504061302555331223020060355040a0c19416d617a6f6e205765622053657276696365732c20496e632e31133011060355040b0c0a416d617a6f6e20524453310b300906035504080c025741313e303c06035504030c35416d617a6f6e205244532063612d63656e7472616c2d31205375626f7264696e61746520434120525341323034382047312e412e353110300e06035504070c0753656174746c65)])] }
 DEBUG rustls::client::common         > Client auth requested but no cert/sigscheme available
 DEBUG tokio_postgres::prepare        > preparing query s0: SELECT * FROM information_schema.information_schema_catalog_name
 DEBUG tokio_postgres::query          > executing statement s0 with parameters: []
 INFO  tokio_postgres_rustls_rds_demo > postgres```

## Connecting Without the RDS CA Certificate

Using TLS without Amazon's RDS CA certificate, you would see an error similar to:

```
    Finished dev [unoptimized + debuginfo] target(s) in 1.28s
     Running `target/debug/tokio-postgres-rustls-rds-demo`
 DEBUG tokio_postgres_rustls_rds_demo > settings: Settings { pg: Config { url: None, user: Some("postgres"), password: Some("xxxxxxxx"), dbname: Some("postgres"), options: None, application_name: None, ssl_mode: None, host: Some("database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com"), hosts: None, hostaddr: None, hostaddrs: None, port: None, ports: None, connect_timeout: None, keepalives: None, keepalives_idle: None, target_session_attrs: None, channel_binding: None, load_balance_hosts: None, manager: None, pool: None }, db_ca_cert: None }
Error: Error occurred while creating a new object: db error: FATAL: no pg_hba.conf entry for host "12.34.56.78", user "postgres", database "postgres", no encryption

Caused by:
    0: db error: FATAL: no pg_hba.conf entry for host "12.34.56.78", user "postgres", database "postgres", no encryption
    1: FATAL: no pg_hba.conf entry for host "12.34.56.78", user "postgres", database "postgres", no encryption
```
