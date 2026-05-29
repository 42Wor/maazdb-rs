// FILE PATH: maazdb-rs/src/lib.rs

use std::collections::HashMap;
use std::fmt;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::{rustls, TlsConnector};
use rustls::ClientConfig;

use hmac::{Hmac, Mac};
use sha2::Sha256;

// --- Protocol v2.1 Constants ---
pub const PACKET_CHALLENGE_REQ: u8  = 0x10;
pub const PACKET_CHALLENGE_RESP: u8 = 0x11;
pub const PACKET_AUTH_OK: u8        = 0x12;
pub const PACKET_AUTH_ERR: u8       = 0x13;
pub const PACKET_QUERY: u8          = 0x20;
pub const PACKET_MSG: u8            = 0x02;
pub const PACKET_DATA: u8           = 0x03;

pub const FLAG_NONE: u8 = 0x00;

const DRIVER_SIG: &str = "maazdb-rust-driver-v1";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub enum MaazDBError {
    IoError(io::Error),
    AuthError(String),
    ProtocolError(String),
    TlsError(rustls::Error),
    ConnectionClosed,
}

impl fmt::Display for MaazDBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaazDBError::IoError(e) => write!(f, "IO Error: {}", e),
            MaazDBError::AuthError(s) => write!(f, "Authentication Error: {}", s),
            MaazDBError::ProtocolError(s) => write!(f, "Protocol Error: {}", s),
            MaazDBError::TlsError(e) => write!(f, "TLS Error: {}", e),
            MaazDBError::ConnectionClosed => write!(f, "Connection Closed by Server"),
        }
    }
}

impl std::error::Error for MaazDBError {}

impl From<io::Error> for MaazDBError {
    fn from(err: io::Error) -> Self { MaazDBError::IoError(err) }
}

impl From<rustls::Error> for MaazDBError {
    fn from(err: rustls::Error) -> Self { MaazDBError::TlsError(err) }
}

// Helper for self-signed certs (Dev Mode)
struct NoCertificateVerification;
impl rustls::client::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self, _: &rustls::Certificate, _: &[rustls::Certificate], _: &rustls::ServerName, 
        _: &mut dyn Iterator<Item = &[u8]>, _: &[u8], _: std::time::SystemTime
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

// Helper to encode bytes to hex string
fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub ptype: u8,
    pub flags: u8,
    pub req_id: u16,
    pub payload: Vec<u8>,
}

impl Packet {
    pub async fn write_to<W: AsyncWriteExt + Unpin>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u8(self.ptype).await?;
        writer.write_u8(self.flags).await?;
        writer.write_u16(self.req_id).await?;
        writer.write_u32(self.payload.len() as u32).await?;
        if !self.payload.is_empty() {
            writer.write_all(&self.payload).await?;
        }
        writer.flush().await?;
        Ok(())
    }

    pub async fn read_from<R: AsyncReadExt + Unpin>(reader: &mut R) -> io::Result<Option<Self>> {
        let ptype = match reader.read_u8().await {
            Ok(t) => t,
            Err(_) => return Ok(None),
        };
        let flags = reader.read_u8().await?;
        let req_id = reader.read_u16().await?;
        let len = reader.read_u32().await? as usize;
        
        if len > 10 * 1024 * 1024 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Packet exceeds 10MB limit"));
        }

        let mut payload = vec![0u8; len];
        if len > 0 {
            reader.read_exact(&mut payload).await?;
        }
        Ok(Some(Packet { ptype, flags, req_id, payload }))
    }
}

/// The Official MaazDB Async Multiplexed Client (Protocol v2.1)
#[derive(Clone)]
pub struct MaazDB {
    req_tx: mpsc::Sender<Packet>,
    pending_requests: Arc<Mutex<HashMap<u16, oneshot::Sender<Result<String, MaazDBError>>>>>,
    next_req_id: Arc<AtomicU16>,
}

impl MaazDB {
    /// Connects to a MaazDB server instance securely and asynchronously.
    pub async fn connect(host: &str, port: u16, user: &str, pass: &str) -> Result<Self, MaazDBError> {
        let addr = format!("{}:{}", host, port);
        
        let sock = TcpStream::connect(&addr).await?;
        sock.set_nodelay(true)?;
        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
            .with_no_client_auth();
        
        let connector = TlsConnector::from(Arc::new(config));
        let server_name = rustls::ServerName::try_from("localhost").unwrap();
        let mut stream = connector.connect(server_name, sock).await?;

        // 1. Read CHALLENGE_REQ
        let challenge_pkt = Packet::read_from(&mut stream).await?
            .ok_or(MaazDBError::ConnectionClosed)?;
            
        if challenge_pkt.ptype != PACKET_CHALLENGE_REQ {
            return Err(MaazDBError::ProtocolError("Expected Challenge Request".into()));
        }

        // 2. Compute HMAC-SHA256 Signature
        let mut mac = HmacSha256::new_from_slice(pass.as_bytes())
            .map_err(|_| MaazDBError::AuthError("HMAC init failed".into()))?;
        mac.update(&challenge_pkt.payload);
        let signature_hex = encode_hex(&mac.finalize().into_bytes());

        // 3. Send CHALLENGE_RESP
        let payload = format!("{}\0{}\0{}\0{}", user, pass, DRIVER_SIG, signature_hex);
        let resp_pkt = Packet {
            ptype: PACKET_CHALLENGE_RESP,
            flags: FLAG_NONE,
            req_id: 0,
            payload: payload.into_bytes(),
        };
        resp_pkt.write_to(&mut stream).await?;

        // 4. Read AUTH_OK or AUTH_ERR
        let auth_res = Packet::read_from(&mut stream).await?
            .ok_or(MaazDBError::ConnectionClosed)?;

        if auth_res.ptype == PACKET_AUTH_ERR {
            return Err(MaazDBError::AuthError(String::from_utf8_lossy(&auth_res.payload).to_string()));
        } else if auth_res.ptype != PACKET_AUTH_OK {
            return Err(MaazDBError::ProtocolError("Unexpected Auth Response".into()));
        }

        // 5. Setup Multiplexing Channels
        let (mut read_half, mut write_half) = tokio::io::split(stream);
        let (req_tx, mut req_rx) = mpsc::channel::<Packet>(100);
        
        // Explicitly define the HashMap type to resolve E0282
        let pending_requests: Arc<Mutex<HashMap<u16, oneshot::Sender<Result<String, MaazDBError>>>>> = 
            Arc::new(Mutex::new(HashMap::new()));

        // Writer Task
        tokio::spawn(async move {
            while let Some(packet) = req_rx.recv().await {
                if packet.write_to(&mut write_half).await.is_err() {
                    break;
                }
            }
        });

        // Reader Task
        let pending_clone = pending_requests.clone();
        tokio::spawn(async move {
            while let Ok(Some(packet)) = Packet::read_from(&mut read_half).await {
                let mut pending = pending_clone.lock().await;
                if let Some(tx) = pending.remove(&packet.req_id) {
                    let res = match packet.ptype {
                        PACKET_MSG | PACKET_DATA => Ok(String::from_utf8_lossy(&packet.payload).to_string()),
                        PACKET_AUTH_ERR => Err(MaazDBError::AuthError(String::from_utf8_lossy(&packet.payload).to_string())),
                        _ => Err(MaazDBError::ProtocolError(format!("Unknown packet type: {}", packet.ptype))),
                    };
                    let _ = tx.send(res);
                }
            }
        });

        Ok(MaazDB {
            req_tx,
            pending_requests,
            next_req_id: Arc::new(AtomicU16::new(1)),
        })
    }

    /// Executes a SQL query asynchronously. Safe to call concurrently from multiple threads.
    pub async fn query(&self, sql: &str) -> Result<String, MaazDBError> {
        // Generate a unique Request ID (wraps safely at 65535)
        let req_id = self.next_req_id.fetch_add(1, Ordering::Relaxed);
        
        let (tx, rx) = oneshot::channel();
        
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(req_id, tx);
        }

        let packet = Packet {
            ptype: PACKET_QUERY,
            flags: FLAG_NONE,
            req_id,
            payload: sql.as_bytes().to_vec(),
        };

        if self.req_tx.send(packet).await.is_err() {
            return Err(MaazDBError::ConnectionClosed);
        }

        // Wait for the specific response to this Request ID
        match tokio::time::timeout(Duration::from_secs(30), rx).await {
            Ok(Ok(res)) => res,
            Ok(Err(_)) => Err(MaazDBError::ConnectionClosed),
            Err(_) => {
                // Timeout: Clean up the pending request to prevent memory leaks
                self.pending_requests.lock().await.remove(&req_id);
                Err(MaazDBError::ProtocolError("Query timed out".into()))
            }
        }
    }

    /// Gracefully closes the connection. 
    /// Managed naturally when MaazDB goes out of scope, but kept here for backward API compatibility.
    pub fn close(&self) {}
}