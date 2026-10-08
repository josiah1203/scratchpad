# G33 — Audio workspace and transcript editing

> **Phase:** Platform expansion  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G25, G26, G27

## Outcome

**[REQUIRED]** Provide multitrack audio editing with transcript and waveform as linked views of the same material.

## Scope

- Implement session, track, clip, waveform, transcript, speaker, cleanup, level, and export objects for a bounded podcast workflow.
- Connect transcript edits to audio timeline operations and preserve original recordings.
- Support DSP controls and AI-assisted cleanup as editable, attributable operations.

## Exclusions

- Do not ship advanced mastering or every DAW workflow in this goal.
- Do not alter source recordings destructively.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A user can import/record, trim, clean, transcribe, edit, and export a podcast session.
- **[REQUIRED]** Transcript and waveform changes remain synchronized and undoable.
- **[REQUIRED]** AI cleanup is represented as an editable effect or derived node.

## Verification

- Test sample-accurate trim semantics, transcript alignment, speaker changes, interruption, export, and audio golden metrics.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Audio lead owns timebase/session semantics. Speech/AI tasks can be isolated services using audio command contracts.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
