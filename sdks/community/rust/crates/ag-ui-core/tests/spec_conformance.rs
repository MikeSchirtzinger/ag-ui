//! Spec-conformance / drift-detection test for `ag_ui_core::event::EventType`.
//!
//! `ag-ui-core` is a hand-maintained Rust port of the AG-UI protocol, whose
//! canonical source of truth is the Zod schemas in the TypeScript SDK
//! (`sdks/typescript/packages/core/src/events.ts`, this same repo). There is
//! no mechanical link between the two, so the Rust port can silently drift out
//! of sync with the spec — events get added or renamed in `events.ts` and the
//! Rust `EventType` enum quietly falls behind. This file is the trip wire.
//!
//! **How it works.** `event_types.snapshot.json` under `tests/fixtures/spec/`
//! is a checked-in copy of the canonical `EventType` names, extracted from the
//! TypeScript SDK's `EventType` enum (refresh procedure in the sibling
//! `REFRESH.md`). This test compares that snapshot against the Rust
//! `EventType` enum. It runs in ordinary Rust CI with no TypeScript toolchain,
//! because the snapshot is a checked-in fact, refreshed deliberately.
//!
//! **How acknowledged gaps work.** If a spec event type is intentionally not
//! implemented yet, it must be listed in `SPEC_EVENTS_NOT_YET_PORTED`. The test
//! asserts the Rust enum equals `snapshot − not_yet_ported`. The list is empty
//! while Rust has full parity with the snapshot; adding a deferral or porting a
//! deferred event requires updating it explicitly.

#[cfg(test)]
mod tests {
    use ag_ui_core::event::EventType;
    use std::collections::BTreeSet;

    /// Spec `EventType`s that exist in `events.ts` but are **not yet ported**
    /// to this crate's `EventType` enum.
    ///
    /// This is a truthful record of the current gap between the Rust SDK and
    /// the canonical TypeScript spec, not a permanent exclusion. As each event
    /// is implemented in `event.rs`, delete its entry here (the test below will
    /// fail until you do, which is the point). When this list is empty, the
    /// Rust `EventType` enum is at full parity with the spec's event set.
    const SPEC_EVENTS_NOT_YET_PORTED: &[&str] = &[];

    #[derive(serde::Deserialize)]
    struct SpecSnapshot {
        event_types: Vec<String>,
    }

    /// The canonical AG-UI spec event-type names, vendored from `events.ts`.
    fn canonical_spec_event_types() -> BTreeSet<String> {
        let snapshot: SpecSnapshot =
            serde_json::from_str(include_str!("fixtures/spec/event_types.snapshot.json"))
                .expect("vendored spec snapshot must be valid JSON — see fixtures/spec/REFRESH.md");
        snapshot.event_types.into_iter().collect()
    }

    /// `EventType`'s wire tag, e.g. `EventType::TextMessageStart` -> `"TEXT_MESSAGE_START"`.
    fn event_type_tag(event_type: EventType) -> String {
        serde_json::to_string(&event_type)
            .unwrap()
            .trim_matches('"')
            .to_string()
    }

    /// Every `EventType` variant's wire tag, exhaustively.
    ///
    /// The `match` below is the enforcement mechanism, not documentation: if
    /// `EventType` gains, loses, or renames a variant without this function
    /// being updated to match, the crate fails to compile. That is what makes
    /// the drift-alarm test reliable rather than something that can silently go
    /// stale on its own.
    fn all_event_type_tags() -> Vec<String> {
        use EventType::*;
        let variants = [
            TextMessageStart,
            TextMessageContent,
            TextMessageEnd,
            TextMessageChunk,
            ThinkingTextMessageStart,
            ThinkingTextMessageContent,
            ThinkingTextMessageEnd,
            ToolCallStart,
            ToolCallArgs,
            ToolCallEnd,
            ToolCallChunk,
            ToolCallResult,
            ThinkingStart,
            ThinkingEnd,
            StateSnapshot,
            StateDelta,
            MessagesSnapshot,
            Raw,
            Custom,
            RunStarted,
            RunFinished,
            RunError,
            StepStarted,
            StepFinished,
            ReasoningStart,
            ReasoningMessageStart,
            ReasoningMessageContent,
            ReasoningMessageEnd,
            ReasoningMessageChunk,
            ReasoningEnd,
            ReasoningEncryptedValue,
            ActivitySnapshot,
            ActivityDelta,
        ];
        for variant in variants {
            match variant {
                TextMessageStart
                | TextMessageContent
                | TextMessageEnd
                | TextMessageChunk
                | ThinkingTextMessageStart
                | ThinkingTextMessageContent
                | ThinkingTextMessageEnd
                | ToolCallStart
                | ToolCallArgs
                | ToolCallEnd
                | ToolCallChunk
                | ToolCallResult
                | ThinkingStart
                | ThinkingEnd
                | StateSnapshot
                | StateDelta
                | MessagesSnapshot
                | Raw
                | Custom
                | RunStarted
                | RunFinished
                | RunError
                | StepStarted
                | StepFinished
                | ReasoningStart
                | ReasoningMessageStart
                | ReasoningMessageContent
                | ReasoningMessageEnd
                | ReasoningMessageChunk
                | ReasoningEnd
                | ReasoningEncryptedValue
                | ActivitySnapshot
                | ActivityDelta => {}
            }
        }
        variants.into_iter().map(event_type_tag).collect()
    }

    #[test]
    fn event_type_matches_canonical_spec_snapshot() {
        let canonical = canonical_spec_event_types();
        let not_yet_ported: BTreeSet<String> = SPEC_EVENTS_NOT_YET_PORTED
            .iter()
            .map(|s| s.to_string())
            .collect();
        let rust_types: BTreeSet<String> = all_event_type_tags().into_iter().collect();

        // 1. Every deferral must be a real spec event type; otherwise the list
        //    is lying about what it's tracking.
        let bogus_deferrals: Vec<_> = not_yet_ported.difference(&canonical).collect();
        assert!(
            bogus_deferrals.is_empty(),
            "SPEC_EVENTS_NOT_YET_PORTED names event types that are not in the \
             spec snapshot: {bogus_deferrals:?} — remove them or fix the name."
        );

        // 2. An event that is now implemented in Rust but still listed as
        //    not-yet-ported: delete it from the list.
        let ported_but_still_listed: Vec<_> = rust_types.intersection(&not_yet_ported).collect();
        assert!(
            ported_but_still_listed.is_empty(),
            "these EventType variants are implemented but still listed in \
             SPEC_EVENTS_NOT_YET_PORTED: {ported_but_still_listed:?} — remove \
             each from that list now that it's ported."
        );

        // 3. The real drift alarm: the Rust enum must equal the spec set minus
        //    the acknowledged gap. A spec event that is neither implemented nor
        //    listed as pending is undocumented drift; a Rust variant absent from
        //    the spec is either a rename or a non-spec extension.
        let expected: BTreeSet<String> = canonical.difference(&not_yet_ported).cloned().collect();
        let missing_from_rust: Vec<_> = expected.difference(&rust_types).collect();
        let extra_in_rust: Vec<_> = rust_types.difference(&expected).collect();
        assert!(
            missing_from_rust.is_empty() && extra_in_rust.is_empty(),
            "EventType has drifted from the vendored spec snapshot \
             (tests/fixtures/spec/event_types.snapshot.json). \
             Spec event types missing from Rust (add them, or list them in \
             SPEC_EVENTS_NOT_YET_PORTED if intentionally deferred): \
             {missing_from_rust:?}. Rust variants not in the spec snapshot \
             (a rename, or refresh the snapshot — see fixtures/spec/REFRESH.md): \
             {extra_in_rust:?}."
        );
    }
}
