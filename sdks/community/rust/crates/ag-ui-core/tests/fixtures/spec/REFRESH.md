# Refreshing the vendored EventType snapshot

`event_types.snapshot.json` is a checked-in copy of the AG-UI spec's canonical
event-type names, extracted from the TypeScript SDK's `EventType` enum:

    sdks/typescript/packages/core/src/events.ts

It exists so `tests/spec_conformance.rs`'s drift-alarm test
(`event_type_matches_canonical_spec_snapshot`) can run in ordinary Rust CI with
no TypeScript toolchain: it compares the Rust `EventType` enum against this
file, accounting for the spec events listed in that test's
`SPEC_EVENTS_NOT_YET_PORTED` allow-list.

## When to refresh

Whenever the spec's `events.ts` changes its `EventType` enum — periodically, or
whenever doing a catch-up pass on `ag-ui-core`.

## How to refresh

1. From the repo root, extract the current `EventType` member names:

   ```
   sed -n '/export enum EventType {/,/^}/p' \
     sdks/typescript/packages/core/src/events.ts \
     | grep -oE '"[A-Z_]+"' | tr -d '"' | sort -u
   ```

2. Replace `event_types.snapshot.json`'s `event_types` array with that list,
   and update `source_file_commit` (the commit that last touched `events.ts`)
   and `fetched`.

3. Run `cargo test -p ag-ui-core --test spec_conformance`.
   - **Fails with events "missing from Rust"** → the spec added event types the
     Rust `EventType` enum doesn't have. Either implement them in `event.rs`, or
     — if you're only refreshing the snapshot and deferring the port — add them
     to `SPEC_EVENTS_NOT_YET_PORTED` in `tests/spec_conformance.rs`. Either way
     the gap is now visible rather than silent.
   - **Fails with events "not in the spec snapshot"** → a Rust variant was
     renamed relative to the spec, or the snapshot is stale for that name.
   - **Passes** → the crate's event set is in sync with the spec (modulo the
     acknowledged `SPEC_EVENTS_NOT_YET_PORTED` gap); commit the refreshed
     snapshot.

## Design note

This file is intentionally not fetched live in CI: `ag-ui-core` has no
TypeScript toolchain dependency, and a live network fetch inside a test isn't
reproducible. Committing the snapshot turns drift into a *visible, deliberate*
refresh step instead of a silent, invisible one — which is the point of the
test existing at all.
