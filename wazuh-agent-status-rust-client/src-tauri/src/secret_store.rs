use std::path::{Path, PathBuf};
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use anyhow::Result;
use crate::tls_utils;

pub trait SecretStore: Send + Sync {
    fn load_ca_certs(&self) -> Result<Vec<CertificateDer<'static>>>;
    fn load_entity_certs(&self) -> Result<Vec<CertificateDer<'static>>>;
    fn load_private_key(&self) -> Result<PrivateKeyDer<'static>>;
}

pub struct FileSecretStore {
    ca_path: PathBuf,
    cert_path: PathBuf,
    key_path: PathBuf,
}

impl FileSecretStore {
    pub fn new(ca_path: PathBuf, cert_path: PathBuf, key_path: PathBuf) -> Self {
        Self { ca_path, cert_path, key_path }
    }

    #[cfg(unix)]
    fn check_permissions(path: &Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path)?;
        let mode = metadata.permissions().mode();
        if mode & 0o007 != 0 {
            anyhow::bail!("Security risk: Secret file {:?} is world-readable (mode: {:o})", path, mode);
        }
        Ok(())
    }

    #[cfg(not(unix))]
    fn check_permissions(_path: &Path) -> Result<()> {
        Ok(())
    }
}

impl SecretStore for FileSecretStore {
    fn load_ca_certs(&self) -> Result<Vec<CertificateDer<'static>>> {
        tls_utils::load_certs(&self.ca_path)
    }

    fn load_entity_certs(&self) -> Result<Vec<CertificateDer<'static>>> {
        tls_utils::load_certs(&self.cert_path)
    }

    fn load_private_key(&self) -> Result<PrivateKeyDer<'static>> {
        Self::check_permissions(&self.key_path)?;
        tls_utils::load_private_key(&self.key_path)
    }
}
