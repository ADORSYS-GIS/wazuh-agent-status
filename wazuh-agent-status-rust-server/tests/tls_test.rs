#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::net::TcpStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use wazuh_agent_status_rust_server::tls;
    use wazuh_agent_status_rust_server::secret_store::FileSecretStore;
    use tokio_rustls::{TlsConnector, TlsAcceptor};
    use std::path::PathBuf;
    use tokio_rustls::rustls;

    fn get_fixture_path(name: &str) -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("tests/resources/certs");
        p.push(name);
        p
    }

    async fn build_test_client_connector(ca: &str, cert: &str, key: &str) -> anyhow::Result<TlsConnector> {
        let ca_certs = tls::load_certs(&get_fixture_path(ca))?;
        let client_certs = tls::load_certs(&get_fixture_path(cert))?;
        let client_key = tls::load_private_key(&get_fixture_path(key))?;

        let mut root_store = rustls::RootCertStore::empty();
        for c in ca_certs {
            root_store.add(c)?;
        }

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(client_certs, client_key)?;

        Ok(TlsConnector::from(Arc::new(config)))
    }

    async fn run_mock_server(acceptor: TlsAcceptor, listener: tokio::net::TcpListener) {
        if let Ok((socket, _)) = listener.accept().await {
            if let Ok(mut tls_stream) = acceptor.accept(socket).await {
                let mut buf = [0u8; 4];
                if tls_stream.read_exact(&mut buf).await.is_ok() {
                    let _ = tls_stream.write_all(b"pong").await;
                }
            }
        }
    }

    #[tokio::test]
    async fn test_mtls_successful_handshake() -> anyhow::Result<()> {
        let store = FileSecretStore::new(
            get_fixture_path("ca.pem"),
            get_fixture_path("server.pem"),
            get_fixture_path("server.key"),
        );

        let acceptor = tls::build_mtls_acceptor(&store)?;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        tokio::spawn(run_mock_server(acceptor, listener));

        let connector = build_test_client_connector("ca.pem", "client.pem", "client.key").await?;
        let stream = TcpStream::connect(addr).await?;
        let server_name = rustls::pki_types::ServerName::try_from("localhost")?.to_owned();

        let mut tls_stream = connector.connect(server_name, stream).await?;
        tls_stream.write_all(b"ping").await?;
        let mut buf = [0u8; 4];
        tls_stream.read_exact(&mut buf).await?;
        assert_eq!(&buf, b"pong");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_mtls_rejection_no_cert() -> anyhow::Result<()> {
        let store = FileSecretStore::new(
            get_fixture_path("ca.pem"),
            get_fixture_path("server.pem"),
            get_fixture_path("server.key"),
        );

        let acceptor = tls::build_mtls_acceptor(&store)?;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        tokio::spawn(run_mock_server(acceptor, listener));

        // Client without client certificate
        let ca_certs = tls::load_certs(&get_fixture_path("ca.pem"))?;
        let mut root_store = rustls::RootCertStore::empty();
        for c in ca_certs {
            root_store.add(c)?;
        }
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(config));

        let stream = TcpStream::connect(addr).await?;
        let server_name = rustls::pki_types::ServerName::try_from("localhost")?.to_owned();

        let mut tls_stream = connector.connect(server_name, stream).await?;
        
        let _ = tls_stream.write_all(b"ping").await;
        let mut buf = [0u8; 4];
        let res = tls_stream.read_exact(&mut buf).await;
        assert!(res.is_err());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_mtls_rejection_untrusted_cert() -> anyhow::Result<()> {
        let store = FileSecretStore::new(
            get_fixture_path("ca.pem"),
            get_fixture_path("server.pem"),
            get_fixture_path("server.key"),
        );

        let acceptor = tls::build_mtls_acceptor(&store)?;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        tokio::spawn(run_mock_server(acceptor, listener));

        // Client with untrusted cert
        let ca_certs = tls::load_certs(&get_fixture_path("ca.pem"))?;
        let mut root_store = rustls::RootCertStore::empty();
        for c in ca_certs {
            root_store.add(c)?;
        }
        
        let client_certs = tls::load_certs(&get_fixture_path("untrusted/client.pem"))?;
        let client_key = tls::load_private_key(&get_fixture_path("untrusted/client.key"))?;

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(client_certs, client_key)?;
        let connector = TlsConnector::from(Arc::new(config));

        let server_name = rustls::pki_types::ServerName::try_from("localhost")?.to_owned();
        let stream = TcpStream::connect(addr).await?;
        
        let res = connector.connect(server_name, stream).await;
        if let Ok(mut tls_stream) = res {
             let _ = tls_stream.write_all(b"ping").await;
             let mut buf = [0u8; 4];
             assert!(tls_stream.read_exact(&mut buf).await.is_err());
        }

        Ok(())
    }
}
