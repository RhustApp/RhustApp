use time::PrimitiveDateTime;

use super::{VerifiedName, JID};

/// Contains basic sender and chat information about a message.
pub struct MessageSource {
    /// The chat where the message was sent.
    pub chat: JID,
    /// The user who sent the message.
    pub sender: JID,
    /// Whether the message was sent by the current user instead of someone else.
    pub is_from_me: bool,
    /// Whether the chat is a group chat or broadcast list.
    pub is_group: bool,

    /// When sending a read receipt to a broadcast list message, the Chat is the broadcast
    /// list and Sender is you, so this field contains the recipeint of the read receipt.
    pub broadcast_list_owner: Option<JID>,
}

impl MessageSource {
    /// Returns true if the message was sent to a broadcast list instead of directly to
    /// the user.
    pub fn is_incoming_broadcast(&self) -> bool {
        (!self.is_from_me || self.broadcast_list_owner.is_some()) && self.chat.is_broadcast_list()
    }

    /// Returns a log-friendly representation of who sent the message and where.
    pub fn source_string(&self) -> String {
        if self.sender == self.chat {
            self.chat.to_string()
        } else {
            format!("{} in {}", self.sender, self.chat)
        }
    }
}

/// Contains the metadata from messages sent by another one of the user's own devices.
pub struct DeviceSentMeta {
    /// The destination user. This should match the `MessageInfo.recipient` field.
    pub destination_jid: String,
    pub phash: String,
}

/// Contains metadata about an incoming message
pub struct MessageInfo {
    pub id: String,
    pub source: MessageSource,
    pub r#type: String,
    pub timestamp: PrimitiveDateTime,
    pub category: String,
    pub multicast: bool,
    pub media_type: String,

    pub verified_name: Option<VerifiedName>,
    /// Metadata for direct messages sent from another one of the user's own devices.
    pub device_sent_meta: Option<DeviceSentMeta>,
}
