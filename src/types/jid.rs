use crate::{new_rhustapp_error, RhustAppError};
use lazy_static::lazy_static;
use libsignal_protocol::{DeviceId, ProtocolAddress};
use std::{fmt, str::FromStr};

/// Default server for users
pub const DEFAULT_USER_SERVER: &str = "s.whatsapp.net";
/// Server for groups
pub const GROUP_SERVER: &str = "g.us";
/// Old Server for users
pub const LEGACY_USER_SERVER: &str = "c.us";
/// Server for broadcasts
pub const BROADCAST_SERVER: &str = "broadcast";
/// Server for hidden users (?)
pub const HIDDEN_USER_SERVER: &str = "lid";

lazy_static! {
    /// Empty JID
    pub static ref EMPTY_JID: JID = JID::new("", "");
    /// JID with group server
    pub static ref GROUP_SERVER_JID: JID = JID::new("", GROUP_SERVER);
    /// JID with default user server
    pub static ref SERVER_JID: JID = JID::new("", DEFAULT_USER_SERVER);
    /// JID with broadcast server
    pub static ref BROADCAST_SERVER_JID: JID = JID::new("", BROADCAST_SERVER);
    /// JID for status updates
    pub static ref STATUS_BROADCAST_JID: JID = JID::new("status", BROADCAST_SERVER);
    /// JID for official WhatsApp chat
    pub static ref PSA_JID: JID = JID::new("0", LEGACY_USER_SERVER);
    /// JID for official WhatsApp Business chat
    pub static ref OFFICIAL_BUSINESS_JID: JID = JID::new("16505361212", LEGACY_USER_SERVER);
}

/// JID represents a WhatsApp user or group ID.
/// There are two types of JIDs: regular JID pairs (user and server)
/// and AD-JIDs (user, agent, device and server).
/// AD JIDs are only used to refer to specific devices of users, so
/// the server is always `s.whatsapp.net` (`DEFAULT_USER_SERVER`).
/// Regular JIDs can be used for entities on any servers (users, groups, broadcasts).
#[derive(Default, PartialEq, Clone)]
pub struct JID {
    pub user: String,
    pub agent: Option<u8>,
    pub device: Option<u8>,
    pub server: String,
}

impl JID {
    /// Creates a new regular JID.
    pub fn new(user: &str, server: &str) -> Self {
        Self {
            user: user.to_string(),
            agent: None,
            device: None,
            server: server.to_string(),
        }
    }

    pub fn new_ad(user: &str, agent: u8, device: u8) -> Self {
        Self {
            user: user.to_string(),
            agent: Some(agent),
            device: Some(device),
            server: DEFAULT_USER_SERVER.to_string(),
        }
    }

    /// Returns whether the JID is AD-JID or not.
    pub fn is_ad(&self) -> bool {
        self.agent.is_some() && self.device.is_some()
    }

    /// Returns true if the JID is a broadcast list, BUT NOT THE STATUS BROADCAST.
    pub fn is_broadcast_list(&self) -> bool {
        self.server.eq(BROADCAST_SERVER) && !self.user.eq(&STATUS_BROADCAST_JID.user)
    }

    /// Returns true if JID has no server (which is required for all JIDs).
    pub fn is_empty(&self) -> bool {
        self.server.len() != 0
    }

    /// Returns the JID's user as an optional u64.
    /// This is only safe to run on normal users, not on groups or
    /// broadcast lists.
    pub fn user_int(&self) -> Option<u64> {
        match self.user.parse() {
            Ok(u) => Some(u),
            Err(_) => None,
        }
    }

    /// Returns a version of JID struct that doesn't have the agent
    /// and device set.
    pub fn to_non_ad(&self) -> Self {
        if self.is_ad() {
            Self {
                user: self.user.to_string(),
                agent: None,
                device: None,
                server: DEFAULT_USER_SERVER.to_string(),
            }
        } else {
            self.clone()
        }
    }

    /// Returns the Signal Protocol address for the user.
    pub fn signal_address(&self) -> ProtocolAddress {
        let mut user = self.user.to_string();

        if let Some(agent) = self.agent {
            user = format!("{}_{}", user, agent);
        };

        ProtocolAddress::new(user, DeviceId::from(self.device.unwrap_or(0) as u32))
    }

    /// Converts the JID into a string representation. The output can be parsed
    /// with `JID::from`, except for JIDs with no user part specified.
    pub fn to_string(&self) -> String {
        if self.is_ad() {
            format!(
                "{}.{}:{}@{}",
                self.user,
                self.agent.unwrap_or(0),
                self.device.unwrap_or(0),
                self.server
            )
        } else if self.user.len() > 0 {
            format!("{}@{}", self.user, self.server)
        } else {
            self.server.to_string()
        }
    }
}

impl fmt::Display for JID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for JID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JID({})", self.to_string())
    }
}

impl FromStr for JID {
    type Err = RhustAppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("@").collect();
        if parts.len() == 0 {
            Err(new_rhustapp_error("failed to split string on '@'", None))
        } else if parts.len() == 1 {
            Ok(JID::new("", parts[0]))
        } else if parts[0].contains(":")
            && parts[0].contains(".")
            && parts[1].eq(DEFAULT_USER_SERVER)
        {
            parse_ad_jid(parts[0])
        } else {
            Ok(JID::new(parts[0], parts[1]))
        }
    }
}

fn parse_ad_jid(user: &str) -> Result<JID, RhustAppError> {
    let mut jid = JID::default();
    jid.server = DEFAULT_USER_SERVER.to_string();

    let dot_opt = user.find(".");
    let colon_opt = user.find(":");

    if dot_opt.is_none() || colon_opt.is_none() {
        return Err(new_rhustapp_error("missing separators ('.', ':')", None));
    };

    let dot_index = dot_opt.unwrap();
    let colon_index = colon_opt.unwrap();

    if colon_index + 1 <= dot_index {
        return Err(new_rhustapp_error(
            "separators ('.', ':') not in correct order",
            None,
        ));
    };

    jid.user = user[..dot_index].to_string();

    let agent: u8 = user[dot_index + 1..colon_index]
        .parse::<u8>()
        .map_err(|err| {
            new_rhustapp_error("failed to parse agent string to u8", Some(err.to_string()))
        })?;
    jid.agent = Some(agent);

    let device: u8 = user[colon_index + 1..].parse::<u8>().map_err(|err| {
        new_rhustapp_error("failed to parse device string to u8", Some(err.to_string()))
    })?;
    jid.device = Some(device);

    Ok(jid)
}
