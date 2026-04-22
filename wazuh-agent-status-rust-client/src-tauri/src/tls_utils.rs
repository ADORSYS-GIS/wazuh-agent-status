use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use rustls_pemfile::{certs, private_key};
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use anyhow::Result;

pub fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>> {
    let mut reader = BufReader::new(File::open(path)?);
    let certs = certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(certs)
}

pub fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>> {
    let mut reader = BufReader::new(File::open(path)?);
    let key = private_key(&mut reader)?
        .ok_or_else(|| anyhow::anyhow!("No private key found in {:?}", path))?;
    Ok(key)
}
