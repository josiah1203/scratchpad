# G22 — Native Android Design client

> **Phase:** Design  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G09, G15, G16

## Outcome

**[REQUIRED]** Make core Design work genuinely useful on Android phones and tablets.

## Scope

- Build native project browsing/opening, frame selection, text edits, object transforms, comments/review, and camera/photo ingest.
- Bind Compose/native interactions to shared runtime semantics.
- Support stylus where available, adaptive layouts, background lifecycle, storage pressure, and intermittent network.

## Exclusions

- Do not ship a browser wrapper as the only creative experience.
- Do not fork graph or command behavior for Android.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A user can complete defined phone/tablet tasks, including offline edits.
- **[REQUIRED]** Android and desktop project state converge and recover after process death.
- **[REQUIRED]** Device-specific capabilities and parity gaps are documented.

## Verification

- Run emulator/device tests for process death, background upload, offline sync, import, accessibility, and resource pressure.
- Compare shared operation traces from Android and desktop for equivalent edits.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Android lead owns native UI and bindings. Coordinate shared API changes through the runtime contract owner.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
