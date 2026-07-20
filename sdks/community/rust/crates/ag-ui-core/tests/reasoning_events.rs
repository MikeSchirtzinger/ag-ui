//! Event-shape tests for the reasoning-event spec catch-up.
//!
//! Mirrors the round-trip / JSON-literal style used in tests/unit.rs.

#[cfg(test)]
mod tests {
    use ag_ui_core::JsonValue;
    use ag_ui_core::event::{
        BaseEvent, Event as AgUiEvent, EventType, ReasoningEncryptedValueEvent,
        ReasoningEncryptedValueSubtype, ReasoningEndEvent, ReasoningMessageChunkEvent,
        ReasoningMessageContentEvent, ReasoningMessageEndEvent, ReasoningMessageStartEvent,
        ReasoningStartEvent,
    };
    use ag_ui_core::types::{MessageId, Role};

    fn base() -> BaseEvent {
        BaseEvent {
            timestamp: None,
            raw_event: None,
        }
    }

    #[test]
    fn test_reasoning_start_and_end_roundtrip() {
        let message_id = MessageId::random();

        let start = AgUiEvent::<JsonValue>::ReasoningStart(ReasoningStartEvent {
            base: base(),
            message_id: message_id.clone(),
        });
        assert_eq!(start.event_type(), EventType::ReasoningStart);
        let start_json = serde_json::to_string(&start).unwrap();
        assert!(start_json.contains(r#""type":"REASONING_START""#));
        assert_eq!(start, serde_json::from_str(&start_json).unwrap());

        let end = AgUiEvent::<JsonValue>::ReasoningEnd(ReasoningEndEvent {
            base: base(),
            message_id,
        });
        assert_eq!(end.event_type(), EventType::ReasoningEnd);
        let end_json = serde_json::to_string(&end).unwrap();
        assert!(end_json.contains(r#""type":"REASONING_END""#));
        assert_eq!(end, serde_json::from_str(&end_json).unwrap());
    }

    #[test]
    fn test_reasoning_message_start_role_is_reasoning() {
        let event = ReasoningMessageStartEvent {
            base: base(),
            message_id: MessageId::random(),
            role: Role::Reasoning,
        };
        let wrapped = AgUiEvent::<JsonValue>::ReasoningMessageStart(event);
        assert_eq!(wrapped.event_type(), EventType::ReasoningMessageStart);

        let json_str = serde_json::to_string(&wrapped).unwrap();
        assert!(json_str.contains(r#""role":"reasoning""#));
        assert_eq!(wrapped, serde_json::from_str(&json_str).unwrap());
    }

    #[test]
    fn test_reasoning_message_start_role_defaults_when_omitted() {
        let json_str = format!(
            r#"{{"type":"REASONING_MESSAGE_START","messageId":"{}"}}"#,
            MessageId::random()
        );
        let event: AgUiEvent = serde_json::from_str(&json_str).unwrap();
        match event {
            AgUiEvent::ReasoningMessageStart(e) => assert_eq!(e.role, Role::Reasoning),
            other => panic!("expected ReasoningMessageStart, got {other:?}"),
        }
    }

    #[test]
    fn test_reasoning_message_content_and_end_roundtrip() {
        let message_id = MessageId::random();

        let content =
            AgUiEvent::<JsonValue>::ReasoningMessageContent(ReasoningMessageContentEvent {
                base: base(),
                message_id: message_id.clone(),
                delta: "Let me think about this...".to_string(),
            });
        assert_eq!(content.event_type(), EventType::ReasoningMessageContent);
        let content_json = serde_json::to_string(&content).unwrap();
        assert!(content_json.contains(r#""type":"REASONING_MESSAGE_CONTENT""#));
        assert_eq!(content, serde_json::from_str(&content_json).unwrap());

        let end = AgUiEvent::<JsonValue>::ReasoningMessageEnd(ReasoningMessageEndEvent {
            base: base(),
            message_id,
        });
        assert_eq!(end.event_type(), EventType::ReasoningMessageEnd);
        let end_json = serde_json::to_string(&end).unwrap();
        assert!(end_json.contains(r#""type":"REASONING_MESSAGE_END""#));
        assert_eq!(end, serde_json::from_str(&end_json).unwrap());
    }

    #[test]
    fn test_reasoning_message_chunk_all_fields_optional() {
        let event: AgUiEvent =
            serde_json::from_str(r#"{"type":"REASONING_MESSAGE_CHUNK"}"#).unwrap();
        match event {
            AgUiEvent::ReasoningMessageChunk(e) => {
                assert!(e.message_id.is_none());
                assert!(e.delta.is_none());
            }
            other => panic!("expected ReasoningMessageChunk, got {other:?}"),
        }

        let full = AgUiEvent::<JsonValue>::ReasoningMessageChunk(ReasoningMessageChunkEvent {
            base: base(),
            message_id: Some(MessageId::random()),
            delta: Some("partial".to_string()),
        });
        let json_str = serde_json::to_string(&full).unwrap();
        assert!(json_str.contains(r#""type":"REASONING_MESSAGE_CHUNK""#));
        assert_eq!(full, serde_json::from_str(&json_str).unwrap());
    }

    #[test]
    fn test_reasoning_encrypted_value_roundtrip_and_subtypes() {
        for (subtype, expected) in [
            (
                ReasoningEncryptedValueSubtype::ToolCall,
                r#""subtype":"tool-call""#,
            ),
            (
                ReasoningEncryptedValueSubtype::Message,
                r#""subtype":"message""#,
            ),
        ] {
            let event =
                AgUiEvent::<JsonValue>::ReasoningEncryptedValue(ReasoningEncryptedValueEvent {
                    base: base(),
                    subtype,
                    entity_id: "entity-123".to_string(),
                    encrypted_value: "opaque-blob".to_string(),
                });
            assert_eq!(event.event_type(), EventType::ReasoningEncryptedValue);
            let json_str = serde_json::to_string(&event).unwrap();
            assert!(
                json_str.contains(expected),
                "{json_str} should contain {expected}"
            );
            assert!(json_str.contains(r#""type":"REASONING_ENCRYPTED_VALUE""#));
            assert!(json_str.contains(r#""entityId":"entity-123""#));
            assert!(json_str.contains(r#""encryptedValue":"opaque-blob""#));
            assert_eq!(event, serde_json::from_str(&json_str).unwrap());
        }
    }

    #[test]
    fn test_reasoning_event_type_tags() {
        let cases = [
            (EventType::ReasoningStart, "\"REASONING_START\""),
            (
                EventType::ReasoningMessageStart,
                "\"REASONING_MESSAGE_START\"",
            ),
            (
                EventType::ReasoningMessageContent,
                "\"REASONING_MESSAGE_CONTENT\"",
            ),
            (EventType::ReasoningMessageEnd, "\"REASONING_MESSAGE_END\""),
            (
                EventType::ReasoningMessageChunk,
                "\"REASONING_MESSAGE_CHUNK\"",
            ),
            (EventType::ReasoningEnd, "\"REASONING_END\""),
            (
                EventType::ReasoningEncryptedValue,
                "\"REASONING_ENCRYPTED_VALUE\"",
            ),
        ];
        for (event_type, expected_tag) in cases {
            let json_str = serde_json::to_string(&event_type).unwrap();
            assert_eq!(json_str, expected_tag);
        }
    }
}
