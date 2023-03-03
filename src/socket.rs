//! `socket` implements a subset of the Noise protocol framework on top of websockets as used
//! by WhatsApp.

use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
};

use tungstenite::{http::Uri, stream::MaybeTlsStream, WebSocket};

use crate::{binary::token, new_rhustapp_error, RhustAppError};

/// It is the Origin header for all WhatsApp websocket connection.
pub const ORIGIN: &str = "https://web.whatsapp.com";
/// It is the websocket URL for the new multidevice protocol.
pub const URL: &str = "wss://web.whatsapp.com/ws/chat";

pub const NOISE_START_PATTERN: &str = "Noise_XX_25519_AESGCM_SHA256\x00\x00\x00\x00";
pub const WA_MAGIC_VALUE: u8 = 5;

pub fn get_wa_header() -> [u8; 4] {
    [b'W', b'A', WA_MAGIC_VALUE, token::DICT_VERSION]
}

pub const FRAME_MAX_SIZE: usize = 2 << 23;
pub const FRAME_LENGTH_SIZE: usize = 3;

pub enum SocketError {
    FrameTooLarge,
    SocketClosed,
    SocketAlreadyOpen,
}

impl SocketError {
    pub fn to_string(&self) -> String {
        match self {
            Self::FrameTooLarge => String::from("frame is too large"),
            Self::SocketClosed => String::from("frame socket is closed"),
            Self::SocketAlreadyOpen => String::from("frame socket is already open"),
        }
    }
}

pub struct FrameSocket {
    connection: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    pub header: Option<[u8; 4]>,
    lock: Arc<Mutex<u8>>,
    incoming_length: usize,
    received_length: usize,
}

impl FrameSocket {
    pub fn new() -> Self {
        Self {
            connection: None,
            header: Some(get_wa_header()),
            lock: Arc::new(Mutex::new(0)),
            incoming_length: 0,
            received_length: 0,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub fn close(&mut self, code: i32) {
        todo!()
    }

    pub fn connect(&mut self) -> Result<(), RhustAppError> {
        let lock = Arc::clone(&self.lock);
        let mut data = lock
            .lock()
            .map_err(|err| new_rhustapp_error("failed to get a lock", Some(err.to_string())))?;
        *data += 1;

        if self.connection.is_some() {
            return Err(new_rhustapp_error(
                "failed to connect",
                Some(SocketError::SocketAlreadyOpen.to_string()),
            ));
        };

        let ws_request = Self::build_connnection_request().map_err(|err| {
            new_rhustapp_error(
                "failed to build websocket connection request",
                Some(err.to_string()),
            )
        })?;

        let (socket, _) = tungstenite::connect(ws_request).map_err(|err| {
            new_rhustapp_error("failed to connect to websocket", Some(err.to_string()))
        })?;
        self.connection = Some(socket);

        Ok(())
    }

    fn build_connnection_request() -> Result<tungstenite::http::Request<()>, RhustAppError> {
        let ws_uri = URL.parse::<Uri>().map_err(|err| {
            new_rhustapp_error("failed to parse URL into Uri", Some(err.to_string()))
        })?;

        let authority = ws_uri.authority().unwrap().as_str();
        let host = authority
            .find('@')
            .map(|idx| authority.split_at(idx + 1).1)
            .unwrap_or_else(|| authority);

        let ws_request = tungstenite::http::Request::builder()
            .method("GET")
            .header("Host", host)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                tungstenite::handshake::client::generate_key(),
            )
            .header("Origin", ORIGIN)
            .uri(ws_uri)
            .body(())
            .map_err(|err| {
                new_rhustapp_error(
                    "failed to build new request for websocket",
                    Some(err.to_string()),
                )
            })?;

        Ok(ws_request)
    }

    fn read_pump(&mut self) {}
}
