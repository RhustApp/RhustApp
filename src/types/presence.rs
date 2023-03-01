use std::str::FromStr;

use crate::RhustAppError;

pub enum Presence {
    /// "available"
    Available,
    /// "unavailable"
    Unavailable,
    Value(String),
}

impl FromStr for Presence {
    type Err = RhustAppError;

    fn from_str(input: &str) -> Result<Self, RhustAppError> {
        match input {
            "available" => Ok(Self::Available),
            "unavailable" => Ok(Self::Unavailable),
            _ => Ok(Self::Value(input.to_string())),
        }
    }
}

pub enum ChatPresence {
    /// "composing"
    Composing,
    /// "paused"
    Paused,
    Value(String),
}

impl FromStr for ChatPresence {
    type Err = RhustAppError;

    fn from_str(input: &str) -> Result<Self, RhustAppError> {
        match input {
            "composing" => Ok(Self::Composing),
            "paused" => Ok(Self::Paused),
            _ => Ok(Self::Value(input.to_string())),
        }
    }
}

pub enum ChatPresenceMedia {
    /// ""
    Text,
    /// "audio"
    Audio,
    Value(String),
}

impl FromStr for ChatPresenceMedia {
    type Err = RhustAppError;

    fn from_str(input: &str) -> Result<Self, RhustAppError> {
        match input {
            "" => Ok(Self::Text),
            "audio" => Ok(Self::Audio),
            _ => Ok(Self::Value(input.to_string())),
        }
    }
}
