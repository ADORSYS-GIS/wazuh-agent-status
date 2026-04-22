use std::path::{Path, PathBuf};
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use anyhow::Result;

/// Trait for retrieving cryptographic secrets.
pub trait SecretStore: Send + Sync {
    /// Load CA certificates.
    fn load_ca_certs(&self) -> Result<Vec<CertificateDer<'static>>>;
    /// Load entity certificates (server or client).
    fn load_entity_certs(&self) -> Result<Vec<CertificateDer<'static>>>;
    /// Load the private key.
    fn load_private_key(&self) -> Result<PrivateKeyDer<'static>>;
    /// Get the remaining duration until the entity certificate expires.
    /// Returns None if the certificate has no expiration or is already expired.
    fn check_expiration(&self) -> Result<Option<std::time::Duration>>;
}

/// A SecretStore that loads from the filesystem.
pub struct FileSecretStore {
    ca_path: PathBuf,
    cert_path: PathBuf,
    key_path: PathBuf,
}

impl FileSecretStore {
    pub fn new(ca_path: PathBuf, cert_path: PathBuf, key_path: PathBuf) -> Self {
        Self { ca_path, cert_path, key_path }
    }

    /// On Unix, verify that the file is not world-readable.
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
        // Windows/other permission checks can be added here
        Ok(())
    }
}

impl SecretStore for FileSecretStore {
    fn load_ca_certs(&self) -> Result<Vec<CertificateDer<'static>>> {
        super::tls::load_certs(&self.ca_path)
    }

    fn load_entity_certs(&self) -> Result<Vec<CertificateDer<'static>>> {
        super::tls::load_certs(&self.cert_path)
    }

    fn load_private_key(&self) -> Result<PrivateKeyDer<'static>> {
        Self::check_permissions(&self.key_path)?;
        super::tls::load_private_key(&self.key_path)
    }

    fn check_expiration(&self) -> Result<Option<std::time::Duration>> {
        use x509_parser::prelude::*;
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(&self.cert_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Certificates in PEM format need to be decoded to DER for x509-parser
        let mut reader = std::io::BufReader::new(&buffer[..]);
        let cert_ders = rustls_pemfile::certs(&mut reader).collect::<std::io::Result<Vec<_>>>()?;
        
        let first_cert = cert_ders.first().ok_or_else(|| anyhow::anyhow!("No certificates found in {:?}", self.cert_path))?;
        
        // Parse X.509
        let (_, x509) = X509Certificate::from_der(first_cert.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to parse X.509 certificate: {}", e))?;

        let not_after = x509.validity().not_after;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let expiry_secs = not_after.timestamp() as u64;
        
        if expiry_secs > now {
            Ok(Some(std::time::Duration::from_secs(expiry_secs - now)))
        } else {
            Ok(None)
        }
    }
}
