use ag_ui_core::JsonValue;
use ag_ui_core::event::{
    BaseEvent, Event as AgUiEvent, EventValidationError, RunFinishedEvent, RunFinishedOutcome,
    RunStartedEvent, TextMessageChunkEvent, TextMessageStartEvent,
};
use ag_ui_core::types::{Interrupt, MessageId, Role, RunAgentInput, RunId, ThreadId, ToolCallId};
use serde_json::json;

fn base_event() -> BaseEvent {
    BaseEvent {
        timestamp: None,
        raw_event: None,
    }
}

#[test]
fn text_message_start_name_roundtrips_and_role_defaults() {
    let event = TextMessageStartEvent::new(MessageId::random()).with_name("Ada");
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""name":"Ada""#));
    assert_eq!(event, serde_json::from_str(&json).unwrap());

    let legacy = format!(r#"{{"messageId":"{}"}}"#, MessageId::random());
    let decoded: TextMessageStartEvent = serde_json::from_str(&legacy).unwrap();
    assert_eq!(decoded.role, Role::Assistant);
    assert_eq!(decoded.name, None);
}

#[test]
fn text_message_chunk_optional_role_and_name_roundtrip() {
    let empty: AgUiEvent = serde_json::from_str(r#"{"type":"TEXT_MESSAGE_CHUNK"}"#).unwrap();
    match empty {
        AgUiEvent::TextMessageChunk(event) => {
            assert_eq!(event.role, None);
            assert_eq!(event.name, None);
        }
        other => panic!("expected TextMessageChunk, got {other:?}"),
    }

    let event = AgUiEvent::<JsonValue>::TextMessageChunk(TextMessageChunkEvent {
        base: base_event(),
        message_id: Some(MessageId::random()),
        role: Some(Role::Assistant),
        delta: Some("hi".to_string()),
        name: Some("Ada".to_string()),
    });
    let json = serde_json::to_string(&event).unwrap();
    assert_eq!(event, serde_json::from_str(&json).unwrap());
}

#[test]
fn run_started_parent_and_input_roundtrip() {
    let thread_id = ThreadId::random();
    let run_id = RunId::random();
    let input = RunAgentInput::new(
        thread_id.clone(),
        run_id.clone(),
        json!({"counter": 1}),
        vec![],
        vec![],
        vec![],
        json!({"source": "test"}),
    );
    let event = AgUiEvent::<JsonValue>::RunStarted(RunStartedEvent {
        base: base_event(),
        thread_id,
        run_id,
        parent_run_id: Some(RunId::random()),
        input: Some(input),
    });
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("parentRunId"));
    assert!(json.contains(r#""input""#));
    assert_eq!(event, serde_json::from_str(&json).unwrap());

    let legacy = format!(
        r#"{{"type":"RUN_STARTED","threadId":"{}","runId":"{}"}}"#,
        ThreadId::random(),
        RunId::random()
    );
    let decoded: AgUiEvent = serde_json::from_str(&legacy).unwrap();
    match decoded {
        AgUiEvent::RunStarted(event) => {
            assert_eq!(event.parent_run_id, None);
            assert_eq!(event.input, None);
        }
        other => panic!("expected RunStarted, got {other:?}"),
    }
}

#[test]
fn run_finished_outcomes_roundtrip() {
    for outcome in [
        RunFinishedOutcome::Success,
        RunFinishedOutcome::Interrupt {
            interrupts: vec![Interrupt::new("int-1", "needs_approval")],
        },
    ] {
        let event = AgUiEvent::<JsonValue>::RunFinished(RunFinishedEvent {
            base: base_event(),
            thread_id: ThreadId::random(),
            run_id: RunId::random(),
            result: None,
            outcome: Some(outcome),
        });
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(event, serde_json::from_str(&json).unwrap());
    }

    let legacy = format!(
        r#"{{"type":"RUN_FINISHED","threadId":"{}","runId":"{}"}}"#,
        ThreadId::random(),
        RunId::random()
    );
    let decoded: AgUiEvent = serde_json::from_str(&legacy).unwrap();
    match decoded {
        AgUiEvent::RunFinished(event) => assert_eq!(event.outcome, None),
        other => panic!("expected RunFinished, got {other:?}"),
    }
}

#[test]
fn run_finished_outcome_validation_accepts_valid_and_rejects_empty_interrupts() {
    assert!(RunFinishedOutcome::Success.validate().is_ok());

    let interrupt = RunFinishedOutcome::Interrupt {
        interrupts: vec![Interrupt::new("int-1", "needs_approval")],
    };
    assert!(interrupt.validate().is_ok());

    let empty = RunFinishedOutcome::Interrupt { interrupts: vec![] };
    assert!(matches!(
        empty.validate(),
        Err(EventValidationError::EmptyInterrupts)
    ));
}

#[test]
fn interrupt_optional_fields_roundtrip_with_lab_wire_names() {
    let interrupt = Interrupt {
        id: "int-42".to_string(),
        reason: "requires_human_review".to_string(),
        message: Some("Please confirm this action.".to_string()),
        tool_call_id: Some(ToolCallId::random()),
        response_schema: Some(json!({"type": "boolean"})),
        expires_at: Some("2026-08-01T00:00:00Z".to_string()),
        metadata: Some(json!({"priority": "high"})),
    };
    let json = serde_json::to_string(&interrupt).unwrap();
    assert!(json.contains("toolCallId"));
    assert!(json.contains("responseSchema"));
    assert!(json.contains("expiresAt"));
    let decoded: Interrupt = serde_json::from_str(&json).unwrap();
    assert_eq!(interrupt, decoded);
}
