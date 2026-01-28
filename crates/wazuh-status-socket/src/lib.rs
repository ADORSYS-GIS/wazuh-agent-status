use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
#[cfg(windows)]
use tokio::net::TcpListener;

#[cfg(windows)]
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Endpoint, Uri};

#[derive(Debug, Clone)]
pub enum SocketConfig {
    Unix { path: PathBuf },
    TcpLoopback { port: u16 },
}

#[cfg(not(windows))]
use std::path::Path;

#[cfg(not(windows))]
use std::os::unix::ffi::OsStrExt;

#[cfg(not(windows))]
use std::os::unix::fs::PermissionsExt;

#[cfg(not(windows))]
use std::ffi::CString;

#[cfg(not(windows))]
use libc::gid_t;

#[cfg(not(windows))]
use tokio::net::UnixListener;

#[cfg(not(windows))]
use tokio_stream::wrappers::UnixListenerStream;

#[cfg(not(windows))]
use hyper_util::rt::TokioIo;

#[cfg(not(windows))]
pub async fn bind_incoming(config: &SocketConfig) -> Result<UnixListenerStream> {
    match config {
        SocketConfig::Unix { path } => {
            ensure_parent_dir(path)?;
            if path.exists() {
                std::fs::remove_file(path)
                    .with_context(|| format!("failed to remove existing socket at {}", path.display()))?;
            }
            let listener = UnixListener::bind(path)
                .with_context(|| format!("failed to bind unix socket at {}", path.display()))?;
            apply_socket_permissions(path)?;
            Ok(UnixListenerStream::new(listener))
        }
        SocketConfig::TcpLoopback { .. } => Err(anyhow!("tcp loopback is only supported on windows")),
    }
}

#[cfg(windows)]
pub async fn bind_incoming(config: &SocketConfig) -> Result<TcpListenerStream> {
    match config {
        SocketConfig::TcpLoopback { port } => {
            let addr = format!("127.0.0.1:{port}");
            let listener = TcpListener::bind(&addr)
                .await
                .with_context(|| format!("failed to bind tcp listener on {addr}"))?;
            Ok(TcpListenerStream::new(listener))
        }
        SocketConfig::Unix { .. } => Err(anyhow!("unix sockets are not supported on windows")),
    }
}

pub async fn connect_channel(config: &SocketConfig) -> Result<Channel> {
    match config {
        SocketConfig::Unix { path } => connect_uds(path).await,
        SocketConfig::TcpLoopback { port } => connect_tcp_loopback(*port).await,
    }
}

#[cfg(not(windows))]
async fn connect_uds(path: &Path) -> Result<Channel> {
    let path = path.to_path_buf();
    let endpoint = Endpoint::try_from("http://[::]:50051")?;
    let channel = endpoint
        .connect_with_connector(tower::service_fn(move |_: Uri| {
            let path = path.clone();
            async move {
                tokio::net::UnixStream::connect(path)
                    .await
                    .map(TokioIo::new)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
            }
        }))
        .await?;
    Ok(channel)
}

async fn connect_tcp_loopback(port: u16) -> Result<Channel> {
    let endpoint = Endpoint::try_from(format!("http://127.0.0.1:{port}"))?;
    Ok(endpoint.connect().await?)
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create socket directory {}", parent.display()))?;
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o770))
            .with_context(|| format!("failed to set socket directory permissions {}", parent.display()))?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn apply_socket_permissions(path: &Path) -> Result<()> {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o660))
        .with_context(|| format!("failed to set socket permissions {}", path.display()))?;
    let euid = unsafe { libc::geteuid() };
    if euid != 0 {
        return Ok(());
    }
    let gid = lookup_group_gid("wazuh")?;
    let c_path = CString::new(path.as_os_str().as_bytes())
        .with_context(|| format!("invalid socket path {}", path.display()))?;
    let result = unsafe { libc::chown(c_path.as_ptr(), 0, gid) };
    if result != 0 {
        return Err(anyhow!(
            "failed to chown socket {} to root:wazuh",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
fn lookup_group_gid(group: &str) -> Result<gid_t> {
    let c_group = CString::new(group).context("invalid group name")?;
    let grp = unsafe { libc::getgrnam(c_group.as_ptr()) };
    if grp.is_null() {
        return Err(anyhow!("group not found: {group}"));
    }
    let gid = unsafe { (*grp).gr_gid };
    Ok(gid)
}

#[cfg(test)]
mod tests {
    use super::ensure_parent_dir;
    use std::path::PathBuf;

    #[test]
    fn ensure_parent_dir_creates_directory() {
        let temp = tempfile::tempdir().expect("tempdir");
        let socket_path = PathBuf::from(temp.path()).join("nested").join("sock.sock");

        ensure_parent_dir(&socket_path).expect("ensure_parent_dir");

        assert!(socket_path.parent().unwrap().exists());
    }
}
