use std::fmt::Display;

use time::{Duration, OffsetDateTime};

use crate::{types::JID, RhustAppError};

pub enum RhustAppEventType {
    /// It is emitted after connecting when there's no session data in the device store.
    ///
    /// The QR codes are available in the Codes slice. You should render the strings as QR
    /// codes one by one, switching to the next one whenever enough time has passed. WhatsApp
    /// web seems to show first code for 60 seconds and all the other codes for 20 seconds.
    ///
    /// When the QR code has been scanned and pairing is complete, `PairSuccess` will be emitted.
    /// If you run out of codes before scanning, the server will close the websocket, and you
    /// will have to reconnect to get more codes.
    QR(QR),

    /// It is emitted after the QR code has been scanned with the phone and the handshake
    /// has been completed. Note that this is generally followed by a websocket reconnection,
    /// so you should wait for the Connected before trying to send anything.
    PairSuccess(PairSuccess),

    /// It is emitted when a pair-success event is received from the server, but finishing
    /// the pairing locally fails.
    PairError(PairError),

    /// It is emitted when the pairing QR code is scanned, but the phone didn't have multidevice
    /// enabled. The same QR code can still be scanned after this event, which means the user can
    /// just be told to enable multidevice and re-scan the code.
    QRScannedWithoutMultidevice,

    /// It is emitted when the client has successfully connected to the WhatsApp servers
    /// and is authenticated. The user, who the client is authenticated as, will be in the device
    /// store at this point, which is why this event doesn't contain any data.
    Connected,

    /// It is emitted when the keepalive ping request to WhatsApp web servers time out.
    ///
    /// Currently, there's no automatic handling for these, but it's expected that the TCP
    /// connection will either start working again or notice it's dead on its own eventually.
    /// Clients may use this event to decide to force a disconnect+reconnect faster.
    KeepAliveTimeout(KeepAliveTimeout),

    /// It is emitted if the keepalive ping starts working again after some `KeepAliveTimeout`
    /// events. Note that if the websocket disconnects before the pings start working, this
    /// event will not be emitted.
    KeepAliveRestored,

    /// It is emitted when the client has been unpaired from the device.
    ///
    /// This can happen while connected (stream:error messages) or right after connecting
    /// (connect failure messages).
    ///
    /// This will not be emitted when the logout is initialized by this client itself.
    LoggedOut(LoggedOut),

    /// It is emitted when the client is disconnected by another client connecting with the
    /// same keys.
    ///
    /// This can happen if you accidentally start another process with the same session or
    /// otherwise try to connect twice with the same session.
    StreamReplaced,

    /// It is emitted when there's a connection failure with the `ConnectFailureReason::TempBanned` reason code.
    TemporaryBan(TemporaryBan),
}

pub struct QR {
    pub codes: Vec<String>,
}

pub struct PairSuccess {
    pub id: JID,
    pub business_name: String,
    pub platform: String,
}

pub struct PairError {
    pub id: JID,
    pub business_name: String,
    pub platform: String,
    pub error: RhustAppError,
}

pub struct KeepAliveTimeout {
    pub error_count: i32,
    pub last_success: OffsetDateTime,
}

/// It is an error code included in the connection failure events.
///
/// 400, 500 and 501 are also existing codes, but the meaning is unknown
///
/// 503 doesn't seem to be included in the web app JS with the other codes, and its
/// very rare, but does happen after a 503 stream error sometimes.
pub enum ConnectFailureReason {
    /// 401
    LoggedOut,
    /// 402
    TempBanned,
    /// 403
    MainDeviceGone,
    /// 406
    UnknownLogout,

    /// 405
    ClientOutdated,
    /// 409
    BadUserAgent,

    /// 503
    ServiceUnavailable,

    Value(i32),
}

impl From<i32> for ConnectFailureReason {
    fn from(value: i32) -> Self {
        match value {
            401 => Self::LoggedOut,
            402 => Self::TempBanned,
            403 => Self::MainDeviceGone,
            406 => Self::UnknownLogout,
            405 => Self::ClientOutdated,
            409 => Self::BadUserAgent,
            503 => Self::ServiceUnavailable,
            _ => Self::Value(value),
        }
    }
}

impl ConnectFailureReason {
    pub fn to_error_code(&self) -> i32 {
        match self {
            Self::LoggedOut => 401,
            Self::TempBanned => 402,
            Self::MainDeviceGone => 403,
            Self::UnknownLogout => 406,
            Self::ClientOutdated => 405,
            Self::BadUserAgent => 409,
            Self::ServiceUnavailable => 503,
            Self::Value(value) => *value,
        }
    }

    pub fn to_string(&self) -> String {
        let error_meaning = match self {
            Self::LoggedOut => String::from("logged out from another device"),
            Self::TempBanned => String::from("account temporarily banned"),
            Self::MainDeviceGone => String::from("primary device was logged out"),
            Self::UnknownLogout => String::from("logged out for unknown reasons"),
            Self::ClientOutdated => String::from("client is out of date"),
            Self::BadUserAgent => String::from("client user agent was rejected"),
            Self::ServiceUnavailable => String::from("service is unavailable"),
            Self::Value(_) => format!("unknown error"),
        };

        format!(
            "{error_code}: {error_meaning}",
            error_code = self.to_error_code()
        )
    }

    pub fn is_logged_out(&self) -> bool {
        match self {
            Self::LoggedOut | Self::MainDeviceGone | Self::UnknownLogout => true,
            _ => false,
        }
    }
}

impl Display for ConnectFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct LoggedOut {
    /// It is true if the event was triggered by a connect failure message.
    /// If it's false, the event was triggered by a stream:error message.
    pub on_connect: bool,
    /// If `on_connect` is true, then this field contains the reason code.
    pub reason: ConnectFailureReason,
}

pub enum TempBanReason {
    /// 101
    SentToTooManyPeople,
    /// 102
    BlockedByUsers,
    /// 103
    CreatedTooManyGroups,
    /// 104
    SentTooManySameMessages,
    /// 106
    BroadcastList,

    Value(i32),
}

impl From<i32> for TempBanReason {
    fn from(value: i32) -> Self {
        match value {
            101 => Self::SentToTooManyPeople,
            102 => Self::BlockedByUsers,
            103 => Self::CreatedTooManyGroups,
            104 => Self::SentTooManySameMessages,
            106 => Self::BroadcastList,
            _ => Self::Value(value),
        }
    }
}

impl TempBanReason {
    pub fn to_error_code(&self) -> i32 {
        match self {
            Self::SentToTooManyPeople => 101,
            Self::BlockedByUsers => 102,
            Self::CreatedTooManyGroups => 103,
            Self::SentTooManySameMessages => 104,
            Self::BroadcastList => 106,
            Self::Value(value) => *value,
        }
    }

    pub fn to_string(&self) -> String {
        let error_meaning = match self {
            Self::SentToTooManyPeople => String::from(
                "you sent too many messages to people who didn't have you in their address books",
            ),
            Self::BlockedByUsers => String::from("too many people blocked you"),
            Self::CreatedTooManyGroups => String::from(
                "you created too many groups with people who didn't have you in their address books",
            ),
            Self::SentTooManySameMessages => String::from("you sent the same message to too many people"),
            Self::BroadcastList => String::from("you sent too many messages to a broadcast list"),
            Self::Value(_) => format!("you may have violated the terms and service (unknown reason)"),
        };

        format!(
            "{error_code}: {error_meaning}",
            error_code = self.to_error_code()
        )
    }
}

impl Display for TempBanReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct TemporaryBan {
    pub code: TempBanReason,
    pub expire: Duration,
}

impl TemporaryBan {
    pub fn to_string(&self) -> String {
        if self.expire.is_zero() {
            format!("You've been temporarily banned: {}", self.code)
        } else {
            format!(
                "You've been temporarily banned: {}. The ban expires in {}",
                self.code, self.expire
            )
        }
    }
}

// TODO: implement the remaining things after `Node`.
