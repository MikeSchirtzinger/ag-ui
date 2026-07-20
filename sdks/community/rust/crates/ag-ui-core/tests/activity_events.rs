use ag_ui_core::event::{Event, EventType};
use ag_ui_core::types::Role;
use serde_json::json;

#[test]
fn activity_snapshot_round_trips() {
    let wire = json!({
        "type": "ACTIVITY_SNAPSHOT",
        "timestamp": 42.25,
        "rawEvent": {"provider": "test"},
        "messageId": "00000000-0000-0000-0000-000000000001",
        "activityType": "search",
        "content": {"query": "rust serde", "matches": 3},
        "replace": false
    });

    let event: Event = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(event.event_type(), EventType::ActivitySnapshot);
    assert_eq!(event.timestamp(), Some(42.25));
    assert_eq!(serde_json::to_value(&event).unwrap(), wire);

    let encoded = serde_json::to_vec(&event).unwrap();
    let decoded: Event = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, event);
}

#[test]
fn activity_snapshot_defaults_replace_to_true() {
    let event: Event = serde_json::from_value(json!({
        "type": "ACTIVITY_SNAPSHOT",
        "messageId": "00000000-0000-0000-0000-000000000001",
        "activityType": "search",
        "content": {"query": "rust serde"}
    }))
    .unwrap();

    let serialized = serde_json::to_value(event).unwrap();
    assert_eq!(serialized["replace"], true);
}

#[test]
fn activity_delta_round_trips() {
    let wire = json!({
        "type": "ACTIVITY_DELTA",
        "timestamp": 43.5,
        "messageId": "00000000-0000-0000-0000-000000000001",
        "activityType": "search",
        "patch": [
            {"op": "replace", "path": "/matches", "value": 4},
            {"op": "add", "path": "/complete", "value": true}
        ]
    });

    let event: Event = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(event.event_type(), EventType::ActivityDelta);
    assert_eq!(event.timestamp(), Some(43.5));
    assert_eq!(serde_json::to_value(&event).unwrap(), wire);

    let encoded = serde_json::to_vec(&event).unwrap();
    let decoded: Event = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, event);
}

#[test]
fn activity_role_round_trips_as_lowercase() {
    let encoded = serde_json::to_string(&Role::Activity).unwrap();
    assert_eq!(encoded, r#""activity""#);

    let decoded: Role = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, Role::Activity);
}
