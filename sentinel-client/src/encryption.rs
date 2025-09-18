use anyhow::Result;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::{Aead, OsRng}};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey};
use rand::RngCore;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use rustls::ClientConfig;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum EncryptionType {
    None,
    Aes256Gcm,
    ChaCha20Poly1305,
    Tls,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    pub encryption_type: EncryptionType,
    pub key: Option<Vec<u8>>,
    pub tls_config: Option<TlsConfig>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub ca_cert: Option<Vec<u8>>,
    pub client_cert: Option<Vec<u8>>,
    pub client_key: Option<Vec<u8>>,
    pub server_cert: Option<Vec<u8>>,
    pub server_key: Option<Vec<u8>>,
    pub verify_hostname: bool,
}

#[allow(dead_code)]
pub struct EncryptedStream {
    inner: Box<dyn EncryptedStreamTrait>,
}

#[allow(dead_code)]
trait EncryptedStreamTrait: AsyncRead + AsyncWrite + Send + Unpin {}

impl<T: AsyncRead + AsyncWrite + Send + Unpin> EncryptedStreamTrait for T {}

impl AsyncRead for EncryptedStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for EncryptedStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

#[allow(dead_code)]
pub struct SymmetricEncryptedStream<T> {
    inner: T,
    cipher: SymmetricCipher,
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>,
}

#[allow(dead_code)]
enum SymmetricCipher {
    Aes256Gcm(Aes256Gcm),
    ChaCha20Poly1305(ChaCha20Poly1305),
}

impl<T: AsyncRead + AsyncWrite + Unpin> SymmetricEncryptedStream<T> {
    #[allow(dead_code)]
    pub fn new_aes256(stream: T, key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            anyhow::bail!("AES-256-GCM requires a 32-byte key");
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| anyhow::anyhow!("Failed to create AES cipher: {}", e))?;

        Ok(Self {
            inner: stream,
            cipher: SymmetricCipher::Aes256Gcm(cipher),
            read_buffer: Vec::new(),
            write_buffer: Vec::new(),
        })
    }

    #[allow(dead_code)]
    pub fn new_chacha20(stream: T, key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            anyhow::bail!("ChaCha20Poly1305 requires a 32-byte key");
        }

        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);

        Ok(Self {
            inner: stream,
            cipher: SymmetricCipher::ChaCha20Poly1305(cipher),
            read_buffer: Vec::new(),
            write_buffer: Vec::new(),
        })
    }

    #[allow(dead_code)]
    fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = match &self.cipher {
            SymmetricCipher::Aes256Gcm(cipher) => {
                cipher.encrypt(nonce, plaintext)
                    .map_err(|e| anyhow::anyhow!("AES encryption failed: {}", e))?
            }
            SymmetricCipher::ChaCha20Poly1305(cipher) => {
                cipher.encrypt(nonce, plaintext)
                    .map_err(|e| anyhow::anyhow!("ChaCha20 encryption failed: {}", e))?
            }
        };

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    #[allow(dead_code)]
    fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        if encrypted_data.len() < 12 {
            anyhow::bail!("Encrypted data too short");
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = match &self.cipher {
            SymmetricCipher::Aes256Gcm(cipher) => {
                cipher.decrypt(nonce, ciphertext)
                    .map_err(|e| anyhow::anyhow!("AES decryption failed: {}", e))?
            }
            SymmetricCipher::ChaCha20Poly1305(cipher) => {
                cipher.decrypt(nonce, ciphertext)
                    .map_err(|e| anyhow::anyhow!("ChaCha20 decryption failed: {}", e))?
            }
        };

        Ok(plaintext)
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for SymmetricEncryptedStream<T> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // For simplicity, this is a basic implementation
        // In production, you'd want proper buffering and framing
        let mut temp_buf = vec![0u8; 8192];
        let mut read_buf = tokio::io::ReadBuf::new(&mut temp_buf);

        match std::pin::Pin::new(&mut self.inner).poll_read(cx, &mut read_buf) {
            std::task::Poll::Ready(Ok(())) => {
                let filled = read_buf.filled();
                if filled.is_empty() {
                    return std::task::Poll::Ready(Ok(()));
                }

                match self.decrypt_data(filled) {
                    Ok(decrypted) => {
                        let to_copy = std::cmp::min(decrypted.len(), buf.remaining());
                        buf.put_slice(&decrypted[..to_copy]);
                        std::task::Poll::Ready(Ok(()))
                    }
                    Err(e) => {
                        std::task::Poll::Ready(Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Decryption error: {}", e),
                        )))
                    }
                }
            }
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(e)),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for SymmetricEncryptedStream<T> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.encrypt_data(buf) {
            Ok(encrypted) => {
                match std::pin::Pin::new(&mut self.inner).poll_write(cx, &encrypted) {
                    std::task::Poll::Ready(Ok(_)) => std::task::Poll::Ready(Ok(buf.len())),
                    std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(e)),
                    std::task::Poll::Pending => std::task::Poll::Pending,
                }
            }
            Err(e) => {
                std::task::Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Encryption error: {}", e),
                )))
            }
        }
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

pub struct EncryptionManager;

impl EncryptionManager {
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub async fn wrap_stream(&self, stream: TcpStream, config: EncryptionConfig) -> Result<EncryptedStream> {
        match config.encryption_type {
            EncryptionType::None => {
                Ok(EncryptedStream {
                    inner: Box::new(stream),
                })
            }
            EncryptionType::Aes256Gcm => {
                let key = config.key.ok_or_else(|| anyhow::anyhow!("AES key required"))?;
                let encrypted_stream = SymmetricEncryptedStream::new_aes256(stream, &key)?;
                Ok(EncryptedStream {
                    inner: Box::new(encrypted_stream),
                })
            }
            EncryptionType::ChaCha20Poly1305 => {
                let key = config.key.ok_or_else(|| anyhow::anyhow!("ChaCha20 key required"))?;
                let encrypted_stream = SymmetricEncryptedStream::new_chacha20(stream, &key)?;
                Ok(EncryptedStream {
                    inner: Box::new(encrypted_stream),
                })
            }
            EncryptionType::Tls => {
                self.wrap_with_tls(stream, config.tls_config).await
            }
        }
    }

    #[allow(dead_code)]
    async fn wrap_with_tls(&self, stream: TcpStream, tls_config: Option<TlsConfig>) -> Result<EncryptedStream> {
        let tls_config = tls_config.ok_or_else(|| anyhow::anyhow!("TLS config required"))?;

        // Create a basic TLS client configuration
        let mut root_store = rustls::RootCertStore::empty();

        if let Some(ca_cert) = tls_config.ca_cert {
            let ca_cert = rustls_pemfile::certs(&mut ca_cert.as_slice())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| anyhow::anyhow!("Failed to parse CA certificate: {}", e))?;

            for cert in ca_cert {
                root_store.add(cert)
                    .map_err(|e| anyhow::anyhow!("Failed to add CA certificate: {}", e))?;
            }
        } else {
            // Use system root certificates
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        }

        let client_config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(client_config));
        let domain = rustls::pki_types::ServerName::try_from("localhost")
            .map_err(|e| anyhow::anyhow!("Invalid server name: {}", e))?;

        let tls_stream = connector.connect(domain, stream).await
            .map_err(|e| anyhow::anyhow!("TLS connection failed: {}", e))?;

        Ok(EncryptedStream {
            inner: Box::new(tls_stream),
        })
    }

    #[allow(dead_code)]
    pub fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            encryption_type: EncryptionType::None,
            key: None,
            tls_config: None,
        }
    }
}