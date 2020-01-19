// Message definitions generated with protogen-rust.py

use super::serialization::{DeserializationError, MessageReader, MessageWriter};
use super::textmessage::TextMessage;
use std::convert::TryInto;
use std::fmt;

pub static VERSION: &str = "dp:4.21.2";
pub const UNDO_DEPTH: u32 = 30;

#[derive(Clone, Debug, PartialEq)]
pub struct DisconnectMessage {
    pub reason: u8,
    pub message: String,
}

impl DisconnectMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(1, 65535, 1, 0)?;

        let reason = reader.read::<u8>();
        let message = reader.read_remaining_str();

        Ok(Self { reason, message })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(1, user_id, 1 + self.message.len());
        w.write(self.reason);
        w.write(&self.message);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("reason", self.reason.to_string())
            .set("message", self.message.clone())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            reason: tm.get_u8("reason"),
            message: tm.get_str("message").to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct JoinMessage {
    pub flags: u8,
    pub name: String,
    pub avatar: Vec<u8>,
}

impl JoinMessage {
    pub const FLAGS_AUTH: u8 = 0x1;
    pub const FLAGS_MOD: u8 = 0x2;
    pub const FLAGS_BOT: u8 = 0x4;
    pub const FLAGS: &'static [&'static str] = &["auth", "mod", "bot"];

    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(2, 65535, 32, 0)?;

        let flags = reader.read::<u8>();
        let name_len = reader.read::<u8>() as usize;
        if reader.remaining() < name_len {
            return Err(DeserializationError {
                user_id: 0,
                message_type: 32,
                payload_len: buf.len(),
                error: "Join::name field is too long",
            });
        }
        let name = reader.read_str(name_len);
        let avatar = reader.read_remaining_vec::<u8>();

        Ok(Self {
            flags,
            name,
            avatar,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(
            32,
            user_id,
            1 + self.name.len() + self.avatar.len(),
        );
        w.write(self.flags);
        w.write(self.name.len() as u8);
        w.write(&self.name);
        w.write(&self.avatar);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set_flags("flags", &Self::FLAGS, self.flags)
            .set("name", self.name.clone())
            .set_bytes("avatar", &self.avatar)
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            flags: tm.get_flags(&Self::FLAGS, "flags"),
            name: tm.get_str("name").to_string(),
            avatar: tm.get_bytes("avatar"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub flags: u8,
    pub message: String,
}

impl ChatMessage {
    pub const FLAGS_BYPASS: u8 = 0x1;
    pub const FLAGS_SHOUT: u8 = 0x2;
    pub const FLAGS_ACTION: u8 = 0x4;
    pub const FLAGS_PIN: u8 = 0x8;
    pub const FLAGS: &'static [&'static str] = &["bypass", "shout", "action", "pin"];

    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(1, 65535, 35, 0)?;

        let flags = reader.read::<u8>();
        let message = reader.read_remaining_str();

        Ok(Self { flags, message })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(35, user_id, 1 + self.message.len());
        w.write(self.flags);
        w.write(&self.message);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set_flags("flags", &Self::FLAGS, self.flags)
            .set("message", self.message.clone())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            flags: tm.get_flags(&Self::FLAGS, "flags"),
            message: tm.get_str("message").to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrivateChatMessage {
    pub target: u8,
    pub flags: u8,
    pub message: String,
}

impl PrivateChatMessage {
    pub const FLAGS_ACTION: u8 = 0x1;
    pub const FLAGS: &'static [&'static str] = &["action"];

    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(2, 65535, 38, 0)?;

        let target = reader.read::<u8>();
        let flags = reader.read::<u8>();
        let message = reader.read_remaining_str();

        Ok(Self {
            target,
            flags,
            message,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(38, user_id, 2 + self.message.len());
        w.write(self.target);
        w.write(self.flags);
        w.write(&self.message);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("target", self.target.to_string())
            .set_flags("flags", &Self::FLAGS, self.flags)
            .set("message", self.message.clone())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            target: tm.get_u8("target"),
            flags: tm.get_flags(&Self::FLAGS, "flags"),
            message: tm.get_str("message").to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaserTrailMessage {
    pub color: u32,
    pub persistence: u8,
}

impl LaserTrailMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(5, 5, 65, 0)?;

        let color = reader.read::<u32>();
        let persistence = reader.read::<u8>();

        Ok(Self { color, persistence })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(65, user_id, 5);
        w.write(self.color);
        w.write(self.persistence);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set_argb32("color", self.color)
            .set("persistence", self.persistence.to_string())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            color: tm.get_argb32("color"),
            persistence: tm.get_u8("persistence"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MovePointerMessage {
    pub x: i32,
    pub y: i32,
}

impl MovePointerMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(8, 8, 66, 0)?;

        let x = reader.read::<i32>();
        let y = reader.read::<i32>();

        Ok(Self { x, y })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(66, user_id, 8);
        w.write(self.x);
        w.write(self.y);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("x", self.x.to_string())
            .set("y", self.y.to_string())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            x: tm.get_u32("x") as i32,
            y: tm.get_u32("y") as i32,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerACLMessage {
    pub id: u16,
    pub flags: u8,
    pub exclusive: Vec<u8>,
}

impl LayerACLMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(3, 258, 69, 0)?;

        let id = reader.read::<u16>();
        let flags = reader.read::<u8>();
        let exclusive = reader.read_remaining_vec();

        Ok(Self {
            id,
            flags,
            exclusive,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(69, user_id, 3 + self.exclusive.len());
        w.write(self.id);
        w.write(self.flags);
        w.write(&self.exclusive);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("id", self.id.to_string())
            .set("flags", self.flags.to_string())
            .set_vec_u8("exclusive", &self.exclusive)
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            id: tm.get_u16("id"),
            flags: tm.get_u8("flags"),
            exclusive: tm.get_vec_u8("exclusive"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasResizeMessage {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl CanvasResizeMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(16, 16, 129, 0)?;

        let top = reader.read::<i32>();
        let right = reader.read::<i32>();
        let bottom = reader.read::<i32>();
        let left = reader.read::<i32>();

        Ok(Self {
            top,
            right,
            bottom,
            left,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(129, user_id, 16);
        w.write(self.top);
        w.write(self.right);
        w.write(self.bottom);
        w.write(self.left);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("top", self.top.to_string())
            .set("right", self.right.to_string())
            .set("bottom", self.bottom.to_string())
            .set("left", self.left.to_string())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            top: tm.get_u32("top") as i32,
            right: tm.get_u32("right") as i32,
            bottom: tm.get_u32("bottom") as i32,
            left: tm.get_u32("left") as i32,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerCreateMessage {
    pub id: u16,
    pub source: u16,
    pub fill: u32,
    pub flags: u8,
    pub name: String,
}

impl LayerCreateMessage {
    pub const FLAGS_COPY: u8 = 0x1;
    pub const FLAGS_INSERT: u8 = 0x2;
    pub const FLAGS: &'static [&'static str] = &["copy", "insert"];

    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(9, 65535, 130, 0)?;

        let id = reader.read::<u16>();
        let source = reader.read::<u16>();
        let fill = reader.read::<u32>();
        let flags = reader.read::<u8>();
        let name = reader.read_remaining_str();

        Ok(Self {
            id,
            source,
            fill,
            flags,
            name,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(130, user_id, 9 + self.name.len());
        w.write(self.id);
        w.write(self.source);
        w.write(self.fill);
        w.write(self.flags);
        w.write(&self.name);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("id", format!("0x{:04x}", self.id))
            .set("source", format!("0x{:04x}", self.source))
            .set_argb32("fill", self.fill)
            .set_flags("flags", &Self::FLAGS, self.flags)
            .set("name", self.name.clone())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            id: tm.get_u16("id"),
            source: tm.get_u16("source"),
            fill: tm.get_argb32("fill"),
            flags: tm.get_flags(&Self::FLAGS, "flags"),
            name: tm.get_str("name").to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerAttributesMessage {
    pub id: u16,
    pub sublayer: u8,
    pub flags: u8,
    pub opacity: u8,
    pub blend: u8,
}

impl LayerAttributesMessage {
    pub const FLAGS_CENSOR: u8 = 0x1;
    pub const FLAGS_FIXED: u8 = 0x2;
    pub const FLAGS: &'static [&'static str] = &["censor", "fixed"];

    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(6, 6, 131, 0)?;

        let id = reader.read::<u16>();
        let sublayer = reader.read::<u8>();
        let flags = reader.read::<u8>();
        let opacity = reader.read::<u8>();
        let blend = reader.read::<u8>();

        Ok(Self {
            id,
            sublayer,
            flags,
            opacity,
            blend,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(131, user_id, 6);
        w.write(self.id);
        w.write(self.sublayer);
        w.write(self.flags);
        w.write(self.opacity);
        w.write(self.blend);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("id", format!("0x{:04x}", self.id))
            .set("sublayer", self.sublayer.to_string())
            .set_flags("flags", &Self::FLAGS, self.flags)
            .set("opacity", self.opacity.to_string())
            .set("blend", self.blend.to_string())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            id: tm.get_u16("id"),
            sublayer: tm.get_u8("sublayer"),
            flags: tm.get_flags(&Self::FLAGS, "flags"),
            opacity: tm.get_u8("opacity"),
            blend: tm.get_u8("blend"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerRetitleMessage {
    pub id: u16,
    pub title: String,
}

impl LayerRetitleMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(2, 65535, 132, 0)?;

        let id = reader.read::<u16>();
        let title = reader.read_remaining_str();

        Ok(Self { id, title })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(132, user_id, 2 + self.title.len());
        w.write(self.id);
        w.write(&self.title);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("id", format!("0x{:04x}", self.id))
            .set("title", self.title.clone())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            id: tm.get_u16("id"),
            title: tm.get_str("title").to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerDeleteMessage {
    pub id: u16,
    pub merge: bool,
}

impl LayerDeleteMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(3, 3, 134, 0)?;

        let id = reader.read::<u16>();
        let merge = reader.read::<bool>();

        Ok(Self { id, merge })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(134, user_id, 3);
        w.write(self.id);
        w.write(self.merge);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("id", format!("0x{:04x}", self.id))
            .set("merge", self.merge.to_string())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            id: tm.get_u16("id"),
            merge: tm.get_str("merge") == "true",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerVisibilityMessage {
    pub id: u16,
    pub visible: bool,
}

impl LayerVisibilityMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(3, 3, 135, 0)?;

        let id = reader.read::<u16>();
        let visible = reader.read::<bool>();

        Ok(Self { id, visible })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(135, user_id, 3);
        w.write(self.id);
        w.write(self.visible);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("id", format!("0x{:04x}", self.id))
            .set("visible", self.visible.to_string())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            id: tm.get_u16("id"),
            visible: tm.get_str("visible") == "true",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PutImageMessage {
    pub layer: u16,
    pub mode: u8,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub image: Vec<u8>,
}

impl PutImageMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(19, 65535, 136, 0)?;

        let layer = reader.read::<u16>();
        let mode = reader.read::<u8>();
        let x = reader.read::<u32>();
        let y = reader.read::<u32>();
        let w = reader.read::<u32>();
        let h = reader.read::<u32>();
        let image = reader.read_remaining_vec::<u8>();

        Ok(Self {
            layer,
            mode,
            x,
            y,
            w,
            h,
            image,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(136, user_id, 19 + self.image.len());
        w.write(self.layer);
        w.write(self.mode);
        w.write(self.x);
        w.write(self.y);
        w.write(self.w);
        w.write(self.h);
        w.write(&self.image);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("layer", format!("0x{:04x}", self.layer))
            .set("mode", self.mode.to_string())
            .set("x", self.x.to_string())
            .set("y", self.y.to_string())
            .set("w", self.w.to_string())
            .set("h", self.h.to_string())
            .set_bytes("image", &self.image)
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            layer: tm.get_u16("layer"),
            mode: tm.get_u8("mode"),
            x: tm.get_u32("x"),
            y: tm.get_u32("y"),
            w: tm.get_u32("w"),
            h: tm.get_u32("h"),
            image: tm.get_bytes("image"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FillRectMessage {
    pub layer: u16,
    pub mode: u8,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub color: u32,
}

impl FillRectMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(23, 23, 137, 0)?;

        let layer = reader.read::<u16>();
        let mode = reader.read::<u8>();
        let x = reader.read::<u32>();
        let y = reader.read::<u32>();
        let w = reader.read::<u32>();
        let h = reader.read::<u32>();
        let color = reader.read::<u32>();

        Ok(Self {
            layer,
            mode,
            x,
            y,
            w,
            h,
            color,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(137, user_id, 23);
        w.write(self.layer);
        w.write(self.mode);
        w.write(self.x);
        w.write(self.y);
        w.write(self.w);
        w.write(self.h);
        w.write(self.color);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("layer", format!("0x{:04x}", self.layer))
            .set("mode", self.mode.to_string())
            .set("x", self.x.to_string())
            .set("y", self.y.to_string())
            .set("w", self.w.to_string())
            .set("h", self.h.to_string())
            .set_argb32("color", self.color)
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            layer: tm.get_u16("layer"),
            mode: tm.get_u8("mode"),
            x: tm.get_u32("x"),
            y: tm.get_u32("y"),
            w: tm.get_u32("w"),
            h: tm.get_u32("h"),
            color: tm.get_argb32("color"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnnotationCreateMessage {
    pub id: u16,
    pub x: i32,
    pub y: i32,
    pub w: u16,
    pub h: u16,
}

impl AnnotationCreateMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(14, 14, 141, 0)?;

        let id = reader.read::<u16>();
        let x = reader.read::<i32>();
        let y = reader.read::<i32>();
        let w = reader.read::<u16>();
        let h = reader.read::<u16>();

        Ok(Self { id, x, y, w, h })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(141, user_id, 14);
        w.write(self.id);
        w.write(self.x);
        w.write(self.y);
        w.write(self.w);
        w.write(self.h);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("id", format!("0x{:04x}", self.id))
            .set("x", self.x.to_string())
            .set("y", self.y.to_string())
            .set("w", self.w.to_string())
            .set("h", self.h.to_string())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            id: tm.get_u16("id"),
            x: tm.get_u32("x") as i32,
            y: tm.get_u32("y") as i32,
            w: tm.get_u16("w"),
            h: tm.get_u16("h"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnnotationReshapeMessage {
    pub id: u16,
    pub x: i32,
    pub y: i32,
    pub w: u16,
    pub h: u16,
}

impl AnnotationReshapeMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(14, 14, 142, 0)?;

        let id = reader.read::<u16>();
        let x = reader.read::<i32>();
        let y = reader.read::<i32>();
        let w = reader.read::<u16>();
        let h = reader.read::<u16>();

        Ok(Self { id, x, y, w, h })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(142, user_id, 14);
        w.write(self.id);
        w.write(self.x);
        w.write(self.y);
        w.write(self.w);
        w.write(self.h);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("id", format!("0x{:04x}", self.id))
            .set("x", self.x.to_string())
            .set("y", self.y.to_string())
            .set("w", self.w.to_string())
            .set("h", self.h.to_string())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            id: tm.get_u16("id"),
            x: tm.get_u32("x") as i32,
            y: tm.get_u32("y") as i32,
            w: tm.get_u16("w"),
            h: tm.get_u16("h"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnnotationEditMessage {
    pub id: u16,
    pub bg: u32,
    pub flags: u8,
    pub border: u8,
    pub text: String,
}

impl AnnotationEditMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(8, 65535, 143, 0)?;

        let id = reader.read::<u16>();
        let bg = reader.read::<u32>();
        let flags = reader.read::<u8>();
        let border = reader.read::<u8>();
        let text = reader.read_remaining_str();

        Ok(Self {
            id,
            bg,
            flags,
            border,
            text,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(143, user_id, 8 + self.text.len());
        w.write(self.id);
        w.write(self.bg);
        w.write(self.flags);
        w.write(self.border);
        w.write(&self.text);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("id", format!("0x{:04x}", self.id))
            .set_argb32("bg", self.bg)
            .set("flags", self.flags.to_string())
            .set("border", self.border.to_string())
            .set("text", self.text.clone())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            id: tm.get_u16("id"),
            bg: tm.get_argb32("bg"),
            flags: tm.get_u8("flags"),
            border: tm.get_u8("border"),
            text: tm.get_str("text").to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MoveRegionMessage {
    pub layer: u16,
    pub bx: i32,
    pub by: i32,
    pub bw: i32,
    pub bh: i32,
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub x3: i32,
    pub y3: i32,
    pub x4: i32,
    pub y4: i32,
    pub mask: Vec<u8>,
}

impl MoveRegionMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(50, 65535, 145, 0)?;

        let layer = reader.read::<u16>();
        let bx = reader.read::<i32>();
        let by = reader.read::<i32>();
        let bw = reader.read::<i32>();
        let bh = reader.read::<i32>();
        let x1 = reader.read::<i32>();
        let y1 = reader.read::<i32>();
        let x2 = reader.read::<i32>();
        let y2 = reader.read::<i32>();
        let x3 = reader.read::<i32>();
        let y3 = reader.read::<i32>();
        let x4 = reader.read::<i32>();
        let y4 = reader.read::<i32>();
        let mask = reader.read_remaining_vec::<u8>();

        Ok(Self {
            layer,
            bx,
            by,
            bw,
            bh,
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
            x4,
            y4,
            mask,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(145, user_id, 50 + self.mask.len());
        w.write(self.layer);
        w.write(self.bx);
        w.write(self.by);
        w.write(self.bw);
        w.write(self.bh);
        w.write(self.x1);
        w.write(self.y1);
        w.write(self.x2);
        w.write(self.y2);
        w.write(self.x3);
        w.write(self.y3);
        w.write(self.x4);
        w.write(self.y4);
        w.write(&self.mask);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("layer", format!("0x{:04x}", self.layer))
            .set("bx", self.bx.to_string())
            .set("by", self.by.to_string())
            .set("bw", self.bw.to_string())
            .set("bh", self.bh.to_string())
            .set("x1", self.x1.to_string())
            .set("y1", self.y1.to_string())
            .set("x2", self.x2.to_string())
            .set("y2", self.y2.to_string())
            .set("x3", self.x3.to_string())
            .set("y3", self.y3.to_string())
            .set("x4", self.x4.to_string())
            .set("y4", self.y4.to_string())
            .set_bytes("mask", &self.mask)
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            layer: tm.get_u16("layer"),
            bx: tm.get_u32("bx") as i32,
            by: tm.get_u32("by") as i32,
            bw: tm.get_u32("bw") as i32,
            bh: tm.get_u32("bh") as i32,
            x1: tm.get_u32("x1") as i32,
            y1: tm.get_u32("y1") as i32,
            x2: tm.get_u32("x2") as i32,
            y2: tm.get_u32("y2") as i32,
            x3: tm.get_u32("x3") as i32,
            y3: tm.get_u32("y3") as i32,
            x4: tm.get_u32("x4") as i32,
            y4: tm.get_u32("y4") as i32,
            mask: tm.get_bytes("mask"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PutTileMessage {
    pub layer: u16,
    pub col: u16,
    pub row: u16,
    pub repeat: u16,
    pub sublayer: u8,
    pub image: Vec<u8>,
}

impl PutTileMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(9, 65535, 146, 0)?;

        let layer = reader.read::<u16>();
        let col = reader.read::<u16>();
        let row = reader.read::<u16>();
        let repeat = reader.read::<u16>();
        let sublayer = reader.read::<u8>();
        let image = reader.read_remaining_vec::<u8>();

        Ok(Self {
            layer,
            col,
            row,
            repeat,
            sublayer,
            image,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(146, user_id, 9 + self.image.len());
        w.write(self.layer);
        w.write(self.col);
        w.write(self.row);
        w.write(self.repeat);
        w.write(self.sublayer);
        w.write(&self.image);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("layer", format!("0x{:04x}", self.layer))
            .set("col", self.col.to_string())
            .set("row", self.row.to_string())
            .set("repeat", self.repeat.to_string())
            .set("sublayer", self.sublayer.to_string())
            .set_bytes("image", &self.image)
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            layer: tm.get_u16("layer"),
            col: tm.get_u16("col"),
            row: tm.get_u16("row"),
            repeat: tm.get_u16("repeat"),
            sublayer: tm.get_u8("sublayer"),
            image: tm.get_bytes("image"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClassicDab {
    pub x: i8,
    pub y: i8,
    pub size: u16,
    pub opacity: u8,
    pub hardness: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawDabsClassicMessage {
    pub layer: u16,
    pub x: i32,
    pub y: i32,
    pub color: u32,
    pub mode: u8,
    pub dabs: Vec<ClassicDab>,
}

impl DrawDabsClassicMessage {
    pub const MAX_ITEMS: usize = 10920;

    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(21, 65535, 148, 0)?;

        let layer = reader.read::<u16>();
        let x = reader.read::<i32>();
        let y = reader.read::<i32>();
        let color = reader.read::<u32>();
        let mode = reader.read::<u8>();
        let mut dabs = Vec::<ClassicDab>::with_capacity(reader.remaining() / 6);
        while reader.remaining() > 0 {
            let x = reader.read::<i8>();
            let y = reader.read::<i8>();
            let size = reader.read::<u16>();
            let opacity = reader.read::<u8>();
            let hardness = reader.read::<u8>();
            dabs.push(ClassicDab {
                x,
                y,
                size,
                opacity,
                hardness,
            });
        }
        Ok(Self {
            layer,
            x,
            y,
            color,
            mode,
            dabs,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(148, user_id, 15 + (self.dabs.len() * 6));
        w.write(self.layer);
        w.write(self.x);
        w.write(self.y);
        w.write(self.color);
        w.write(self.mode);
        for item in self.dabs.iter() {
            w.write(item.x);
            w.write(item.y);
            w.write(item.size);
            w.write(item.opacity);
            w.write(item.hardness);
        }

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        let mut dabs: Vec<Vec<f64>> = Vec::with_capacity(self.dabs.len());
        for dab in self.dabs.iter() {
            dabs.push(vec![
                dab.x as f64 / 4.0,
                dab.y as f64 / 4.0,
                dab.size as f64 / 256.0,
                dab.opacity as f64,
                dab.hardness as f64,
            ]);
        }
        txt.set("layer", format!("0x{:04x}", self.layer))
            .set("x", (self.x as f64 / 4.0).to_string())
            .set("y", (self.y as f64 / 4.0).to_string())
            .set_argb32("color", self.color)
            .set("mode", self.mode.to_string())
            .set_dabs(dabs)
    }

    fn from_text(tm: &TextMessage) -> Self {
        let mut dab_structs: Vec<ClassicDab> = Vec::with_capacity(tm.dabs.len());
        for dab in tm.dabs.iter() {
            if dab.len() != 5 {
                continue;
            }
            dab_structs.push(ClassicDab {
                x: (dab[0] * 4.0) as i8,
                y: (dab[1] * 4.0) as i8,
                size: (dab[2] * 256.0) as u16,
                opacity: (dab[3]) as u8,
                hardness: (dab[4]) as u8,
            });
        }

        Self {
            layer: tm.get_u16("layer"),
            x: (tm.get_f64("x") * 4.0) as i32,
            y: (tm.get_f64("y") * 4.0) as i32,
            color: tm.get_argb32("color"),
            mode: tm.get_u8("mode"),
            dabs: dab_structs,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PixelDab {
    pub x: i8,
    pub y: i8,
    pub size: u8,
    pub opacity: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawDabsPixelMessage {
    pub layer: u16,
    pub x: i32,
    pub y: i32,
    pub color: u32,
    pub mode: u8,
    pub dabs: Vec<PixelDab>,
}

impl DrawDabsPixelMessage {
    pub const MAX_ITEMS: usize = 16380;

    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(19, 65535, 149, 0)?;

        let layer = reader.read::<u16>();
        let x = reader.read::<i32>();
        let y = reader.read::<i32>();
        let color = reader.read::<u32>();
        let mode = reader.read::<u8>();
        let mut dabs = Vec::<PixelDab>::with_capacity(reader.remaining() / 4);
        while reader.remaining() > 0 {
            let x = reader.read::<i8>();
            let y = reader.read::<i8>();
            let size = reader.read::<u8>();
            let opacity = reader.read::<u8>();
            dabs.push(PixelDab {
                x,
                y,
                size,
                opacity,
            });
        }
        Ok(Self {
            layer,
            x,
            y,
            color,
            mode,
            dabs,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(149, user_id, 15 + (self.dabs.len() * 4));
        w.write(self.layer);
        w.write(self.x);
        w.write(self.y);
        w.write(self.color);
        w.write(self.mode);
        for item in self.dabs.iter() {
            w.write(item.x);
            w.write(item.y);
            w.write(item.size);
            w.write(item.opacity);
        }

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        let mut dabs: Vec<Vec<f64>> = Vec::with_capacity(self.dabs.len());
        for dab in self.dabs.iter() {
            dabs.push(vec![
                dab.x as f64,
                dab.y as f64,
                dab.size as f64,
                dab.opacity as f64,
            ]);
        }
        txt.set("layer", format!("0x{:04x}", self.layer))
            .set("x", self.x.to_string())
            .set("y", self.y.to_string())
            .set_argb32("color", self.color)
            .set("mode", self.mode.to_string())
            .set_dabs(dabs)
    }

    fn from_text(tm: &TextMessage) -> Self {
        let mut dab_structs: Vec<PixelDab> = Vec::with_capacity(tm.dabs.len());
        for dab in tm.dabs.iter() {
            if dab.len() != 4 {
                continue;
            }
            dab_structs.push(PixelDab {
                x: (dab[0]) as i8,
                y: (dab[1]) as i8,
                size: (dab[2]) as u8,
                opacity: (dab[3]) as u8,
            });
        }

        Self {
            layer: tm.get_u16("layer"),
            x: tm.get_u32("x") as i32,
            y: tm.get_u32("y") as i32,
            color: tm.get_argb32("color"),
            mode: tm.get_u8("mode"),
            dabs: dab_structs,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UndoMessage {
    pub override_user: u8,
    pub redo: bool,
}

impl UndoMessage {
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf).check_len(2, 2, 255, 0)?;

        let override_user = reader.read::<u8>();
        let redo = reader.read::<bool>();

        Ok(Self {
            override_user,
            redo,
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload(255, user_id, 2);
        w.write(self.override_user);
        w.write(self.redo);

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        txt.set("override_user", self.override_user.to_string())
            .set("redo", self.redo.to_string())
    }

    fn from_text(tm: &TextMessage) -> Self {
        Self {
            override_user: tm.get_u8("override_user"),
            redo: tm.get_str("redo") == "true",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControlMessage {
    /// Server command message
    ///
    /// This is a general purpose message for sending commands to the server
    /// and receiving replies. This is used for (among other things):
    ///
    /// - the login handshake
    /// - setting session parameters (e.g. max user count and password)
    /// - sending administration commands (e.g. kick user)
    ///
    ServerCommand(u8, String),

    /// Disconnect notification
    ///
    /// This message is used when closing the connection gracefully. The message queue
    /// will automatically close the socket after sending this message.
    ///
    Disconnect(u8, DisconnectMessage),

    /// Ping message
    ///
    /// This is used for latency measurement as well as a keepalive. Normally, the client
    /// should be the one to send the ping messages.
    ///
    /// The server should return a Ping with the is_pong flag set
    ///
    Ping(u8, bool),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ServerMetaMessage {
    /// Inform the client of a new user
    ///
    /// This message is sent only be the server. It associates a username
    /// with a context ID.
    ///
    Join(u8, JoinMessage),

    /// Inform the client of a user leaving
    ///
    /// This message is sent only by the server. Upon receiving this message,
    /// clients will typically remove the user from the user listing. The client
    /// is also allowed to release resources associated with this context ID.
    ///
    Leave(u8),

    /// Session ownership change
    ///
    /// This message sets the users who have operator status. It can be
    /// sent by users who are already operators or by the server (user id=0).
    ///
    /// The list of operators implicitly contains the user who sends the
    /// message, thus users cannot deop themselves.
    ///
    /// The server sanitizes the ID list so, when distributed to other users,
    /// it does not contain any duplicates or non-existing users and can be trusted
    /// without checking the access control list.
    ///
    SessionOwner(u8, Vec<u8>),

    /// A chat message
    ///
    /// Chat message sent by the server with the user ID 0 are server messages.
    /// (Typically a Command message is used for server announcements, but the Chat message
    /// is used for those messages that must be stored in the session history.)
    ///
    Chat(u8, ChatMessage),

    /// List of trusted users
    ///
    /// This message sets the list of user who have been tagged as trusted,
    /// but who are not operators. The meaning of "trusted" is a mostly
    /// clientside concept, but the session can be configured to allow trusted
    /// users access to some operator commands. (Deputies)
    ///
    /// This command can be sent by operators or by the server (ctx=0).
    ///
    /// The server sanitizes the ID list so, when distributed to other users,
    /// it does not contain any duplicates or non-existing users and can be trusted
    /// without checking the access control list.
    ///
    TrustedUsers(u8, Vec<u8>),

    /// Soft reset point marker
    ///
    /// This message marks the point in the session history where a soft reset occurs.
    /// A thick-server performs an internal soft-reset when a user joins.
    ///
    /// All users should truncate their own session history when receiving this message,
    /// since undos cannot cross the reset boundary.
    ///
    SoftReset(u8),

    /// A private chat message
    ///
    /// Note. This message type was added in protocol 4.21.2 (v. 2.1.0). For backward compatiblity,
    /// the server will not send any private messages from itself; it will only relay them from
    /// other users. In version 3.0, this should be merged with the normal Chat message.
    ///
    /// Private messages always bypass the session history.
    ///
    PrivateChat(u8, PrivateChatMessage),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClientMetaMessage {
    /// Event interval record
    ///
    /// This is used to preserve timing information in session recordings.
    ///
    /// Note. The maximum interval (using a single message) is about 65 seconds.
    /// Typically the intervals we want to store are a few seconds at most, so this should be enough.
    ///
    Interval(u8, u16),

    /// Start/end drawing pointer laser trail
    ///
    /// This signals the beginning or the end of a laser pointer trail. The trail coordinates
    /// are sent with MovePointer messages.
    ///
    /// A nonzero persistence indicates the start of the trail and zero the end.
    ///
    LaserTrail(u8, LaserTrailMessage),

    /// Move user pointer
    ///
    /// This is message is used to update the position of the user pointer when no
    /// actual drawing is taking place. It is also used to draw the "laser pointer" trail.
    /// Note. This is a META message, since this is used for a temporary visual effect only,
    /// and thus doesn't affect the actual canvas content.
    ///
    /// The pointer position is given in integer coordinates.
    ///
    MovePointer(u8, MovePointerMessage),

    /// A bookmark
    ///
    /// This is used to bookmark points in the session for quick access when playing back a recording
    ///
    Marker(u8, String),

    /// Set user specific locks
    ///
    /// This is an opaque meta command that contains a list of users to be locked.
    /// It can only be sent by session operators.
    ///
    UserACL(u8, Vec<u8>),

    /// Change layer access control list
    ///
    /// This is an opaque meta command. It is used to set the general layer lock
    /// as well as give exclusive access to selected users.
    ///
    /// When the OWNLAYERS mode is set, any user can use this to change the ACLs on layers they themselves
    /// have created (identified by the ID prefix.)
    ///
    /// Using layer ID 0 sets or clears a general canvaswide lock. The tier and exclusive user list is not
    /// used in this case.
    ///
    /// The eighth bit of the flags field (0x80) indicates whether the layer is locked in general.
    /// The first three bits (0x07) indicate the access tier level.
    ///
    LayerACL(u8, LayerACLMessage),

    /// Change feature access tiers
    FeatureAccessLevels(u8, Vec<u8>),

    /// Set the default layer
    ///
    /// The default layer is the one new users default to when logging in.
    /// If no default layer is set, the newest layer will be selected by default.
    ///
    DefaultLayer(u8, u16),

    /// A message that has been filtered away by the ACL filter
    ///
    /// This is only used in recordings for mainly debugging purposes.
    /// This message should never be sent over the network.
    ///
    Filtered(u8, Vec<u8>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandMessage {
    /// Undo demarcation point
    ///
    /// The client sends an UndoPoint message to signal the start of an undoable sequence.
    ///
    UndoPoint(u8),

    /// Adjust canvas size
    ///
    /// This is the first command that must be sent to initialize the session.
    ///
    /// This affects the size of all existing and future layers.
    ///
    /// The new canvas size is relative to the old one. The four adjustement
    /// parameters extend or retract their respective borders.
    /// Initial canvas resize should be (0, w, h, 0).
    ///
    CanvasResize(u8, CanvasResizeMessage),

    /// Create a new layer
    ///
    /// A session starts with zero layers, so a layer creation command is typically
    /// the second command to be sent, right after setting the canvas size.
    ///
    /// The layer ID must be prefixed with the context ID of the user creating it.
    /// This allows the client to choose the layer ID without worrying about
    /// clashes. In multiuser mode the ACL filter validates the prefix for all new layers.
    ///
    /// The following flags can be used with layer creation:
    /// - COPY: a copy of the Source layer is made, rather than a blank layer
    /// - INSERT: the new layer is inserted above the Source layer. Source 0 means
    ///           the layer will be placed bottom-most on the stack
    ///
    /// The Source layer ID should be zero when COPY or INSERT flags are not used.
    /// When COPY is used, it should refer to an existing layer. Copy commands
    /// referring to missing layers are dropped.
    /// When INSERT is used, referring to 0 or a nonexistent layer places
    /// the new layer at the bottom of the stack.
    ///
    /// If layer controls are locked, this command requires session operator privileges.
    ///
    LayerCreate(u8, LayerCreateMessage),

    /// Change layer attributes
    ///
    /// If the target layer is locked, this command requires session operator privileges.
    ///
    /// Specifying a sublayer requires session operator privileges. Currently, it is used
    /// only when sublayers are needed at canvas initialization.
    ///
    LayerAttributes(u8, LayerAttributesMessage),

    /// Change a layer's title
    LayerRetitle(u8, LayerRetitleMessage),

    /// Reorder layers
    ///
    /// New layers are always added to the top of the stack.
    /// This command includes a list of layer IDs that define the new stacking order.
    ///
    /// An order change should list all layers in the stack, but due to synchronization issues, that
    /// is not always possible.
    /// The layer order should therefore be sanitized by removing all layers not in the current layer stack
    /// and adding all missing layers to the end in their current relative order.
    ///
    /// For example: if the current stack is [1,2,3,4,5] and the client receives
    /// a reordering command [3,4,1], the missing layers are appended: [3,4,1,2,5].
    ///
    /// If layer controls are locked, this command requires session operator privileges.
    ///
    LayerOrder(u8, Vec<u16>),

    /// Delete a layer
    ///
    /// If the merge attribute is set, the contents of the layer is merged
    /// to the layer below it. Merging the bottom-most layer does nothing.
    ///
    /// If the current layer or layer controls in general are locked, this command
    /// requires session operator privileges.
    ///
    LayerDelete(u8, LayerDeleteMessage),

    /// Change layer visibility
    ///
    /// This command is used to toggle the layer visibility for the local user.
    /// (I.e. any user is allowed to send this command and it has no effect on
    /// other users.)
    /// Even though this only affects the sending user, this message can be
    /// sent through the official session history to keep the architecture simple.
    ///
    /// Note: to hide the layer for all users, use LayerAttributes to set its opacity
    /// to zero.
    ///
    LayerVisibility(u8, LayerVisibilityMessage),

    /// Draw a bitmap onto a layer
    ///
    /// This is used for pasting images, floodfill, merging annotations and
    /// other tasks where image processing is done clientisde.
    ///
    /// All layer blending modes are supported.
    ///
    /// The image data is DEFLATEd 32bit premultiplied ARGB data. The image
    /// is prefixed with a 32 bit unsigned integer (big endian) which contains
    /// the expected length of the uncompressed data.
    ///
    /// Note that since the message length is fairly limited, a
    /// large image may have to be divided into multiple PutImage
    /// commands.
    ///
    PutImage(u8, PutImageMessage),

    /// Fill a rectangle with solid color
    FillRect(u8, FillRectMessage),

    /// Pen up command
    ///
    /// The pen up command signals the end of a stroke. In indirect drawing mode, it causes
    /// indirect dabs (by this user) to be merged to their parent layers.
    ///
    PenUp(u8),

    /// Create a new annotation
    ///
    /// Annotations are floating text layers. They are drawn over the image layers and
    /// have no defined stacking order.
    ///
    /// The new annotation created with this command is initally empy with a transparent background
    ///
    AnnotationCreate(u8, AnnotationCreateMessage),

    /// Change the position and size of an annotation
    AnnotationReshape(u8, AnnotationReshapeMessage),

    /// Change annotation content
    ///
    /// Accepted contents is the subset of HTML understood by QTextDocument
    ///
    /// If an annotation is flagged as protected, it cannot be modified by users
    /// other than the one who created it, or session operators.
    ///
    AnnotationEdit(u8, AnnotationEditMessage),

    /// Delete an annotation
    ///
    /// Note: Unlike in layer delete command, there is no "merge" option here.
    /// Merging an annotation is done by rendering the annotation item to
    /// an image and drawing the image with the PutImage command. This ensures
    /// identical rendering on all clients.
    ///
    AnnotationDelete(u8, u16),

    /// Move (and transform) a region of a layer.
    ///
    /// This is used to implement selection moving. It is equivalent
    /// to doing two PutImages: the first to mask away the original
    /// selection and the other to paste the selection to a new location.
    /// This command packages that into a single action that is more
    /// bandwidth efficient and can be used even when PutImages in general
    /// are locked, since it's not introducing any new pixels onto the canvas.
    /// Internally, the paint engine performs the following steps:
    ///
    /// 1. Copy selected pixels to a buffer
    /// 2. Erase selected pixels from the layer
    /// 3. Composite transformed buffer onto the layer.
    ///
    /// The pixel selection is determined by the mask bitmap. The mask
    /// is DEFLATEd 1 bit per pixel bitmap data.
    /// For axis aligned rectangle selections, no bitmap is necessary.
    ///
    MoveRegion(u8, MoveRegionMessage),

    /// Set the content of a tile
    ///
    /// Unlike PutImage, this replaces an entire tile directly without any blending.
    /// This command is typically used during canvas initialization to set the initial content.
    ///
    /// PutTile can target sublayers as well. This is used when generating a reset image
    /// with incomplete indirect strokes. Sending a PenUp command will merge the sublayer.
    ///
    PutTile(u8, PutTileMessage),

    /// Set the canvas background tile
    ///
    /// If the payload is exactly 4 bytes long, it should be interpreted as a solid background color.
    /// Otherwise, it is the DEFLATED tile bitmap
    ///
    CanvasBackground(u8, Vec<u8>),

    /// Draw classic brush dabs
    ///
    /// A simple delta compression scheme is used.
    /// The coordinates of each dab are relative to the previous dab.
    /// The coordinate system has 1/4 pixel resolution. Divide by 4.0 before use.
    /// The size field is the brush diameter multiplied by 256.
    ///
    DrawDabsClassic(u8, DrawDabsClassicMessage),

    /// Draw round pixel brush dabs
    ///
    /// The same kind of delta compression is used as in classicdabs,
    /// but the fields all have integer precision.
    ///
    DrawDabsPixel(u8, DrawDabsPixelMessage),

    /// Draw square pixel brush dabs
    DrawDabsPixelSquare(u8, DrawDabsPixelMessage),

    /// Undo or redo actions
    Undo(u8, UndoMessage),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Control(ControlMessage),
    ServerMeta(ServerMetaMessage),
    ClientMeta(ClientMetaMessage),
    Command(CommandMessage),
}

impl ControlMessage {
    pub fn serialize(&self) -> Vec<u8> {
        use ControlMessage::*;
        match &self {
            ServerCommand(user_id, b) => MessageWriter::single(0, *user_id, b),
            Disconnect(user_id, b) => b.serialize(*user_id),
            Ping(user_id, b) => MessageWriter::single(2, *user_id, *b),
        }
    }

    pub fn as_text(&self) -> TextMessage {
        use ControlMessage::*;
        match &self {
            ServerCommand(user_id, b) => {
                TextMessage::new(*user_id, "servercommand").set("msg", b.clone())
            }
            Disconnect(user_id, b) => b.to_text(TextMessage::new(*user_id, "disconnect")),
            Ping(user_id, b) => TextMessage::new(*user_id, "ping").set("is_pong", b.to_string()),
        }
    }

    pub fn user(&self) -> u8 {
        use ControlMessage::*;
        match &self {
            ServerCommand(user_id, _) => *user_id,
            Disconnect(user_id, _) => *user_id,
            Ping(user_id, _) => *user_id,
        }
    }
}

impl fmt::Display for ControlMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_text().fmt(f)
    }
}

impl From<ControlMessage> for Message {
    fn from(item: ControlMessage) -> Message {
        Message::Control(item)
    }
}

impl ServerMetaMessage {
    pub fn serialize(&self) -> Vec<u8> {
        use ServerMetaMessage::*;
        match &self {
            Join(user_id, b) => b.serialize(*user_id),
            Leave(user_id) => MessageWriter::with_expected_payload(33, *user_id, 0).into(),
            SessionOwner(user_id, b) => MessageWriter::single(34, *user_id, b),
            Chat(user_id, b) => b.serialize(*user_id),
            TrustedUsers(user_id, b) => MessageWriter::single(36, *user_id, b),
            SoftReset(user_id) => MessageWriter::with_expected_payload(37, *user_id, 0).into(),
            PrivateChat(user_id, b) => b.serialize(*user_id),
        }
    }

    pub fn as_text(&self) -> TextMessage {
        use ServerMetaMessage::*;
        match &self {
            Join(user_id, b) => b.to_text(TextMessage::new(*user_id, "join")),
            Leave(user_id) => TextMessage::new(*user_id, "leave"),
            SessionOwner(user_id, b) => {
                TextMessage::new(*user_id, "sessionowner").set_vec_u8("users", &b)
            }
            Chat(user_id, b) => b.to_text(TextMessage::new(*user_id, "chat")),
            TrustedUsers(user_id, b) => {
                TextMessage::new(*user_id, "trusted").set_vec_u8("users", &b)
            }
            SoftReset(user_id) => TextMessage::new(*user_id, "softreset"),
            PrivateChat(user_id, b) => b.to_text(TextMessage::new(*user_id, "privatechat")),
        }
    }

    pub fn user(&self) -> u8 {
        use ServerMetaMessage::*;
        match &self {
            Join(user_id, _) => *user_id,
            Leave(user_id) => *user_id,
            SessionOwner(user_id, _) => *user_id,
            Chat(user_id, _) => *user_id,
            TrustedUsers(user_id, _) => *user_id,
            SoftReset(user_id) => *user_id,
            PrivateChat(user_id, _) => *user_id,
        }
    }
}

impl fmt::Display for ServerMetaMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_text().fmt(f)
    }
}

impl From<ServerMetaMessage> for Message {
    fn from(item: ServerMetaMessage) -> Message {
        Message::ServerMeta(item)
    }
}

impl ClientMetaMessage {
    pub fn serialize(&self) -> Vec<u8> {
        use ClientMetaMessage::*;
        match &self {
            Interval(user_id, b) => MessageWriter::single(64, *user_id, *b),
            LaserTrail(user_id, b) => b.serialize(*user_id),
            MovePointer(user_id, b) => b.serialize(*user_id),
            Marker(user_id, b) => MessageWriter::single(67, *user_id, b),
            UserACL(user_id, b) => MessageWriter::single(68, *user_id, b),
            LayerACL(user_id, b) => b.serialize(*user_id),
            FeatureAccessLevels(user_id, b) => MessageWriter::single(70, *user_id, b),
            DefaultLayer(user_id, b) => MessageWriter::single(71, *user_id, *b),
            Filtered(user_id, b) => MessageWriter::single(72, *user_id, b),
        }
    }

    pub fn as_text(&self) -> TextMessage {
        use ClientMetaMessage::*;
        match &self {
            Interval(user_id, b) => {
                TextMessage::new(*user_id, "interval").set("msecs", b.to_string())
            }
            LaserTrail(user_id, b) => b.to_text(TextMessage::new(*user_id, "lasertrail")),
            MovePointer(user_id, b) => b.to_text(TextMessage::new(*user_id, "movepointer")),
            Marker(user_id, b) => TextMessage::new(*user_id, "marker").set("text", b.clone()),
            UserACL(user_id, b) => TextMessage::new(*user_id, "useracl").set_vec_u8("users", &b),
            LayerACL(user_id, b) => b.to_text(TextMessage::new(*user_id, "layeracl")),
            FeatureAccessLevels(user_id, b) => {
                TextMessage::new(*user_id, "featureaccess").set_vec_u8("feature_tiers", &b)
            }
            DefaultLayer(user_id, b) => {
                TextMessage::new(*user_id, "defaultlayer").set("id", format!("0x{:04x}", b))
            }
            Filtered(user_id, b) => TextMessage::new(*user_id, "filtered").set_bytes("message", &b),
        }
    }

    pub fn user(&self) -> u8 {
        use ClientMetaMessage::*;
        match &self {
            Interval(user_id, _) => *user_id,
            LaserTrail(user_id, _) => *user_id,
            MovePointer(user_id, _) => *user_id,
            Marker(user_id, _) => *user_id,
            UserACL(user_id, _) => *user_id,
            LayerACL(user_id, _) => *user_id,
            FeatureAccessLevels(user_id, _) => *user_id,
            DefaultLayer(user_id, _) => *user_id,
            Filtered(user_id, _) => *user_id,
        }
    }
}

impl fmt::Display for ClientMetaMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_text().fmt(f)
    }
}

impl From<ClientMetaMessage> for Message {
    fn from(item: ClientMetaMessage) -> Message {
        Message::ClientMeta(item)
    }
}

impl CommandMessage {
    pub fn serialize(&self) -> Vec<u8> {
        use CommandMessage::*;
        match &self {
            UndoPoint(user_id) => MessageWriter::with_expected_payload(128, *user_id, 0).into(),
            CanvasResize(user_id, b) => b.serialize(*user_id),
            LayerCreate(user_id, b) => b.serialize(*user_id),
            LayerAttributes(user_id, b) => b.serialize(*user_id),
            LayerRetitle(user_id, b) => b.serialize(*user_id),
            LayerOrder(user_id, b) => MessageWriter::single(133, *user_id, b),
            LayerDelete(user_id, b) => b.serialize(*user_id),
            LayerVisibility(user_id, b) => b.serialize(*user_id),
            PutImage(user_id, b) => b.serialize(*user_id),
            FillRect(user_id, b) => b.serialize(*user_id),
            PenUp(user_id) => MessageWriter::with_expected_payload(140, *user_id, 0).into(),
            AnnotationCreate(user_id, b) => b.serialize(*user_id),
            AnnotationReshape(user_id, b) => b.serialize(*user_id),
            AnnotationEdit(user_id, b) => b.serialize(*user_id),
            AnnotationDelete(user_id, b) => MessageWriter::single(144, *user_id, *b),
            MoveRegion(user_id, b) => b.serialize(*user_id),
            PutTile(user_id, b) => b.serialize(*user_id),
            CanvasBackground(user_id, b) => MessageWriter::single(147, *user_id, b),
            DrawDabsClassic(user_id, b) => b.serialize(*user_id),
            DrawDabsPixel(user_id, b) => b.serialize(*user_id),
            DrawDabsPixelSquare(user_id, b) => b.serialize(*user_id),
            Undo(user_id, b) => b.serialize(*user_id),
        }
    }

    pub fn as_text(&self) -> TextMessage {
        use CommandMessage::*;
        match &self {
            UndoPoint(user_id) => TextMessage::new(*user_id, "undopoint"),
            CanvasResize(user_id, b) => b.to_text(TextMessage::new(*user_id, "resize")),
            LayerCreate(user_id, b) => b.to_text(TextMessage::new(*user_id, "newlayer")),
            LayerAttributes(user_id, b) => b.to_text(TextMessage::new(*user_id, "layerattr")),
            LayerRetitle(user_id, b) => b.to_text(TextMessage::new(*user_id, "retitlelayer")),
            LayerOrder(user_id, b) => {
                TextMessage::new(*user_id, "layerorder").set_vec_u16("layers", &b, true)
            }
            LayerDelete(user_id, b) => b.to_text(TextMessage::new(*user_id, "deletelayer")),
            LayerVisibility(user_id, b) => b.to_text(TextMessage::new(*user_id, "layervisibility")),
            PutImage(user_id, b) => b.to_text(TextMessage::new(*user_id, "putimage")),
            FillRect(user_id, b) => b.to_text(TextMessage::new(*user_id, "fillrect")),
            PenUp(user_id) => TextMessage::new(*user_id, "penup"),
            AnnotationCreate(user_id, b) => b.to_text(TextMessage::new(*user_id, "newannotation")),
            AnnotationReshape(user_id, b) => {
                b.to_text(TextMessage::new(*user_id, "reshapeannotation"))
            }
            AnnotationEdit(user_id, b) => b.to_text(TextMessage::new(*user_id, "editannotation")),
            AnnotationDelete(user_id, b) => {
                TextMessage::new(*user_id, "deleteannotation").set("id", format!("0x{:04x}", b))
            }
            MoveRegion(user_id, b) => b.to_text(TextMessage::new(*user_id, "moveregion")),
            PutTile(user_id, b) => b.to_text(TextMessage::new(*user_id, "puttile")),
            CanvasBackground(user_id, b) => {
                TextMessage::new(*user_id, "background").set_bytes("image", &b)
            }
            DrawDabsClassic(user_id, b) => b.to_text(TextMessage::new(*user_id, "classicdabs")),
            DrawDabsPixel(user_id, b) => b.to_text(TextMessage::new(*user_id, "pixeldabs")),
            DrawDabsPixelSquare(user_id, b) => {
                b.to_text(TextMessage::new(*user_id, "squarepixeldabs"))
            }
            Undo(user_id, b) => b.to_text(TextMessage::new(*user_id, "undo")),
        }
    }

    pub fn user(&self) -> u8 {
        use CommandMessage::*;
        match &self {
            UndoPoint(user_id) => *user_id,
            CanvasResize(user_id, _) => *user_id,
            LayerCreate(user_id, _) => *user_id,
            LayerAttributes(user_id, _) => *user_id,
            LayerRetitle(user_id, _) => *user_id,
            LayerOrder(user_id, _) => *user_id,
            LayerDelete(user_id, _) => *user_id,
            LayerVisibility(user_id, _) => *user_id,
            PutImage(user_id, _) => *user_id,
            FillRect(user_id, _) => *user_id,
            PenUp(user_id) => *user_id,
            AnnotationCreate(user_id, _) => *user_id,
            AnnotationReshape(user_id, _) => *user_id,
            AnnotationEdit(user_id, _) => *user_id,
            AnnotationDelete(user_id, _) => *user_id,
            MoveRegion(user_id, _) => *user_id,
            PutTile(user_id, _) => *user_id,
            CanvasBackground(user_id, _) => *user_id,
            DrawDabsClassic(user_id, _) => *user_id,
            DrawDabsPixel(user_id, _) => *user_id,
            DrawDabsPixelSquare(user_id, _) => *user_id,
            Undo(user_id, _) => *user_id,
        }
    }
}

impl fmt::Display for CommandMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_text().fmt(f)
    }
}

impl From<CommandMessage> for Message {
    fn from(item: CommandMessage) -> Message {
        Message::Command(item)
    }
}

impl Message {
    pub fn deserialize(buf: &[u8]) -> Result<Message, DeserializationError> {
        if buf.len() < 4 {
            return Err(DeserializationError {
                user_id: 0,
                message_type: 0,
                payload_len: 0,
                error: "Message header too short",
            });
        }
        let payload_len = u16::from_be_bytes(buf[0..2].try_into().unwrap()) as usize;
        let message_type = buf[2];
        let user_id = buf[3];

        if buf.len() < 4 + payload_len {
            return Err(DeserializationError {
                user_id,
                message_type,
                payload_len,
                error: "Message truncated",
            });
        }

        let buf = &buf[4..];

        use Message::*;
        Ok(match message_type {
            0 => Control(ControlMessage::ServerCommand(
                user_id,
                MessageReader::new(&buf).read_remaining_str(),
            )),
            1 => Control(ControlMessage::Disconnect(
                user_id,
                DisconnectMessage::deserialize(&buf)?,
            )),
            2 => Control(ControlMessage::Ping(
                user_id,
                MessageReader::new(&buf)
                    .check_len(1, 1, 2, 0)?
                    .read::<bool>(),
            )),
            32 => ServerMeta(ServerMetaMessage::Join(
                user_id,
                JoinMessage::deserialize(&buf)?,
            )),
            33 => ServerMeta(ServerMetaMessage::Leave(user_id)),
            34 => ServerMeta(ServerMetaMessage::SessionOwner(
                user_id,
                MessageReader::new(&buf)
                    .check_len(0, 255, 34, 0)?
                    .read_remaining_vec(),
            )),
            35 => ServerMeta(ServerMetaMessage::Chat(
                user_id,
                ChatMessage::deserialize(&buf)?,
            )),
            36 => ServerMeta(ServerMetaMessage::TrustedUsers(
                user_id,
                MessageReader::new(&buf)
                    .check_len(0, 255, 36, 0)?
                    .read_remaining_vec(),
            )),
            37 => ServerMeta(ServerMetaMessage::SoftReset(user_id)),
            38 => ServerMeta(ServerMetaMessage::PrivateChat(
                user_id,
                PrivateChatMessage::deserialize(&buf)?,
            )),
            64 => ClientMeta(ClientMetaMessage::Interval(
                user_id,
                MessageReader::new(&buf)
                    .check_len(2, 2, 64, 0)?
                    .read::<u16>(),
            )),
            65 => ClientMeta(ClientMetaMessage::LaserTrail(
                user_id,
                LaserTrailMessage::deserialize(&buf)?,
            )),
            66 => ClientMeta(ClientMetaMessage::MovePointer(
                user_id,
                MovePointerMessage::deserialize(&buf)?,
            )),
            67 => ClientMeta(ClientMetaMessage::Marker(
                user_id,
                MessageReader::new(&buf).read_remaining_str(),
            )),
            68 => ClientMeta(ClientMetaMessage::UserACL(
                user_id,
                MessageReader::new(&buf)
                    .check_len(0, 255, 68, 0)?
                    .read_remaining_vec(),
            )),
            69 => ClientMeta(ClientMetaMessage::LayerACL(
                user_id,
                LayerACLMessage::deserialize(&buf)?,
            )),
            70 => ClientMeta(ClientMetaMessage::FeatureAccessLevels(
                user_id,
                MessageReader::new(&buf)
                    .check_len(9, 9, 70, 0)?
                    .read_remaining_vec(),
            )),
            71 => ClientMeta(ClientMetaMessage::DefaultLayer(
                user_id,
                MessageReader::new(&buf)
                    .check_len(2, 2, 71, 0)?
                    .read::<u16>(),
            )),
            72 => ClientMeta(ClientMetaMessage::Filtered(
                user_id,
                MessageReader::new(&buf).read_remaining_vec::<u8>(),
            )),
            128 => Command(CommandMessage::UndoPoint(user_id)),
            129 => Command(CommandMessage::CanvasResize(
                user_id,
                CanvasResizeMessage::deserialize(&buf)?,
            )),
            130 => Command(CommandMessage::LayerCreate(
                user_id,
                LayerCreateMessage::deserialize(&buf)?,
            )),
            131 => Command(CommandMessage::LayerAttributes(
                user_id,
                LayerAttributesMessage::deserialize(&buf)?,
            )),
            132 => Command(CommandMessage::LayerRetitle(
                user_id,
                LayerRetitleMessage::deserialize(&buf)?,
            )),
            133 => Command(CommandMessage::LayerOrder(
                user_id,
                MessageReader::new(&buf).read_remaining_vec(),
            )),
            134 => Command(CommandMessage::LayerDelete(
                user_id,
                LayerDeleteMessage::deserialize(&buf)?,
            )),
            135 => Command(CommandMessage::LayerVisibility(
                user_id,
                LayerVisibilityMessage::deserialize(&buf)?,
            )),
            136 => Command(CommandMessage::PutImage(
                user_id,
                PutImageMessage::deserialize(&buf)?,
            )),
            137 => Command(CommandMessage::FillRect(
                user_id,
                FillRectMessage::deserialize(&buf)?,
            )),
            140 => Command(CommandMessage::PenUp(user_id)),
            141 => Command(CommandMessage::AnnotationCreate(
                user_id,
                AnnotationCreateMessage::deserialize(&buf)?,
            )),
            142 => Command(CommandMessage::AnnotationReshape(
                user_id,
                AnnotationReshapeMessage::deserialize(&buf)?,
            )),
            143 => Command(CommandMessage::AnnotationEdit(
                user_id,
                AnnotationEditMessage::deserialize(&buf)?,
            )),
            144 => Command(CommandMessage::AnnotationDelete(
                user_id,
                MessageReader::new(&buf)
                    .check_len(2, 2, 144, 0)?
                    .read::<u16>(),
            )),
            145 => Command(CommandMessage::MoveRegion(
                user_id,
                MoveRegionMessage::deserialize(&buf)?,
            )),
            146 => Command(CommandMessage::PutTile(
                user_id,
                PutTileMessage::deserialize(&buf)?,
            )),
            147 => Command(CommandMessage::CanvasBackground(
                user_id,
                MessageReader::new(&buf).read_remaining_vec::<u8>(),
            )),
            148 => Command(CommandMessage::DrawDabsClassic(
                user_id,
                DrawDabsClassicMessage::deserialize(&buf)?,
            )),
            149 => Command(CommandMessage::DrawDabsPixel(
                user_id,
                DrawDabsPixelMessage::deserialize(&buf)?,
            )),
            150 => Command(CommandMessage::DrawDabsPixelSquare(
                user_id,
                DrawDabsPixelMessage::deserialize(&buf)?,
            )),
            255 => Command(CommandMessage::Undo(
                user_id,
                UndoMessage::deserialize(&buf)?,
            )),
            _ => {
                return Err(DeserializationError {
                    user_id,
                    message_type,
                    payload_len,
                    error: "Unknown message type",
                });
            }
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        use Message::*;
        match &self {
            Control(m) => m.serialize(),
            ServerMeta(m) => m.serialize(),
            ClientMeta(m) => m.serialize(),
            Command(m) => m.serialize(),
        }
    }

    pub fn from_text(tm: &TextMessage) -> Option<Message> {
        // tm.user_id
        use Message::*;
        Some(match tm.name.as_ref() {
            "servercommand" => Control(ControlMessage::ServerCommand(
                tm.user_id,
                tm.get_str("msg").to_string(),
            )),
            "disconnect" => Control(ControlMessage::Disconnect(
                tm.user_id,
                DisconnectMessage::from_text(&tm),
            )),
            "ping" => Control(ControlMessage::Ping(
                tm.user_id,
                tm.get_str("is_pong") == "true",
            )),
            "join" => ServerMeta(ServerMetaMessage::Join(
                tm.user_id,
                JoinMessage::from_text(&tm),
            )),
            "leave" => ServerMeta(ServerMetaMessage::Leave(tm.user_id)),
            "sessionowner" => ServerMeta(ServerMetaMessage::SessionOwner(
                tm.user_id,
                tm.get_vec_u8("users"),
            )),
            "chat" => ServerMeta(ServerMetaMessage::Chat(
                tm.user_id,
                ChatMessage::from_text(&tm),
            )),
            "trusted" => ServerMeta(ServerMetaMessage::TrustedUsers(
                tm.user_id,
                tm.get_vec_u8("users"),
            )),
            "softreset" => ServerMeta(ServerMetaMessage::SoftReset(tm.user_id)),
            "privatechat" => ServerMeta(ServerMetaMessage::PrivateChat(
                tm.user_id,
                PrivateChatMessage::from_text(&tm),
            )),
            "interval" => ClientMeta(ClientMetaMessage::Interval(tm.user_id, tm.get_u16("msecs"))),
            "lasertrail" => ClientMeta(ClientMetaMessage::LaserTrail(
                tm.user_id,
                LaserTrailMessage::from_text(&tm),
            )),
            "movepointer" => ClientMeta(ClientMetaMessage::MovePointer(
                tm.user_id,
                MovePointerMessage::from_text(&tm),
            )),
            "marker" => ClientMeta(ClientMetaMessage::Marker(
                tm.user_id,
                tm.get_str("text").to_string(),
            )),
            "useracl" => ClientMeta(ClientMetaMessage::UserACL(
                tm.user_id,
                tm.get_vec_u8("users"),
            )),
            "layeracl" => ClientMeta(ClientMetaMessage::LayerACL(
                tm.user_id,
                LayerACLMessage::from_text(&tm),
            )),
            "featureaccess" => ClientMeta(ClientMetaMessage::FeatureAccessLevels(
                tm.user_id,
                tm.get_vec_u8("feature_tiers"),
            )),
            "defaultlayer" => ClientMeta(ClientMetaMessage::DefaultLayer(
                tm.user_id,
                tm.get_u16("id"),
            )),
            "filtered" => ClientMeta(ClientMetaMessage::Filtered(
                tm.user_id,
                tm.get_bytes("message"),
            )),
            "undopoint" => Command(CommandMessage::UndoPoint(tm.user_id)),
            "resize" => Command(CommandMessage::CanvasResize(
                tm.user_id,
                CanvasResizeMessage::from_text(&tm),
            )),
            "newlayer" => Command(CommandMessage::LayerCreate(
                tm.user_id,
                LayerCreateMessage::from_text(&tm),
            )),
            "layerattr" => Command(CommandMessage::LayerAttributes(
                tm.user_id,
                LayerAttributesMessage::from_text(&tm),
            )),
            "retitlelayer" => Command(CommandMessage::LayerRetitle(
                tm.user_id,
                LayerRetitleMessage::from_text(&tm),
            )),
            "layerorder" => Command(CommandMessage::LayerOrder(
                tm.user_id,
                tm.get_vec_u16("layers"),
            )),
            "deletelayer" => Command(CommandMessage::LayerDelete(
                tm.user_id,
                LayerDeleteMessage::from_text(&tm),
            )),
            "layervisibility" => Command(CommandMessage::LayerVisibility(
                tm.user_id,
                LayerVisibilityMessage::from_text(&tm),
            )),
            "putimage" => Command(CommandMessage::PutImage(
                tm.user_id,
                PutImageMessage::from_text(&tm),
            )),
            "fillrect" => Command(CommandMessage::FillRect(
                tm.user_id,
                FillRectMessage::from_text(&tm),
            )),
            "penup" => Command(CommandMessage::PenUp(tm.user_id)),
            "newannotation" => Command(CommandMessage::AnnotationCreate(
                tm.user_id,
                AnnotationCreateMessage::from_text(&tm),
            )),
            "reshapeannotation" => Command(CommandMessage::AnnotationReshape(
                tm.user_id,
                AnnotationReshapeMessage::from_text(&tm),
            )),
            "editannotation" => Command(CommandMessage::AnnotationEdit(
                tm.user_id,
                AnnotationEditMessage::from_text(&tm),
            )),
            "deleteannotation" => Command(CommandMessage::AnnotationDelete(
                tm.user_id,
                tm.get_u16("id"),
            )),
            "moveregion" => Command(CommandMessage::MoveRegion(
                tm.user_id,
                MoveRegionMessage::from_text(&tm),
            )),
            "puttile" => Command(CommandMessage::PutTile(
                tm.user_id,
                PutTileMessage::from_text(&tm),
            )),
            "background" => Command(CommandMessage::CanvasBackground(
                tm.user_id,
                tm.get_bytes("image"),
            )),
            "classicdabs" => Command(CommandMessage::DrawDabsClassic(
                tm.user_id,
                DrawDabsClassicMessage::from_text(&tm),
            )),
            "pixeldabs" => Command(CommandMessage::DrawDabsPixel(
                tm.user_id,
                DrawDabsPixelMessage::from_text(&tm),
            )),
            "squarepixeldabs" => Command(CommandMessage::DrawDabsPixelSquare(
                tm.user_id,
                DrawDabsPixelMessage::from_text(&tm),
            )),
            "undo" => Command(CommandMessage::Undo(
                tm.user_id,
                UndoMessage::from_text(&tm),
            )),
            _ => {
                return None;
            }
        })
    }

    pub fn as_text(&self) -> TextMessage {
        use Message::*;
        match &self {
            Control(m) => m.as_text(),
            ServerMeta(m) => m.as_text(),
            ClientMeta(m) => m.as_text(),
            Command(m) => m.as_text(),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_text().fmt(f)
    }
}
