use std::str::FromStr;

use crate::{binary::proto as wa_proto, new_rhustapp_error, RhustAppError};

use super::JID;

/// Contains verified WhatsApp Business details.
pub struct VerifiedName {
    pub certificate: wa_proto::VerifiedNameCertificate,
    pub details: wa_proto::verified_name_certificate::Details,
}

/// Contains info about a WhatsApp user.
pub struct UserInfo {
    /// Verified WhatsApp Business name, if exists.
    pub verified_name: Option<VerifiedName>,
    /// The about string of the user.
    pub status: String,
    /// The picture id, if exists.
    pub picture_id: String,
    /// The various devices known.
    pub devices: Vec<JID>,
}

pub enum ProfilePictureType {
    /// Full resolution picture
    Image,
    /// A low quality thumbnail
    Preview,
    /// Just as a fallback incase there is any other value
    Value(String),
}

impl FromStr for ProfilePictureType {
    type Err = RhustAppError;

    fn from_str(input: &str) -> Result<Self, RhustAppError> {
        match input {
            "image" => Ok(Self::Image),
            "preview" => Ok(Self::Preview),
            _ => Ok(Self::Value(input.to_string())),
        }
    }
}

/// Contains the ID and URL for a WhatsApp user's profile picture or a group's photo.
pub struct ProfilePictureInfo {
    /// The full URL for the image, can be downloaded with a simple HTTP request.
    pub url: String,
    /// The ID of the image. This is the same as `UserInfo.picture_id`.
    pub id: String,
    /// The type (basically quality) of the profile picture.
    pub r#type: ProfilePictureType,

    /// The path to the image, probably not very useful.
    pub direct_path: String,
}

/// Contains the cached names of a WhatsApp user.
pub struct ContactInfo {
    pub first_name: String,
    pub full_name: String,
    pub push_name: String,
    pub business_name: String,
}

/// Contains the cached local settings for a chat.
pub struct LocalChatSettings {
    pub muted_until: time::PrimitiveDateTime,
    pub pinned: bool,
    pub archived: bool,
}

/// Contains the information received in response to checking if a phone number is
/// registered on WhatsApp.
pub struct IsOnWhatsAppResponse {
    /// The query string used.
    pub query: String,
    /// The canonical user ID.
    pub jid: JID,
    /// Whether the phone is registered or not.
    pub is_in: bool,

    /// If the phone is a business, then the verified business details.
    pub verified_name: Option<VerifiedName>,
}

/// Contains the information that is found using a business message link.
/// TODO: Add link to `ResolveBusinessMessageLink` after implementation.
pub struct BusniessMessageLinkTarget {
    /// The JID of the business.
    pub jid: JID,
    /// The notify / push name of the business.
    pub push_name: String,
    /// The verified name of the business.
    pub verified_name: String,
    /// Some boolean which seems to be true always.
    pub is_signed: bool,
    /// Tulir guesses the level of verification, starting from "unknown".
    pub verified_level: String,

    /// The message that the WhatsApp clients will pre-fill in the input box when clicking
    /// the link.
    pub message: String,
}

/// Contains the information that is found using a contact QR link.
/// TODO: Add link to `ResolveContactQRLink` after implementation.
pub struct ContactQRLinkTarget {
    pub jid: JID,
    pub r#type: String,
    pub push_name: String,
}

/// Possible privacy setting values.
pub enum PrivacySetting {
    Undefined,
    All,
    Contacts,
    None,
}

impl FromStr for PrivacySetting {
    type Err = RhustAppError;

    fn from_str(input: &str) -> Result<Self, RhustAppError> {
        match input {
            "" => Ok(Self::Undefined),
            "all" => Ok(Self::All),
            "contacts" => Ok(Self::Contacts),
            "none" => Ok(Self::None),
            _ => Err(new_rhustapp_error(
                &format!("'{}' did not match any known PrivacySetting", input),
                None,
            )),
        }
    }
}

/// Contains the user's privacy settings.
pub struct PrivacySettings {
    pub group_add: PrivacySetting,
    pub last_seen: PrivacySetting,
    pub status: PrivacySetting,
    pub profile: PrivacySetting,
    pub read_receipts: PrivacySetting,
}

/// Type of list in `StatusPrivacy`
pub enum StatusPrivacyType {
    /// Means statuses are sent to all contacts.
    Contacts,
    /// Means statuses are sent to all contacts, except the one on the list.
    Blacklist,
    /// Means statuses are only sent to users on the list.
    Whitelist,
}

impl FromStr for StatusPrivacyType {
    type Err = RhustAppError;

    fn from_str(input: &str) -> Result<Self, RhustAppError> {
        match input {
            "contacts" => Ok(Self::Contacts),
            "blacklist" => Ok(Self::Blacklist),
            "whitelist" => Ok(Self::Whitelist),
            _ => Err(new_rhustapp_error(
                &format!("'{}' did not match any known StatusPrivacyType", input),
                None,
            )),
        }
    }
}

/// Contains the settings for whom to send status messages to by default.
pub struct StatusPrivacy {
    pub r#type: StatusPrivacyType,
    pub list: Vec<JID>,

    pub is_default: bool,
}
