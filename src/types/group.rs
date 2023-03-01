use std::str::FromStr;

use time::PrimitiveDateTime;

use crate::RhustAppError;

use super::JID;

pub enum GroupMemberAddMode {
    /// ("admin_add") If added by the admin.
    AdminAdd,
    /// Just as a fallback incase there is any other value
    Value(String),
}

impl FromStr for GroupMemberAddMode {
    type Err = RhustAppError;

    fn from_str(input: &str) -> Result<Self, RhustAppError> {
        match input {
            "admin_add" => Ok(Self::AdminAdd),
            _ => Ok(Self::Value(input.to_string())),
        }
    }
}

/// Contains basic information about a group chat on WhatsApp.
pub struct GroupInfo {
    pub jid: JID,
    pub owner_jid: JID,

    pub group_name: Option<GroupName>,
    pub group_topic: Option<GroupTopic>,
    pub group_locked: Option<GroupLocked>,
    pub group_announce: Option<GroupAnnounce>,
    pub group_ephemeral: Option<GroupEphemeral>,

    pub group_parent: Option<GroupParent>,
    pub group_linked_parent: Option<GroupLinkedParent>,
    pub group_is_default_sub: Option<GroupIsDefaultSub>,

    pub creation_time: PrimitiveDateTime,

    pub participant_version_id: String,
    pub participants: Vec<GroupParticipant>,

    pub member_add_mode: GroupMemberAddMode,
}

/// Contains information about a participant of a WhatsApp group chat.
pub struct GroupParticipant {
    pub jid: JID,
    pub is_admin: bool,
    pub is_super_admin: bool,

    /// When creating groups, adding some participants may fail.
    /// In such cases, the error code will be here.
    pub error_code: i32,
    pub add_request: Option<GroupParticipantAddRequest>,
}

pub struct GroupParticipantAddRequest {
    pub code: String,
    pub expiration: PrimitiveDateTime,
}

pub enum MembershipApprovalMode {
    /// "request_required"
    RequestRequired,
    Value(String),
}

impl FromStr for MembershipApprovalMode {
    type Err = RhustAppError;

    fn from_str(input: &str) -> Result<Self, RhustAppError> {
        match input {
            "request_required" => Ok(Self::RequestRequired),
            _ => Ok(Self::Value(input.to_string())),
        }
    }
}

pub struct GroupParent {
    pub is_parent: bool,
    /// request_required
    pub default_membership_approval_mode: MembershipApprovalMode,
}

pub struct GroupLinkedParent {
    pub linked_parent_jid: JID,
}

pub struct GroupIsDefaultSub {
    pub is_default_sub_group: bool,
}

/// Contains the name of a group along with metadata of who set it and when.
pub struct GroupName {
    pub name: String,
    pub name_set_at: PrimitiveDateTime,
    pub name_set_by: JID,
}

/// Contains the topic (description) of a group along with metadata of who set it and when.
pub struct GroupTopic {
    pub topic: String,
    pub topic_id: String,
    pub topic_set_at: PrimitiveDateTime,
    pub topic_set_by: JID,
    pub topic_deleted: bool,
}

/// Specifies whether the group information can only be edited by admins.
pub struct GroupLocked {
    pub is_locked: bool,
}

/// Specifies whether only admins can send messages in the group.
pub struct GroupAnnounce {
    pub is_announce: bool,
    pub announce_version_id: String,
}

/// Contains the group's disappearing messages settings.
pub struct GroupEphemeral {
    pub is_ephemeral: bool,
    pub disappearing_timer: u32,
}

pub struct GroupDelete {
    pub deleted: bool,
    pub deleted_reason: String,
}

pub enum GroupLinkChangeType {
    /// "parent_group"
    Parent,
    /// "sub_group"
    Sub,
    /// "sibling_group"
    Sibling,
    Value(String),
}

impl FromStr for GroupLinkChangeType {
    type Err = RhustAppError;

    fn from_str(input: &str) -> Result<Self, RhustAppError> {
        match input {
            "parent_group" => Ok(Self::Parent),
            "sub_group" => Ok(Self::Sub),
            "sibling_group" => Ok(Self::Sibling),
            _ => Ok(Self::Value(input.to_string())),
        }
    }
}

pub enum GroupUnlinkReason {
    /// "unlink_group"
    UnlinkGroup,
    /// "delete_parent"
    DeleteParent,
    Value(String),
}

impl FromStr for GroupUnlinkReason {
    type Err = RhustAppError;

    fn from_str(input: &str) -> Result<Self, RhustAppError> {
        match input {
            "unlink_group" => Ok(Self::UnlinkGroup),
            "delete_parent" => Ok(Self::DeleteParent),
            _ => Ok(Self::Value(input.to_string())),
        }
    }
}

pub struct GroupLinkTarget {
    pub jid: JID,
    pub group_name: GroupName,
    pub group_is_default_sub: GroupIsDefaultSub,
}

pub struct GroupLinkChange {
    pub r#type: GroupLinkChangeType,
    pub unlink_reason: GroupUnlinkReason,
}
