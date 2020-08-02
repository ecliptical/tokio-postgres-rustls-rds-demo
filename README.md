# Connecting Securely to Amazon RDS for PostgreSQL

This project demonstrates how to use [Tokio Postgres](https://crates.io/crates/tokio-postgres) with [Rustls](https://crates.io/crates/rustls) to connect to [Amazon RDS for PostgreSQL](https://aws.amazon.com/rds/postgresql/) over TLS.

The trick? Configure your Rustls-backed Tokio Postgres client to use the AWS-issued RDS CA certificate, which can be downloaded [directly from Amazon](https://s3.amazonaws.com/rds-downloads/rds-ca-2019-root.pem). See [Using SSL/TLS to Encrypt a Connection to a DB Instance](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/UsingWithRDS.SSL.html) for additional details.

## RDS Setup

Ensure you have access to an _RDS for Postgres_ database. If not, you may be able to create one for free in the AWS Console.

### Parameter Group

You'll need to modify a database engine parameter, which means you need a custom (i.e., non-default) _Parameter Group_:

1. In RDS dashboard, _Parameter groups_ tab, click _Create parameter group_
2. Select _postgres12_ as the _Parameter group family_
3. Give it a _name_ (e.g., `secure`) and _description_
4. Click _Create_

### Database Instance

Now fow the actual database instance:

1. In the _Databases_ tab, click _Create database_
2. Choose _Standard Create_ and pick the _PostgreSQL_ engine type
3. Pick the latest _Version_ (e.g., PostgreSQL 12.3-R1)
4. If available, pick the _Free tier_ template
5. Fill out _Credentials Settings_ (i.e., Master username and password)
6. Under _Connectivity_, expand _Additional connectivity configuration_ and check _Yes_ under _Publicly accessible_. This will allow you to connect to your instance remotely from your computer.
7. Expand _Additional configuration_ and select the _DB parameter group_ that you previously created (e.g., `secure`)
8. Click _Create database_

### Security Group

Unless you previously set up and configured your Security Group, the default one that was created for your database instance won't let you connect remotely. To address this:

1. Open the details of your newly created database instance
2. In the _Connectivity and security_ tab, _Security_ section, click the first (active) security group under _VPC security groups_ (it should be named something like `default (sg-0123abcd)`).
3. Open its _Inbound rules_ tab and click _Edit inbound rules_
4. Ensure your laptop has access to TCP port 5432; e.g., add your public IP address as the _Source_ for a _PostgreSQL_ type rule. *BE CAUTIOUS* -- this has implications on the security of your newly created database instance and any other AWS assets that may be protected by this security group!

## Testing Connectivity

Once your new database instance becomes available, find its public hostname:

1. Open its details
2. In the _Connectivity and security_ tab, _Endpoint & port_ section, copy the _Endpoint_ value (i.e., its fully-qualified domain name). It should look something like `database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com`.

By default, the RDS instance you created allows both secure and insecure connections. To test that you can access the database without the use of TLS:

```bash
env PG.DBNAME=postgres PG.HOST=<your database hostname> PG.USER=postgres PG.PASSWORD=<your database password> RUST_LOG=debug cargo run
```

You should see output similar to:

```
    Finished dev [unoptimized + debuginfo] target(s) in 0.32s
     Running `target/debug/tokio-postgres-rustls-rds-demo`
 DEBUG tokio_postgres_rustls_rds_demo > settings: Settings { pg: Config { user: Some("postgres"), password: Some("xxxxxxxx"), dbname: Some("postgres"), options: None, application_name: None, ssl_mode: None, host: Some("database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com"), hosts: None, port: None, ports: None, connect_timeout: None, keepalives: None, keepalives_idle: None, target_session_attrs: None, channel_binding: None, manager: None, pool: None }, db_ca_cert: None }
 DEBUG tokio_postgres::prepare        > preparing query s0: SELECT * FROM information_schema.information_schema_catalog_name
 DEBUG tokio_postgres::query          > executing statement s0 with parameters: []
 INFO  tokio_postgres_rustls_rds_demo > postgres
```

## Securing the Connection

To connect using TLS, add the `DB_CA_CERT` parameter with the path to the RDS CA certificate:

```bash
env PG.DBNAME=postgres PG.HOST=<your database hostname> PG.USER=postgres PG.PASSWORD=<your database password> DB_CA_CERT=ca-certificates/rds-ca-2019-root.pem RUST_LOG=debug cargo run
```

You should see output similar to:

```
    Finished dev [unoptimized + debuginfo] target(s) in 0.44s
     Running `target/debug/tokio-postgres-rustls-rds-demo`
 DEBUG tokio_postgres_rustls_rds_demo > settings: Settings { pg: Config { user: Some("postgres"), password: Some("xxxxxxxx"), dbname: Some("postgres"), options: None, application_name: None, ssl_mode: None, host: Some("database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com"), hosts: None, port: None, ports: None, connect_timeout: None, keepalives: None, keepalives_idle: None, target_session_attrs: None, channel_binding: None, manager: None, pool: None }, db_ca_cert: Some("ca-certificates/rds-ca-2019-root.pem") }
 DEBUG rustls::anchors                > add_pem_file processed 1 valid and 0 invalid certs
 DEBUG rustls::client::hs             > No cached session for DNSNameRef("database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com")
 DEBUG rustls::client::hs             > Not resuming any session
 DEBUG rustls::client::hs             > ALPN protocol is None
 DEBUG rustls::client::hs             > Using ciphersuite TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
 DEBUG rustls::client::tls12          > ECDHE curve is ECParameters { curve_type: NamedCurve, named_group: secp256r1 }
 DEBUG rustls::client::tls12          > Got CertificateRequest CertificateRequestPayload { certtypes: [RSASign, DSSSign, ECDSASign], sigschemes: [RSA_PKCS1_SHA512, Unknown(1538), ECDSA_NISTP521_SHA512, RSA_PKCS1_SHA384, Unknown(1282), ECDSA_NISTP384_SHA384, RSA_PKCS1_SHA256, Unknown(1026), ECDSA_NISTP256_SHA256, Unknown(769), Unknown(770), Unknown(771), RSA_PKCS1_SHA1, Unknown(514), ECDSA_SHA1_Legacy], canames: [PayloadU16([48, 129, 151, 49, 11, 48, 9, 6, 3, 85, 4, 6, 19, 2, 85, 83, 49, 19, 48, 17, 6, 3, 85, 4, 8, 12, 10, 87, 97, 115, 104, 105, 110, 103, 116, 111, 110, 49, 16, 48, 14, 6, 3, 85, 4, 7, 12, 7, 83, 101, 97, 116, 116, 108, 101, 49, 34, 48, 32, 6, 3, 85, 4, 10, 12, 25, 65, 109, 97, 122, 111, 110, 32, 87, 101, 98, 32, 83, 101, 114, 118, 105, 99, 101, 115, 44, 32, 73, 110, 99, 46, 49, 19, 48, 17, 6, 3, 85, 4, 11, 12, 10, 65, 109, 97, 122, 111, 110, 32, 82, 68, 83, 49, 40, 48, 38, 6, 3, 85, 4, 3, 12, 31, 65, 109, 97, 122, 111, 110, 32, 82, 68, 83, 32, 99, 97, 45, 99, 101, 110, 116, 114, 97, 108, 45, 49, 32, 50, 48, 49, 57, 32, 67, 65])] }
 DEBUG rustls::client::tls12          > Client auth requested but no cert/sigscheme available
 DEBUG rustls::client::tls12          > Server cert is [Certificate(b"0\x82\x04\xe30\x82\x03\xcb\xa0\x03\x02\x01\x02\x02\x10\0\xc9\x0b^\x92\x04V\xa9\xd4#b*yh ;
 ...
 \xcc\xf5\xb8\tu\xef\x84\xb9\x84\xd3d\xc0\xf7\xf1\xde\x0b\r\xca\x10r0\x89\xc3n\x11\xfc")]
 DEBUG rustls::client::tls12          > Server DNS name is DNSName("database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com")
 DEBUG rustls::client::tls12          > Session not saved: server didn't allocate id or ticket
 DEBUG tokio_postgres::prepare        > preparing query s0: SELECT * FROM information_schema.information_schema_catalog_name
 DEBUG tokio_postgres::query          > executing statement s0 with parameters: []
 INFO  tokio_postgres_rustls_rds_demo > postgres
```

## Requiring Secure Connections

To further ensure that only TLS connections are allowed, change your _DB parameter group_ to require secure connections:

1. In the _Parameter groups_ tab, open the details of your custom parameter group (e.g., `secure`)
2. Type `rds.force_ssl` in the parameter name filter and click _Edit parameters_
2. Change the value of `rds.force_ssl` from `0` to `1` and click _Save changes_

Once the changes have been applied, you will no longer be able to connect without specifying the `DB_CA_CERT` argument:

```bash
env PG.DBNAME=postgres PG.HOST=<your database hostname> PG.USER=postgres PG.PASSWORD=<your database password> RUST_LOG=debug cargo run
```

Output:

```
    Finished dev [unoptimized + debuginfo] target(s) in 0.79s
     Running `target/debug/tokio-postgres-rustls-rds-demo`
 DEBUG tokio_postgres_rustls_rds_demo > settings: Settings { pg: Config { user: Some("postgres"), password: Some("xxxxxxxx"), dbname: Some("postgres"), options: None, application_name: None, ssl_mode: None, host: Some("database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com"), hosts: None, port: None, ports: None, connect_timeout: None, keepalives: None, keepalives_idle: None, target_session_attrs: None, channel_binding: None, manager: None, pool: None }, db_ca_cert: None }
Error: Backend(Error { kind: Connect, cause: Some(Os { code: 2, kind: NotFound, message: "No such file or directory" }) })
```

## Connecting Without the RDS CA Certificate

Using TLS without Amazon's RDS CA certificate, you would see the following error:

```
    Finished dev [unoptimized + debuginfo] target(s) in 10.68s
     Running `target/debug/tokio-postgres-rustls-rds-demo`
 DEBUG tokio_postgres_rustls_rds_demo > settings: Settings { pg: Config { user: Some("postgres"), password: Some("xxxxxxxx"), dbname: Some("postgres"), options: None, application_name: None, ssl_mode: None, host: Some("database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com"), hosts: None, port: None, ports: None, connect_timeout: None, keepalives: None, keepalives_idle: None, target_session_attrs: None, channel_binding: None, manager: None, pool: None }, db_ca_cert: Some("ca-certificates/rds-ca-2019-root.pem") }
 DEBUG rustls::client::hs             > No cached session for DNSNameRef("database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com")
 DEBUG rustls::client::hs             > Not resuming any session
 DEBUG rustls::client::hs             > ALPN protocol is None
 DEBUG rustls::client::hs             > Using ciphersuite TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
 DEBUG rustls::client::tls12          > ECDHE curve is ECParameters { curve_type: NamedCurve, named_group: secp256r1 }
 DEBUG rustls::client::tls12          > Got CertificateRequest CertificateRequestPayload { certtypes: [RSASign, DSSSign, ECDSASign], sigschemes: [RSA_PKCS1_SHA512, Unknown(1538), ECDSA_NISTP521_SHA512, RSA_PKCS1_SHA384, Unknown(1282), ECDSA_NISTP384_SHA384, RSA_PKCS1_SHA256, Unknown(1026), ECDSA_NISTP256_SHA256, Unknown(769), Unknown(770), Unknown(771), RSA_PKCS1_SHA1, Unknown(514), ECDSA_SHA1_Legacy], canames: [PayloadU16([48, 129, 151, 49, 11, 48, 9, 6, 3, 85, 4, 6, 19, 2, 85, 83, 49, 19, 48, 17, 6, 3, 85, 4, 8, 12, 10, 87, 97, 115, 104, 105, 110, 103, 116, 111, 110, 49, 16, 48, 14, 6, 3, 85, 4, 7, 12, 7, 83, 101, 97, 116, 116, 108, 101, 49, 34, 48, 32, 6, 3, 85, 4, 10, 12, 25, 65, 109, 97, 122, 111, 110, 32, 87, 101, 98, 32, 83, 101, 114, 118, 105, 99, 101, 115, 44, 32, 73, 110, 99, 46, 49, 19, 48, 17, 6, 3, 85, 4, 11, 12, 10, 65, 109, 97, 122, 111, 110, 32, 82, 68, 83, 49, 40, 48, 38, 6, 3, 85, 4, 3, 12, 31, 65, 109, 97, 122, 111, 110, 32, 82, 68, 83, 32, 99, 97, 45, 99, 101, 110, 116, 114, 97, 108, 45, 49, 32, 50, 48, 49, 57, 32, 67, 65])] }
 DEBUG rustls::client::tls12          > Client auth requested but no cert/sigscheme available
 DEBUG rustls::client::tls12          > Server cert is [Certificate(b"0\x82\x04\xe30\x82\x03\xcb\xa0\x03\x02\x01\x02\x02\x10\0\xc9\x0b^\x92\x04V\xa9\xd4#b*yh ;
 ...
 \xcc\xf5\xb8\tu\xef\x84\xb9\x84\xd3d\xc0\xf7\xf1\xde\x0b\r\xca\x10r0\x89\xc3n\x11\xfc")]
 DEBUG rustls::client::tls12          > Server DNS name is DNSName("database-1.xq7f5vzbpq1x.ca-central-1.rds.amazonaws.com")
 WARN  rustls::session                > Sending fatal alert BadCertificate
Error: Backend(Error { kind: Tls, cause: Some(Kind(InvalidInput)) })
```
