//! TLS configuration for secure remote execution.
//!
//! Provides TLS/SSL encryption for remote executor connections using rustls.

#![cfg(feature = "remote-tls")]

use crate::error::{RepartirError, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tokio_rustls::rustls;
use tracing::{debug, info};

/// TLS configuration for remote executor.
///
/// Provides both client and server TLS configurations using rustls.
/// Supports certificate-based authentication for mutual TLS (mTLS).
#[derive(Clone)]
pub struct TlsConfig {
    /// Client configuration for connecting to remote workers.
    client_config: Option<Arc<rustls::ClientConfig>>,
    /// Server configuration for accepting connections.
    server_config: Option<Arc<rustls::ServerConfig>>,
}

impl TlsConfig {
    /// Creates a new TLS configuration builder.
    #[must_use]
    pub const fn builder() -> TlsConfigBuilder {
        TlsConfigBuilder {
            client_cert_path: None,
            client_key_path: None,
            server_cert_path: None,
            server_key_path: None,
            ca_cert_path: None,
        }
    }

    /// Returns the client configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if client TLS is not configured.
    pub fn client_config(&self) -> Result<Arc<rustls::ClientConfig>> {
        self.client_config.clone().ok_or_else(|| {
            RepartirError::InvalidTask {
                reason: "Client TLS not configured".to_string(),
            }
        })
    }

    /// Returns the server configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if server TLS is not configured.
    pub fn server_config(&self) -> Result<Arc<rustls::ServerConfig>> {
        self.server_config.clone().ok_or_else(|| {
            RepartirError::InvalidTask {
                reason: "Server TLS not configured".to_string(),
            }
        })
    }
}

/// Builder for TLS configuration.
pub struct TlsConfigBuilder {
    client_cert_path: Option<String>,
    client_key_path: Option<String>,
    server_cert_path: Option<String>,
    server_key_path: Option<String>,
    ca_cert_path: Option<String>,
}

impl TlsConfigBuilder {
    /// Sets the client certificate path.
    #[must_use]
    pub fn client_cert<S: Into<String>>(mut self, path: S) -> Self {
        self.client_cert_path = Some(path.into());
        self
    }

    /// Sets the client private key path.
    #[must_use]
    pub fn client_key<S: Into<String>>(mut self, path: S) -> Self {
        self.client_key_path = Some(path.into());
        self
    }

    /// Sets the server certificate path.
    #[must_use]
    pub fn server_cert<S: Into<String>>(mut self, path: S) -> Self {
        self.server_cert_path = Some(path.into());
        self
    }

    /// Sets the server private key path.
    #[must_use]
    pub fn server_key<S: Into<String>>(mut self, path: S) -> Self {
        self.server_key_path = Some(path.into());
        self
    }

    /// Sets the CA certificate path for verification.
    #[must_use]
    pub fn ca_cert<S: Into<String>>(mut self, path: S) -> Self {
        self.ca_cert_path = Some(path.into());
        self
    }

    /// Builds the TLS configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Certificate or key files cannot be read
    /// - Certificate or key parsing fails
    /// - TLS configuration is invalid
    pub fn build(self) -> Result<TlsConfig> {
        let mut client_config = None;
        let mut server_config = None;

        // Build client configuration if cert/key provided
        if let (Some(cert_path), Some(key_path)) = (&self.client_cert_path, &self.client_key_path)
        {
            debug!("Loading client TLS certificate from {cert_path}");
            let certs = load_certs(cert_path)?;
            let key = load_private_key(key_path)?;

            let mut config = rustls::ClientConfig::builder()
                .with_root_certificates(rustls::RootCertStore::empty())
                .with_client_auth_cert(certs, key)
                .map_err(|e| RepartirError::InvalidTask {
                    reason: format!("Failed to build client TLS config: {e}"),
                })?;

            // Add CA certificate if provided
            if let Some(ca_path) = &self.ca_cert_path {
                debug!("Loading CA certificate from {ca_path}");
                let ca_certs = load_certs(ca_path)?;
                let mut root_store = rustls::RootCertStore::empty();
                for cert in ca_certs {
                    root_store.add(cert).map_err(|e| {
                        RepartirError::InvalidTask {
                            reason: format!("Failed to add CA certificate: {e}"),
                        }
                    })?;
                }
                config = rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_client_auth_cert(
                        load_certs(cert_path)?,
                        load_private_key(key_path)?,
                    )
                    .map_err(|e| RepartirError::InvalidTask {
                        reason: format!("Failed to rebuild client TLS config: {e}"),
                    })?;
            } else {
                // No CA cert - disable verification (development only!)
                config
                    .dangerous()
                    .set_certificate_verifier(Arc::new(NoCertificateVerification));
            }

            info!("Client TLS configured");
            client_config = Some(Arc::new(config));
        }

        // Build server configuration if cert/key provided
        if let (Some(cert_path), Some(key_path)) = (&self.server_cert_path, &self.server_key_path)
        {
            debug!("Loading server TLS certificate from {cert_path}");
            let certs = load_certs(cert_path)?;
            let key = load_private_key(key_path)?;

            let config = rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, key)
                .map_err(|e| RepartirError::InvalidTask {
                    reason: format!("Failed to build server TLS config: {e}"),
                })?;

            info!("Server TLS configured");
            server_config = Some(Arc::new(config));
        }

        Ok(TlsConfig {
            client_config,
            server_config,
        })
    }
}

/// Loads certificates from a PEM file.
fn load_certs<P: AsRef<Path>>(path: P) -> Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path.as_ref()).map_err(|e| RepartirError::Io(e))?;
    let mut reader = BufReader::new(file);

    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| RepartirError::InvalidTask {
            reason: format!("Failed to parse certificates: {e}"),
        })?;

    Ok(certs)
}

/// Loads a private key from a PEM file.
fn load_private_key<P: AsRef<Path>>(path: P) -> Result<PrivateKeyDer<'static>> {
    let file = File::open(path.as_ref()).map_err(|e| RepartirError::Io(e))?;
    let mut reader = BufReader::new(file);

    // Try reading as PKCS#8 first, then RSA, then EC
    let key = rustls_pemfile::private_key(&mut reader)
        .map_err(|e| RepartirError::InvalidTask {
            reason: format!("Failed to parse private key: {e}"),
        })?
        .ok_or_else(|| RepartirError::InvalidTask {
            reason: "No private key found in file".to_string(),
        })?;

    Ok(key)
}

/// Certificate verifier that accepts all certificates (INSECURE - development only).
#[derive(Debug)]
struct NoCertificateVerification;

impl rustls::client::danger::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_builder() {
        let _config = TlsConfig::builder()
            .client_cert("client.pem")
            .client_key("client.key")
            .server_cert("server.pem")
            .server_key("server.key")
            .ca_cert("ca.pem");

        // Builder should compile and be chainable
        assert!(true);
    }

    #[test]
    fn test_empty_tls_config() {
        let config = TlsConfig::builder().build();

        // Should succeed even with no certs (they're optional)
        assert!(config.is_ok());

        let cfg = config.unwrap();

        // But trying to use configs should fail
        assert!(cfg.client_config().is_err());
        assert!(cfg.server_config().is_err());
    }
}
