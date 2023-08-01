use std::borrow::Cow;




#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Message<'a> {
    pub user: u32,
    pub content: Cow<'a, str>,
}
impl Message <'_> {
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = serde_json::to_vec(self).unwrap();
        bytes.push(b'\n');
        bytes
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_serde() {
        let message = Message {
            user: 12,
            content: "hee".into(),
        };
        let serialized = serde_json::to_vec(&message).unwrap();
        let deserialized: Message = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(deserialized.user, 12);
        assert_eq!(deserialized.content, "hee");
    }
}
