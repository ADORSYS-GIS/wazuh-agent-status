use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use rustls_pemfile::{certs, private_key};
use tokio_rustls::rustls::{self, pki_types::{CertificateDer, PrivateKeyDer}};
use tokio_rustls::TlsAcceptor;

/// Load certificates from a PEM file.
pub fn load_certs(path: &Path) -> anyhow::Result<Vec<CertificateDer<'static>>> {
    let mut reader = BufReader::new(File::open(path)?);
    let certs = certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(certs)
}

/// Load the first private key from a PEM file.
pub fn load_private_key(path: &Path) -> anyhow::Result<PrivateKeyDer<'static>> {
    let mut reader = BufReader::new(File::open(path)?);
    let key = private_key(&mut reader)?
        .ok_or_else(|| anyhow::anyhow!("No private key found in {:?}", path))?;
    Ok(key)
}

use crate::secret_store::SecretStore;

/// Build a TlsAcceptor for mutual TLS (mTLS) using a SecretStore.
pub fn build_mtls_acceptor(
    store: &dyn SecretStore,
) -> anyhow::Result<TlsAcceptor> {
    let ca_certs = store.load_ca_certs()?;
    let server_certs = store.load_entity_certs()?;
    let server_key = store.load_private_key()?;

    let mut root_cert_store = rustls::RootCertStore::empty();
    for cert in ca_certs {
        root_cert_store.add(cert)?;
    }

    let client_auth = rustls::server::WebPkiClientVerifier::builder(Arc::new(root_cert_store))
        .build()?;

    let config = rustls::ServerConfig::builder()
        .with_client_cert_verifier(client_auth)
        .with_single_cert(server_certs, server_key)?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}
