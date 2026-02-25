use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CertPaths {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

pub fn default_local_cert_paths() -> CertPaths {
    CertPaths {
        cert_path: PathBuf::from("certs/server.crt"),
        key_path: PathBuf::from("certs/server.key"),
    }
}

pub fn generate_localhost_cert_files(
    cert_path: impl AsRef<Path>,
    key_path: impl AsRef<Path>,
) -> Result<CertPaths, Box<dyn Error>> {
    let cert_path = cert_path.as_ref().to_path_buf();
    let key_path = key_path.as_ref().to_path_buf();

    if let Some(parent) = cert_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if let Some(parent) = key_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;

    fs::write(&cert_path, generated.cert.pem())?;
    fs::write(&key_path, generated.key_pair.serialize_pem())?;

    Ok(CertPaths {
        cert_path,
        key_path,
    })
}

pub fn ensure_localhost_cert_files(
    cert_path: impl AsRef<Path>,
    key_path: impl AsRef<Path>,
) -> Result<(CertPaths, bool), Box<dyn Error>> {
    let cert_path = cert_path.as_ref().to_path_buf();
    let key_path = key_path.as_ref().to_path_buf();

    if cert_path.exists() && key_path.exists() {
        return Ok((
            CertPaths {
                cert_path,
                key_path,
            },
            false,
        ));
    }

    let paths = generate_localhost_cert_files(&cert_path, &key_path)?;
    Ok((paths, true))
}
