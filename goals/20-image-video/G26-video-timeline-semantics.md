# G26 — Video timeline and editing semantics

> **Phase:** Image and Video  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G07, G25

## Outcome

**[REQUIRED]** Deliver a reliable first nonlinear editing workflow using operations over a shared project.

## Scope

- Define sequence, clip, track, trim, split, move, ripple, transition, keyframe, and timebase semantics.
- Implement timeline editing, snapping, selection, playback synchronization, transcript/marker anchors, and basic captions.
- Specify conflict behavior for concurrent timeline edits.

## Exclusions

- Do not implement multicamera, advanced color, or complex VFX in the first slice.
- Do not encode timeline order as unstable array indices without a migration strategy.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Users can create a sequence, arrange clips, trim/split/ripple, add transitions/captions, and undo/replay edits.
- **[REQUIRED]** Timebase and frame rounding behavior is explicit and deterministic.
- **[REQUIRED]** Concurrent edits have defined outcomes or actionable conflict review.

## Verification

- Property-test clip bounds and timeline invariants; replay edit sequences and compare canonical timelines.
- Test offline conflict, variable frame rates, timebase conversion, and golden exports.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Timeline semantics have one owner. UI and playback agents parallelize after operation and timebase contracts are accepted.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
