use super::JID;

/// This contains the basic common metadata about different call events.
pub struct BasicCallMetadata {
    /// This is the chat (user/group) in which the call was created.
    pub from: JID,
    /// This is the timestamp at which the event started.
    pub timestamp: time::OffsetDateTime,
    /// This is the user who initiated the call.
    pub call_creator: JID,
    /// The unique ID for a call event.
    pub call_id: String,
}

/// This contains the metadata about the caller's WhatsApp client
pub struct CallRemoteMetadata {
    /// The platform of the caller's client
    pub remote_platform: String,
    /// The version of the caller's client
    pub remote_version: String,
}
