use std::borrow::Cow;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Message<'a> {
    pub user_id: u32,
    pub user_name: Cow<'a, str>,
    pub content: Cow<'a, str>,
    pub color: ratatui::style::Color,
}
impl Message<'_> {
    pub fn error(msg: &str) -> Self {
        Self {
            user_id: 0,
            user_name: "ERROR".into(),
            content: msg.to_string().into(),
            color: ratatui::style::Color::Red,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = serde_json::to_vec(self).unwrap();
        bytes.push(b'\n');
        bytes
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct NewUserSet<'a> {
    pub user_name: Cow<'a, str>,
    pub color: ratatui::style::Color,
}

pub fn serialize<T: serde::Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).unwrap();
    bytes.push(b'\n');
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_serde() {
        let message = Message {
            user_id: 12,
            user_name: "BOB".into(),
            content: "hee".into(),
            color: ratatui::style::Color::Red,
        };
        let serialized = serde_json::to_vec(&message).unwrap();
        let deserialized: Message = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(deserialized.user_id, 12);
        assert_eq!(deserialized.user_name, "BOB");
        assert_eq!(deserialized.content, "hee");
        assert_eq!(deserialized.color, ratatui::style::Color::Red);
    }
}
