#[cfg(test)]
mod message_tests {
    use crate::message::{Message, Role};

    #[test]
    fn test_system_message() {
        let msg = Message::system("You are helpful");
        assert_eq!(msg.role, Role::System);
        assert_eq!(msg.content, "You are helpful");
        assert!(msg.tool_calls.is_none());
    }

    #[test]
    fn test_user_message() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_assistant_message() {
        let msg = Message::assistant("I can help");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content, "I can help");
    }

    #[test]
    fn test_tool_message() {
        let msg = Message::tool("result", "call-123");
        assert_eq!(msg.role, Role::Tool);
        assert_eq!(msg.content, "result");
        assert_eq!(msg.tool_call_id.as_deref(), Some("call-123"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let msg = Message::user("test");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, Role::User);
        assert_eq!(deserialized.content, "test");
    }
}

#[cfg(test)]
mod provider_tests {
    use super::super::provider::LlmProvider;

    #[test]
    fn test_provider_trailing_slash() {
        let p = LlmProvider::new("http://localhost:8080/v1/", "key");
        assert_eq!(p.base_url(), "http://localhost:8080/v1");
    }

    #[test]
    fn test_provider_no_trailing_slash() {
        let p = LlmProvider::new("http://localhost:8080/v1", "key");
        assert_eq!(p.base_url(), "http://localhost:8080/v1");
    }

    #[test]
    fn test_provider_api_key() {
        let p = LlmProvider::new("http://localhost", "my-secret-key");
        assert_eq!(p.api_key(), "my-secret-key");
    }
}
