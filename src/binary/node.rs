use std::{collections::HashMap, io::Read};

use time::OffsetDateTime;

use crate::{
    new_rhustapp_error,
    types::{EMPTY_JID, JID},
    RhustAppError,
};

use super::token;

/// The various types of content inside an XML element.
#[derive(Clone, Debug, Default)]
pub enum NodeContentType {
    #[default]
    None,
    ListOfNodes(Vec<Node>),
    ByteArray(Vec<u8>),

    // While encoding
    JID(JID),
    String(String),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    Bool(bool),
}

impl NodeContentType {
    pub fn other_types_to_string(&self) -> String {
        match self {
            Self::None | Self::ListOfNodes(_) | Self::ByteArray(_) => String::new(),
            Self::JID(v) => format!("{}", v.to_string()),
            Self::String(v) => format!("{}", v.to_string()),
            Self::I32(v) => format!("{}", v.to_string()),
            Self::U32(v) => format!("{}", v.to_string()),
            Self::I64(v) => format!("{}", v.to_string()),
            Self::U64(v) => format!("{}", v.to_string()),
            Self::Bool(v) => format!("{}", v.to_string()),
        }
    }
}

/// It represents an XML element.
#[derive(Clone, Debug, Default)]
pub struct Node {
    /// The tag of the element.
    pub tag: String,
    /// The attributes of the element.
    pub attrs: Attrs,
    /// The content inside the element. Can be `None`, a list of `Node` or a byte array.
    pub content: NodeContentType,
}

impl Node {
    pub const INDENT_XML: bool = false;
    pub const MAX_BYTES_TO_PRINT_AS_HEX: usize = 128;

    /// Returns the `content` of the `Node` as a list of nodes if they exist.
    pub fn get_children(&self) -> Option<Vec<Node>> {
        match &self.content {
            NodeContentType::ListOfNodes(nodes) => Some(nodes.to_vec()),
            _ => None,
        }
    }

    /// Returns the same list as `self.get_children`, but filters it by tag first.
    pub fn get_children_by_tag(&self, tag: &str) -> Option<Vec<Node>> {
        match self.get_children() {
            Some(nodes) => Some(
                nodes
                    .iter()
                    .filter(|node| node.tag.eq(tag))
                    .map(|node| node.to_owned())
                    .collect::<Vec<Node>>(),
            ),
            None => None,
        }
    }

    /// Finds the first child with the given tag and returns it.
    // Each provided tag will recurse in, so this is useful for getting a specific nested element.
    pub fn get_optional_child_by_tag(&self, tags: &[&str]) -> Option<Node> {
        let mut final_child = self.to_owned();
        let mut children = self.get_children();

        'outer_loop: for tag in tags {
            if children.is_none() {
                return None;
            }
            for child in children.unwrap() {
                if child.tag.eq(tag) {
                    final_child = child.to_owned();
                    children = child.get_children();
                    continue 'outer_loop;
                }
            }
            return None;
        }

        return Some(final_child);
    }

    pub fn attr_getter(&self) -> AttrUtility {
        AttrUtility {
            attrs: &self.attrs,
            errors: vec![],
        }
    }

    pub fn xml_string(&self) -> String {
        let attributes = self.attribute_string();
        let content = self.content_string();
        if content.is_empty() {
            return format!("<{tag} {attrs} />", tag = self.tag, attrs = attributes);
        };

        let new_line: String;
        if content.len() == 1 || !Self::INDENT_XML {
            new_line = String::new();
        } else {
            new_line = String::from("\n");
        };

        format!(
            "<{0} {1}>{3}{2}{3}</{0}>",
            self.tag,
            attributes,
            content.join(&new_line),
            new_line,
        )
    }

    pub fn content_string(&self) -> Vec<String> {
        let mut content_vec: Vec<String> = Vec::new();

        match &self.content {
            NodeContentType::None => {}
            NodeContentType::ListOfNodes(nodes) => {
                for node in nodes.iter() {
                    content_vec.append(
                        &mut node
                            .xml_string()
                            .split("\n")
                            .map(|s| s.to_owned())
                            .collect(),
                    );
                }
            }
            NodeContentType::ByteArray(bytes) => {
                let content = printable(bytes);
                if !content.is_empty() {
                    if Self::INDENT_XML {
                        content_vec
                            .append(&mut content.split("\n").map(|s| s.to_owned()).collect());
                    } else {
                        content_vec.push(content.replace("\n", "\\n"));
                    }
                } else if content.len() > Self::MAX_BYTES_TO_PRINT_AS_HEX {
                    content_vec.push(format!("<!-- {} bytes -->", content.len()));
                } else if !Self::INDENT_XML {
                    content_vec.push(hex::encode(content));
                } else {
                    let hex_data = hex::encode(content);
                    let mut i = 0;
                    while i < hex_data.len() {
                        if hex_data.len() < i + 80 {
                            content_vec.push(String::from(&hex_data.as_str()[i..]));
                        } else {
                            content_vec.push(String::from(&hex_data.as_str()[i..i + 80]));
                        }
                        i += 80;
                    }
                }
            }
            c => {
                let content = c.other_types_to_string();
                if Self::INDENT_XML {
                    content_vec.append(&mut content.split("\n").map(|s| s.to_string()).collect());
                } else {
                    content_vec.push(content.replace("\n", "\\n"));
                }
            }
        }

        if content_vec.len() > 1 && Self::INDENT_XML {
            content_vec = content_vec.iter().map(|s| format!(" {s}")).collect();
        }

        content_vec
    }

    pub fn attribute_string(&self) -> String {
        if self.attrs.is_empty() {
            return String::new();
        };

        let mut string_attrs: Vec<String> = Vec::with_capacity(self.attrs.len());
        for (key, value) in self.attrs.iter() {
            string_attrs.push(format!("{}=\"{}\"", key, value.to_string()));
        }
        string_attrs.sort();
        string_attrs.join(" ")
    }
}

/// It contains all the types for the attributes of an XML element (`Node`).
#[derive(Clone, Debug)]
pub enum AttributeTypes {
    JID(JID),
    String(String),
}

impl AttributeTypes {
    pub fn to_string(&self) -> String {
        match self {
            Self::String(s) => s.to_string(),
            Self::JID(j) => j.to_string(),
        }
    }
}

pub type Attrs = HashMap<String, AttributeTypes>;

pub struct AttrUtility<'a> {
    pub attrs: &'a Attrs,
    pub errors: Vec<RhustAppError>,
}

impl AttrUtility<'_> {
    fn get_jid(&mut self, key: &str, required: bool) -> Option<JID> {
        match self.attrs.get(key) {
            Some(val) => match val {
                AttributeTypes::JID(jid) => {
                    return Some(jid.to_owned());
                }
                AttributeTypes::String(_) => {
                    if required {
                        self.errors.push(new_rhustapp_error(
                            &format!("expected attribute '{key}' to be JID, but was String"),
                            None,
                        ));
                    };
                    return None;
                }
            },
            None => {
                if required {
                    self.errors.push(new_rhustapp_error(
                        &format!("didn't find required JID attribute '{key}'"),
                        None,
                    ));
                };
                return None;
            }
        }
    }

    /// Returns the JID under the given key. If there's no valid JID under the given key,
    /// this will return `None`.
    /// However, if the attribute is completely missing, this will not store an error.
    pub fn optional_jid(&mut self, key: &str) -> Option<JID> {
        self.get_jid(key, false)
    }

    /// Returns the JID under the given key. If there is no valid JID under the given key,
    /// this will return an empty JID.
    /// However, if the attribute is completely missing, this will not store an error.
    pub fn optional_jid_or_empty(&mut self, key: &str) -> JID {
        self.get_jid(key, false).unwrap_or(EMPTY_JID.clone())
    }

    /// Returns the JID under the given key.
    /// If there's no valid JID under the given key, an error will be stored and None
    /// will be returned.
    pub fn jid(&mut self, key: &str) -> Option<JID> {
        self.get_jid(key, true)
    }

    fn get_string(&mut self, key: &str, required: bool) -> Option<String> {
        match self.attrs.get(key) {
            Some(val) => match val {
                AttributeTypes::String(s) => {
                    return Some(s.to_owned());
                }
                AttributeTypes::JID(_) => {
                    if required {
                        self.errors.push(new_rhustapp_error(
                            &format!("expected attribute '{key}' to be String, but was JID"),
                            None,
                        ));
                    };
                    return None;
                }
            },
            None => {
                if required {
                    self.errors.push(new_rhustapp_error(
                        &format!("didn't find required String attribute '{key}'"),
                        None,
                    ));
                };
                return None;
            }
        }
    }

    pub fn optional_string(&mut self, key: &str) -> Option<String> {
        self.get_string(key, false)
    }

    /// Returns the string under the given key.
    /// If there's no valid string under the given key, an error will be stored and an
    /// empty string will be returned.
    pub fn string(&mut self, key: &str) -> Option<String> {
        self.get_string(key, true)
    }

    fn get_i64(&mut self, key: &str, required: bool) -> Option<i64> {
        if let Some(s) = self.get_string(key, required) {
            match s.parse::<i64>() {
                Ok(val) => Some(val),
                Err(err) => {
                    if required {
                        self.errors.push(new_rhustapp_error(
                            &format!("failed to parse i64 in attribute '{key}'"),
                            Some(err.to_string()),
                        ));
                    };
                    return None;
                }
            }
        } else {
            None
        }
    }

    pub fn i64(&mut self, key: &str) -> Option<i64> {
        self.get_i64(key, true)
    }

    fn get_u64(&mut self, key: &str, required: bool) -> Option<u64> {
        if let Some(s) = self.get_string(key, required) {
            match s.parse::<u64>() {
                Ok(val) => Some(val),
                Err(err) => {
                    if required {
                        self.errors.push(new_rhustapp_error(
                            &format!("failed to parse u64 in attribute '{key}'"),
                            Some(err.to_string()),
                        ));
                    };
                    return None;
                }
            }
        } else {
            None
        }
    }

    pub fn u64(&mut self, key: &str) -> Option<u64> {
        self.get_u64(key, true)
    }

    fn get_bool(&mut self, key: &str, required: bool) -> Option<bool> {
        if let Some(s) = self.get_string(key, required) {
            match s.as_str() {
                "1" | "t" | "T" | "true" | "TRUE" | "True" => Some(true),
                "0" | "f" | "F" | "false" | "FALSE" | "False" => Some(false),
                _ => {
                    if required {
                        self.errors.push(new_rhustapp_error(
                            &format!("failed to parse bool in attribute '{key}'"),
                            None,
                        ));
                    };
                    return None;
                }
            }
        } else {
            None
        }
    }

    pub fn optional_bool(&mut self, key: &str) -> Option<bool> {
        self.get_bool(key, false)
    }

    pub fn bool(&mut self, key: &str) -> Option<bool> {
        self.get_bool(key, true)
    }

    fn get_unix_time(&mut self, key: &str, required: bool) -> Option<OffsetDateTime> {
        if let Some(ts) = self.get_i64(key, required) {
            if ts == 0 {
                return Some(OffsetDateTime::UNIX_EPOCH.clone());
            };
            match OffsetDateTime::from_unix_timestamp(ts) {
                Ok(offset_dt) => Some(offset_dt),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn optional_unix_time(&mut self, key: &str) -> Option<OffsetDateTime> {
        self.get_unix_time(key, false)
    }

    pub fn unix_time(&mut self, key: &str) -> Option<OffsetDateTime> {
        self.get_unix_time(key, true)
    }

    pub fn optional_i32(&mut self, key: &str) -> Option<i32> {
        match self.get_i64(key, false) {
            Some(i) => Some(i as i32),
            None => None,
        }
    }

    pub fn i32(&mut self, key: &str) -> Option<i32> {
        match self.get_i64(key, true) {
            Some(i) => Some(i as i32),
            None => None,
        }
    }

    /// Returns true if there are no errors.
    pub fn ok(&self) -> bool {
        self.errors.len() == 0
    }

    /// Returns the list of errors as a single error
    pub fn error(&self) -> Option<RhustAppError> {
        if self.ok() {
            None
        } else {
            let mut error_string = String::from("[");

            for e in &self.errors {
                error_string = format!("{error_string} {},", e.to_string())
            }
            error_string = format!("{} ]", &error_string[..error_string.len() - 1]);

            Some(new_rhustapp_error(
                "error(s) occured while fetching attributes",
                Some(error_string.to_string()),
            ))
        }
    }
}

/// Errors returned by the binary XML decoder.
pub enum DecoderError {
    ErrInvalidType,
    ErrInvalidJIDType,
    ErrInvalidNode,
    ErrInvalidToken,
    ErrNonStringKey,
}

impl DecoderError {
    pub fn to_string(&self) -> String {
        match self {
            Self::ErrInvalidType => String::from("unsupported payload type"),
            Self::ErrInvalidJIDType => String::from("invalid JID type"),
            Self::ErrInvalidNode => String::from("invalid node"),
            Self::ErrInvalidToken => String::from("invalid token with tag"),
            Self::ErrNonStringKey => String::from("non-string key"),
        }
    }
}

impl std::fmt::Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Default)]
pub struct BinaryEncoder {
    data: Vec<u8>,
}

impl BinaryEncoder {
    const TAG_SIZE: i32 = 1;

    pub fn new() -> Self {
        let mut enc = Self::default();
        enc.push_byte(0);
        enc
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn push_byte(&mut self, byte: u8) {
        self.data.push(byte)
    }

    pub fn push_bytes(&mut self, bytes: &mut Vec<u8>) {
        self.data.append(bytes)
    }

    pub fn push_i_n(&mut self, value: i32, n: i32, little_endian: bool) {
        for i in 0..n {
            let current_shift: i32;
            if little_endian {
                current_shift = i;
            } else {
                current_shift = n - i - 1;
            }
            self.push_byte(((value >> (current_shift * 8)) & 0xFF) as u8);
        }
    }

    pub fn push_i_20(&mut self, value: i32) {
        self.push_bytes(&mut vec![
            ((value >> 16) & 0x0F) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8,
        ])
    }

    pub fn push_i_8(&mut self, value: i32) {
        self.push_i_n(value, 1, false)
    }

    pub fn push_i_16(&mut self, value: i32) {
        self.push_i_n(value, 2, false)
    }

    pub fn push_i_32(&mut self, value: i32) {
        self.push_i_n(value, 4, false)
    }

    pub fn push_string(&mut self, value: &str) {
        self.push_bytes(&mut value.clone().as_bytes().to_vec())
    }

    pub fn write_byte_length(&mut self, length: usize) {
        if length < 256 {
            self.push_byte(token::BINARY8);
            self.push_i_8(length as i32);
        } else if length < (1 << 20) {
            self.push_byte(token::BINARY20);
            self.push_i_20(length as i32);
        } else if (length as i32) < std::i32::MAX {
            self.push_byte(token::BINARY32);
            self.push_i_32(length as i32);
        } else {
            panic!(
                "{}",
                new_rhustapp_error(&format!("length is too large: {length}"), None)
            )
        }
    }

    pub fn write_node(&mut self, n: &Node) {
        if n.tag.eq("0") {
            self.push_byte(token::LIST8);
            self.push_byte(token::LIST_EMPTY);
            return;
        };

        let has_content: i32;
        match n.content {
            NodeContentType::None => {
                has_content = 0;
            }
            _ => {
                has_content = 1;
            }
        }

        self.write_list_start((2 * n.attrs.len() as i32) + Self::TAG_SIZE + has_content);
        self.write_string(&n.tag);
        self.write_attributes(&n.attrs);
        if has_content == 1 {
            self.write(&n.content);
        }
    }

    pub fn write(&mut self, data: &NodeContentType) {
        match data {
            NodeContentType::None => self.push_byte(token::LIST_EMPTY),
            NodeContentType::JID(j) => self.write_jid(j),
            NodeContentType::String(s) => self.write_string(s),
            NodeContentType::I32(i) => self.write_string(&format!("{i}")),
            NodeContentType::U32(u) => self.write_string(&format!("{u}")),
            NodeContentType::I64(i) => self.write_string(&format!("{i}")),
            NodeContentType::U64(u) => self.write_string(&format!("{u}")),
            NodeContentType::Bool(b) => self.write_string(&format!("{b}")),
            NodeContentType::ByteArray(b) => self.write_bytes(b),
            NodeContentType::ListOfNodes(l) => {
                self.write_list_start(l.len() as i32);
                for n in l.iter() {
                    self.write_node(n);
                }
            }
        }
    }

    pub fn write_string(&mut self, data: &str) {
        if let Some(token_index) = token::index_of_single_token(data) {
            self.push_byte(token_index);
        } else if let Some((dict_index, token_index)) = token::index_of_double_token(data) {
            self.push_byte(token::DICTIONARY0 + dict_index);
            self.push_byte(token_index);
        } else if BinaryEncoder::validate_nibble(data) {
            self.write_packed_bytes(data, token::NIBBLE8);
        } else if BinaryEncoder::validate_hex(data) {
            self.write_packed_bytes(data, token::HEX8);
        } else {
            self.write_string_raw(data);
        }
    }

    pub fn write_bytes(&mut self, data: &Vec<u8>) {
        self.write_byte_length(data.len());
        self.push_bytes(&mut data.clone());
    }

    pub fn write_string_raw(&mut self, data: &str) {
        self.write_byte_length(data.len());
        self.push_string(data);
    }

    pub fn write_jid(&mut self, jid: &JID) {
        if jid.is_ad() {
            self.push_byte(token::ADJID);
            self.push_byte(jid.agent.unwrap());
            self.push_byte(jid.device.unwrap());
            self.write_string(&jid.user);
        } else {
            self.push_byte(token::JID_PAIR);
            if jid.user.len() == 0 {
                self.push_byte(token::LIST_EMPTY);
            } else {
                self.write(&NodeContentType::String(jid.user.to_string()));
            }
            self.write(&NodeContentType::String(jid.user.to_string()));
        }
    }

    pub fn write_attributes(&mut self, attributes: &Attrs) {
        for (key, value) in attributes.iter() {
            match value {
                AttributeTypes::String(s) => {
                    if !s.is_empty() {
                        self.write_string(key);
                        self.write(&NodeContentType::String(s.to_string()));
                    }
                }
                AttributeTypes::JID(j) => {
                    self.write_string(key);
                    self.write(&NodeContentType::JID(j.to_owned()));
                }
            }
        }
    }

    pub fn write_list_start(&mut self, list_size: i32) {
        if list_size == 0 {
            self.push_byte(token::LIST_EMPTY);
        } else if list_size < 256 {
            self.push_byte(token::LIST8);
            self.push_i_8(list_size);
        } else {
            self.push_byte(token::LIST16);
            self.push_i_16(list_size);
        }
    }

    pub fn write_packed_bytes(&mut self, value: &str, data_type: u8) {
        if value.len() > token::PACKED_MAX {
            panic!(
                "{}",
                new_rhustapp_error(&format!("too many bytes to pack: {}", value.len()), None)
            )
        }
        self.push_byte(data_type);
        let mut rounded_length = f64::ceil((value.len() as f64) / 2.0) as u8;
        if value.len() % 2 != 0 {
            rounded_length |= 128;
        }
        self.push_byte(rounded_length);

        let packer: fn(u8) -> u8;
        match data_type {
            token::NIBBLE8 => packer = BinaryEncoder::pack_nibble,
            token::HEX8 => packer = BinaryEncoder::pack_hex,
            _ => {
                panic!("{}", &format!("invalid packed byte data type: {data_type}"));
            }
        }

        for i in 0..(value.len() / 2) {
            let packed_byte = BinaryEncoder::pack_byte_pair(
                packer,
                value.chars().nth(2 * i).unwrap() as u8,
                value.chars().nth(2 * i + 1).unwrap() as u8,
            );
            self.push_byte(packed_byte);
        }
        if value.len() % 2 != 0 {
            let packed_byte = BinaryEncoder::pack_byte_pair(
                packer,
                value.chars().nth(value.len() - 1).unwrap() as u8,
                b'\x00',
            );
            self.push_byte(packed_byte);
        }
    }

    pub fn pack_byte_pair(packer: fn(u8) -> u8, part_1: u8, part_2: u8) -> u8 {
        (packer(part_1) << 4) | packer(part_2)
    }

    pub fn validate_nibble(value: &str) -> bool {
        if value.len() > token::PACKED_MAX {
            return false;
        };

        for c in value.chars() {
            if !(c >= '0' && c <= '9') && c != '-' && c != '.' {
                return false;
            }
        }
        true
    }

    pub fn pack_nibble(value: u8) -> u8 {
        match value {
            b'-' => 10,
            b'.' => 11,
            0 => 15,
            _ => {
                if value >= b'0' && value <= b'9' {
                    return value - b'0';
                };
                panic!(
                    "{}",
                    new_rhustapp_error(
                        &format!(
                            "invalid string to pack as nibble: {} / '{}'",
                            value,
                            value.to_string()
                        ),
                        None
                    )
                )
            }
        }
    }

    pub fn validate_hex(value: &str) -> bool {
        if value.len() > token::PACKED_MAX {
            return false;
        };
        for c in value.chars() {
            if !(c >= '0' && c <= '9') && !(c >= 'A' && c <= 'F') && !(c >= 'a' && c <= 'f') {
                return false;
            }
        }
        true
    }

    pub fn pack_hex(value: u8) -> u8 {
        match value {
            v if (v >= b'0' && v <= b'9') => v - b'0',
            v if (v >= b'A' && v <= b'F') => 10 + v - b'A',
            v if (v >= b'a' && v <= b'f') => 10 + v - b'a',
            0 => 15,
            _ => {
                panic!(
                    "{}",
                    new_rhustapp_error(
                        &format!(
                            "invalid string to pack as hex: {} / '{}'",
                            value,
                            value.to_string()
                        ),
                        None
                    )
                )
            }
        }
    }
}

#[derive(Default)]
pub struct BinaryDecoder {
    data: Vec<u8>,
    index: usize,
}

impl BinaryDecoder {
    pub fn new(data: &Vec<u8>) -> Self {
        let mut dec = Self::default();
        dec.data = data.clone();
        dec
    }

    pub fn check_eos(&self, length: usize) -> Result<(), RhustAppError> {
        if self.index + length > self.data.len() {
            return Err(new_rhustapp_error("EOF", None));
        };
        Ok(())
    }

    pub fn read_byte(&mut self) -> Result<u8, RhustAppError> {
        self.check_eos(1)
            .map_err(|err| new_rhustapp_error("could not read a byte", Some(err.to_string())))?;

        let b = self.data[self.index];
        self.index += 1;

        Ok(b)
    }

    pub fn read_i_n(&mut self, n: usize, little_endian: bool) -> Result<i32, RhustAppError> {
        self.check_eos(n).map_err(|err| {
            new_rhustapp_error(&format!("could not read i_{n}"), Some(err.to_string()))
        })?;

        let mut return_value: i32 = 0;

        for i in 0..n {
            let current_shift: usize;
            if little_endian {
                current_shift = i;
            } else {
                current_shift = n - i - 1;
            }
            return_value |= (self.data[self.index + i] as i32) << current_shift * 8;
        }

        self.index += n as usize;
        Ok(return_value)
    }

    pub fn read_i_8(&mut self, little_endian: bool) -> Result<i32, RhustAppError> {
        self.read_i_n(1, little_endian)
    }

    pub fn read_i_16(&mut self, little_endian: bool) -> Result<i32, RhustAppError> {
        self.read_i_n(2, little_endian)
    }

    pub fn read_i_20(&mut self) -> Result<i32, RhustAppError> {
        self.check_eos(3).map_err(|err| {
            new_rhustapp_error(&format!("could not read i_20"), Some(err.to_string()))
        })?;

        let return_value: i32 = (((self.data[self.index] as i32) & 15) << 16)
            + ((self.data[self.index + 1] as i32) << 8)
            + (self.data[self.index + 2] as i32);

        self.index += 3;
        Ok(return_value)
    }

    pub fn read_i_32(&mut self, little_endian: bool) -> Result<i32, RhustAppError> {
        self.read_i_n(4, little_endian)
    }

    pub fn read_packed_8(&mut self, tag: u8) -> Result<String, RhustAppError> {
        let start_byte = self.read_byte().map_err(|err| {
            new_rhustapp_error("failed to read packed 8 string", Some(err.to_string()))
        })?;

        let mut bytes = Vec::<u8>::default();

        for _ in 0..(start_byte & 127) {
            let curr_byte = self.read_byte().map_err(|err| {
                new_rhustapp_error("failed to read packed 8 string", Some(err.to_string()))
            })?;

            let lower =
                BinaryDecoder::unpack_byte(tag, (curr_byte & 0xF0) >> 4).map_err(|err| {
                    new_rhustapp_error("failed to read packed 8 string", Some(err.to_string()))
                })?;
            let upper = BinaryDecoder::unpack_byte(tag, curr_byte & 0x0F).map_err(|err| {
                new_rhustapp_error("failed to read packed 8 string", Some(err.to_string()))
            })?;

            bytes.push(lower);
            bytes.push(upper);
        }

        let mut ret = String::from_utf8(bytes).map_err(|err| {
            new_rhustapp_error("failed to read packed 8 string", Some(err.to_string()))
        })?;

        if start_byte >> 7 != 0 {
            ret = ret[..ret.len() - 1].to_string();
        };

        Ok(ret)
    }

    pub fn unpack_byte(tag: u8, value: u8) -> Result<u8, RhustAppError> {
        match tag {
            token::NIBBLE8 => BinaryDecoder::unpack_nibble(value),
            token::HEX8 => BinaryDecoder::unpack_hex(value),
            _ => Err(new_rhustapp_error(
                &format!("unpack_byte with unknown tag: {tag}"),
                None,
            )),
        }
    }

    pub fn unpack_nibble(value: u8) -> Result<u8, RhustAppError> {
        match value {
            v if v < 10 => Ok(b'0' + v),
            10 => Ok(b'-'),
            11 => Ok(b'.'),
            15 => Ok(0),
            _ => Err(new_rhustapp_error(
                &format!("unpack_nibble with value: {value}"),
                None,
            )),
        }
    }

    pub fn unpack_hex(value: u8) -> Result<u8, RhustAppError> {
        match value {
            v if v < 10 => Ok(b'0' + v),
            v if v < 16 => Ok(b'A' + v - 10),
            _ => Err(new_rhustapp_error(
                &format!("unpack_hex with value: {value}"),
                None,
            )),
        }
    }

    pub fn read_list_size(&mut self, tag: u8) -> Result<i32, RhustAppError> {
        match tag {
            token::LIST_EMPTY => Ok(0),
            token::LIST8 => self.read_i_8(false),
            token::LIST16 => self.read_i_16(false),
            _ => Err(new_rhustapp_error(
                &format!(
                    "read_list_size with unknown tag {tag} at position {}",
                    self.index
                ),
                None,
            )),
        }
    }

    pub fn read(&mut self, as_string: bool) -> Result<NodeContentType, RhustAppError> {
        let tag_byte = self
            .read_byte()
            .map_err(|err| new_rhustapp_error("failed to read tag byte", Some(err.to_string())))?;

        match tag_byte {
            token::LIST_EMPTY => Ok(NodeContentType::None),
            token::LIST8 | token::LIST16 => self
                .read_list(tag_byte)
                .map(|val| NodeContentType::ListOfNodes(val))
                .map_err(|err| {
                    new_rhustapp_error("failed to parse list tokens", Some(err.to_string()))
                }),
            token::BINARY8 => {
                let size = self.read_i_8(false).map_err(|err| {
                    new_rhustapp_error("failed to parse token::BINARY8", Some(err.to_string()))
                })?;
                let bytes = self.read_bytes(size as usize).map_err(|err| {
                    new_rhustapp_error("failed to parse token::BINARY8", Some(err.to_string()))
                })?;
                if as_string {
                    let s = String::from_utf8(bytes).map_err(|err| {
                        new_rhustapp_error(
                            "failed to convert bytes to String",
                            Some(err.to_string()),
                        )
                    })?;
                    return Ok(NodeContentType::String(s));
                } else {
                    return Ok(NodeContentType::ByteArray(bytes));
                }
            }
            token::BINARY20 => {
                let size = self.read_i_20().map_err(|err| {
                    new_rhustapp_error("failed to parse token::BINARY20", Some(err.to_string()))
                })?;
                let bytes = self.read_bytes(size as usize).map_err(|err| {
                    new_rhustapp_error("failed to parse token::BINARY20", Some(err.to_string()))
                })?;
                if as_string {
                    let s = String::from_utf8(bytes).map_err(|err| {
                        new_rhustapp_error(
                            "failed to convert bytes to String",
                            Some(err.to_string()),
                        )
                    })?;
                    return Ok(NodeContentType::String(s));
                } else {
                    return Ok(NodeContentType::ByteArray(bytes));
                }
            }
            token::BINARY32 => {
                let size = self.read_i_32(false).map_err(|err| {
                    new_rhustapp_error("failed to parse token::BINARY32", Some(err.to_string()))
                })?;
                let bytes = self.read_bytes(size as usize).map_err(|err| {
                    new_rhustapp_error("failed to parse token::BINARY32", Some(err.to_string()))
                })?;
                if as_string {
                    let s = String::from_utf8(bytes).map_err(|err| {
                        new_rhustapp_error(
                            "failed to convert bytes to String",
                            Some(err.to_string()),
                        )
                    })?;
                    return Ok(NodeContentType::String(s));
                } else {
                    return Ok(NodeContentType::ByteArray(bytes));
                }
            }
            token::DICTIONARY0 | token::DICTIONARY1 | token::DICTIONARY2 | token::DICTIONARY3 => {
                let i = self.read_i_8(false).map_err(|err| {
                    new_rhustapp_error(
                        "failed to parse double byte tokens dictionary tag",
                        Some(err.to_string()),
                    )
                })?;
                return token::get_double_token(tag_byte - token::DICTIONARY0, i as u8)
                    .map(|val| NodeContentType::String(val))
                    .map_err(|err| {
                        new_rhustapp_error(
                            "failed to parse double byte tokens dictionary tag",
                            Some(err.to_string()),
                        )
                    });
            }
            token::JID_PAIR => self
                .read_jid_pair()
                .map(|val| NodeContentType::JID(val))
                .map_err(|err| {
                    new_rhustapp_error("failed to parse token::JID_PAIR", Some(err.to_string()))
                }),
            token::ADJID => self
                .read_ad_jid()
                .map(|val| NodeContentType::JID(val))
                .map_err(|err| {
                    new_rhustapp_error("failed to parse token::ADJID", Some(err.to_string()))
                }),
            token::NIBBLE8 | token::HEX8 => self
                .read_packed_8(tag_byte)
                .map(|val| NodeContentType::String(val))
                .map_err(|err| {
                    new_rhustapp_error(
                        "failed to parse token::NIBBLE8 or token::HEX8",
                        Some(err.to_string()),
                    )
                }),
            _ => {
                if tag_byte >= 1 && (tag_byte as usize) < token::SINGLE_BYTE_TOKENS.len() {
                    return token::get_single_token(tag_byte)
                        .map(|val| NodeContentType::String(val))
                        .map_err(|err| {
                            new_rhustapp_error(
                                "failed to parse default case",
                                Some(err.to_string()),
                            )
                        });
                };
                return Err(new_rhustapp_error(
                    &format!("{} at position {}", tag_byte as i32, self.index),
                    Some(DecoderError::ErrInvalidToken.to_string()),
                ));
            }
        }
    }

    pub fn read_jid_pair(&mut self) -> Result<JID, RhustAppError> {
        let user = self
            .read(true)
            .map_err(|err| new_rhustapp_error("failed to read jid pair", Some(err.to_string())))?;
        let server = self
            .read(true)
            .map_err(|err| new_rhustapp_error("failed to read jid pair", Some(err.to_string())))?;

        match server {
            NodeContentType::String(s) => match user {
                NodeContentType::None => Ok(JID::new("", &s)),
                NodeContentType::String(u) => Ok(JID::new(&u, &s)),
                _ => Err(new_rhustapp_error(
                    "failed to read jid pair",
                    Some(DecoderError::ErrInvalidJIDType.to_string()),
                )),
            },
            _ => Err(new_rhustapp_error(
                "failed to read jid pair",
                Some(DecoderError::ErrInvalidJIDType.to_string()),
            )),
        }
    }

    pub fn read_ad_jid(&mut self) -> Result<JID, RhustAppError> {
        let agent = self
            .read_byte()
            .map_err(|err| new_rhustapp_error("failed to read ad jid", Some(err.to_string())))?;
        let device = self
            .read_byte()
            .map_err(|err| new_rhustapp_error("failed to read ad jid", Some(err.to_string())))?;
        let user = self
            .read(true)
            .map_err(|err| new_rhustapp_error("failed to read ad jid", Some(err.to_string())))?;

        match user {
            NodeContentType::String(u) => Ok(JID::new_ad(&u, agent, device)),
            _ => Err(new_rhustapp_error(
                "failed to read ad jid",
                Some(DecoderError::ErrInvalidJIDType.to_string()),
            )),
        }
    }

    pub fn read_attributes(&mut self, n: i32) -> Result<Attrs, RhustAppError> {
        if n == 0 {
            return Ok(Attrs::new());
        };

        let mut attrs = Attrs::new();
        for _ in 0..n {
            let key_ifc = self.read(true).map_err(|err| {
                new_rhustapp_error("failed to read attributes", Some(err.to_string()))
            })?;

            match key_ifc {
                NodeContentType::String(key) => {
                    let value = self.read(true).map_err(|err| {
                        new_rhustapp_error("failed to read attributes", Some(err.to_string()))
                    })?;
                    match value {
                        NodeContentType::JID(j) => {
                            attrs.insert(key, AttributeTypes::JID(j));
                        }
                        NodeContentType::String(s) => {
                            attrs.insert(key, AttributeTypes::String(s));
                        }
                        _ => {
                            return Err(new_rhustapp_error(
                                "failed to read attributes",
                                Some(format!(
                                    "value is of invalid type at position {index} for key {key}: {value:?}",
                                    index = self.index,
                                )),
                            ))
                        }
                    }
                }
                _ => {
                    return Err(new_rhustapp_error(
                        "failed to read attributes",
                        Some(format!(
                            "'{err}' at position {index} ({key_ifc:?})",
                            err = DecoderError::ErrNonStringKey.to_string(),
                            index = self.index,
                        )),
                    ));
                }
            }
        }

        Ok(attrs)
    }

    pub fn read_list(&mut self, tag: u8) -> Result<Vec<Node>, RhustAppError> {
        let size = self
            .read_list_size(tag)
            .map_err(|err| new_rhustapp_error("failed to read node list", Some(err.to_string())))?;

        let mut nodes = Vec::<Node>::with_capacity(size as usize);

        for _ in 0..size {
            let node = self.read_node().map_err(|err| {
                new_rhustapp_error("failed to read node list", Some(err.to_string()))
            })?;
            nodes.push(node)
        }

        Ok(nodes)
    }

    pub fn read_node(&mut self) -> Result<Node, RhustAppError> {
        let mut node = Node::default();

        let size = self
            .read_i_8(false)
            .map_err(|err| new_rhustapp_error("failed to read node", Some(err.to_string())))?;

        let list_size = self
            .read_list_size(size as u8)
            .map_err(|err| new_rhustapp_error("failed to read node", Some(err.to_string())))?;
        if list_size == 0 {
            return Err(new_rhustapp_error(
                "failed to read node",
                Some(DecoderError::ErrInvalidNode.to_string()),
            ));
        };

        let raw_description = self
            .read(true)
            .map_err(|err| new_rhustapp_error("failed to read node", Some(err.to_string())))?;

        match raw_description {
            NodeContentType::String(s) => {
                if s.is_empty() {
                    return Err(new_rhustapp_error(
                        "failed to read node",
                        Some(DecoderError::ErrInvalidNode.to_string()),
                    ));
                };
                node.tag = s.to_string();

                let attributes = self.read_attributes((list_size - 1) >> 1).map_err(|err| {
                    new_rhustapp_error("failed to read node", Some(err.to_string()))
                })?;
                node.attrs = attributes;

                if list_size % 2 == 1 {
                    return Ok(node);
                };

                let content = self.read(false).map_err(|err| {
                    new_rhustapp_error("failed to read node", Some(err.to_string()))
                })?;
                node.content = content;

                Ok(node)
            }
            _ => {
                return Err(new_rhustapp_error(
                    "failed to read node",
                    Some(DecoderError::ErrInvalidNode.to_string()),
                ));
            }
        }
    }

    pub fn read_string(&mut self, length: usize) -> Result<String, RhustAppError> {
        let bytes = self
            .read_bytes(length)
            .map_err(|err| new_rhustapp_error("failed to read string", Some(err.to_string())))?;

        String::from_utf8(bytes)
            .map_err(|err| new_rhustapp_error("failed to read string", Some(err.to_string())))
    }

    pub fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>, RhustAppError> {
        self.check_eos(length)
            .map_err(|err| new_rhustapp_error("failed to read bytes", Some(err.to_string())))?;

        let return_value = Vec::from(&self.data[self.index..self.index + length]);
        self.index += length;

        Ok(return_value)
    }
}

/// Unpacks the given decrypted data from the WhatsApp web API.
///
/// It checks the first byte to decide whether to uncompress the data with zlib or just return
/// as-is (without the first byte). There's currently no corresponding pack function because
/// marshal returns the data with a leading zero (i.e. not compressed).
pub fn unpack_data(data: &Vec<u8>) -> Result<Vec<u8>, RhustAppError> {
    if data.len() == 0 {
        return Err(new_rhustapp_error(
            "failed to unpack data of length 0",
            None,
        ));
    };

    let data_type = data[0];

    if 2 & data_type > 0 {
        let mut decoder = flate2::read::ZlibDecoder::new(&data.as_slice()[1..]);
        let mut decoded_string = String::new();
        decoder.read_to_string(&mut decoded_string).map_err(|err| {
            new_rhustapp_error("failed to decompress data", Some(err.to_string()))
        })?;
        Ok(decoded_string.as_bytes().to_vec())
    } else {
        Ok(data.as_slice()[1..].to_vec())
    }
}

pub fn printable(data: &Vec<u8>) -> String {
    match String::from_utf8(data.to_vec()) {
        Ok(s) => {
            for c in s.chars() {
                if !c.is_alphanumeric() {
                    return String::new();
                }
            }
            s
        }
        Err(_) => String::new(),
    }
}
