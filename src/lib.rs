use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;
use std::fmt; // <--- NEW IMPORT
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use rustls::ClientConfig;

// --- Protocol Constants ---
const PACKET_HANDSHAKE: u8 = 0x10;
const PACKET_AUTH_OK: u8   = 0x11;
#[allow(dead_code)] // Suppress warning for unused constant
const PACKET_AUTH_ERR: u8  = 0x12;
const PACKET_QUERY: u8     = 0x20;
const PACKET_MSG: u8       = 0x02;
const PACKET_DATA: u8      = 0x03;

const DRIVER_SIG: &str = "maazdb-rust-driver-v1";

#[derive(Debug)]
pub enum MaazDBError {
    IoError(io::Error),
    AuthError(String),
    ProtocolError(String),
    TlsError(rustls::Error),
}

// --- NEW: Implement Display (Required for std::error::Error) ---
impl fmt::Display for MaazDBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaazDBError::IoError(e) => write!(f, "IO Error: {}", e),
            MaazDBError::AuthError(s) => write!(f, "Authentication Error: {}", s),
            MaazDBError::ProtocolError(s) => write!(f, "Protocol Error: {}", s),
            MaazDBError::TlsError(e) => write!(f, "TLS Error: {}", e),
        }
    }
}

// --- NEW: Implement std::error::Error ---
impl std::error::Error for MaazDBError {}

// --- Existing Conversions ---
impl From<io::Error> for MaazDBError {
    fn from(err: io::Error) -> Self { MaazDBError::IoError(err) }
}

impl From<rustls::Error> for MaazDBError {
    fn from(err: rustls::Error) -> Self { MaazDBError::TlsError(err) }
}

// Helper for self-signed certs
struct NoCertificateVerification;
impl rustls::client::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(&self, _: &rustls::Certificate, _: &[rustls::Certificate], _: &rustls::ServerName, _: &mut dyn Iterator<Item = &[u8]>, _: &[u8], _: std::time::SystemTime) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

/// The official MaazDB Client.
pub struct MaazDB {
    stream: rustls::StreamOwned<rustls::ClientConnection, TcpStream>,
    pub connected: bool,
}

impl MaazDB {
    /// Connects to a MaazDB server instance.
    pub fn connect(host: &str, port: u16, user: &str, pass: &str) -> Result<Self, MaazDBError> {
        let addr = format!("{}:{}", host, port);
        
        let sock = TcpStream::connect(&addr)?;
        sock.set_read_timeout(Some(Duration::from_secs(10)))?;
        sock.set_write_timeout(Some(Duration::from_secs(10)))?;

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
            .with_no_client_auth();
        
        let server_name = rustls::ServerName::try_from("localhost").unwrap();
        let conn = rustls::ClientConnection::new(Arc::new(config), server_name)?;
        let mut stream = rustls::StreamOwned::new(conn, sock);

        let payload = format!("{}\0{}\0{}", user, pass, DRIVER_SIG);
        Self::send_packet(&mut stream, PACKET_HANDSHAKE, payload.as_bytes())?;

        let (ptype, msg) = Self::read_packet(&mut stream)?;

        if ptype == PACKET_AUTH_OK {
            Ok(MaazDB { stream, connected: true })
        } else {
            Err(MaazDBError::AuthError(msg))
        }
    }

    pub fn query(&mut self, sql: &str) -> Result<String, MaazDBError> {
        if !self.connected {
            return Err(MaazDBError::ProtocolError("Not connected".into()));
        }
        Self::send_packet(&mut self.stream, PACKET_QUERY, sql.as_bytes())?;
        let (ptype, msg) = Self::read_packet(&mut self.stream)?;
        match ptype {
            PACKET_MSG | PACKET_DATA => Ok(msg),
            _ => Err(MaazDBError::ProtocolError(msg)),
        }
    }

    pub fn close(&mut self) {
        let _ = self.stream.conn.send_close_notify();
        self.connected = false;
    }

    fn send_packet(stream: &mut rustls::StreamOwned<rustls::ClientConnection, TcpStream>, ptype: u8, payload: &[u8]) -> io::Result<()> {
        stream.write_u8(ptype)?;
        stream.write_u32::<BigEndian>(payload.len() as u32)?;
        stream.write_all(payload)?;
        stream.flush()?;
        Ok(())
    }

    fn read_packet(stream: &mut rustls::StreamOwned<rustls::ClientConnection, TcpStream>) -> io::Result<(u8, String)> {
        let ptype = stream.read_u8()?;
        let len = stream.read_u32::<BigEndian>()? as usize;
        
        if len > 10 * 1024 * 1024 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Packet too large"));
        }

        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf)?;
        Ok((ptype, String::from_utf8_lossy(&buf).to_string()))
    }
}