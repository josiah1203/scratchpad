# G21 — Native iOS and iPadOS Design client

> **Phase:** Design  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G09, G15, G16

## Outcome

**[REQUIRED]** Make core Design work genuinely useful on iPhone and iPad.

## Scope

- Build native project browsing/opening, frame selection, text edits, object transforms, comments/review, and camera/photo ingest.
- Bind SwiftUI/UIKit interactions to shared Rust runtime without duplicating graph semantics.
- Support touch, Pencil where relevant, safe areas, lifecycle, storage pressure, and intermittent network.

## Exclusions

- Do not ship a desktop remote control or shrink desktop UI to fit.
- Do not claim feature parity until task coverage is measured.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A user can complete defined creation/review tasks on phone and tablet, including offline edits.
- **[REQUIRED]** Project state converges with desktop and recovers after app termination.
- **[REQUIRED]** Task coverage and platform-specific gaps are documented.

## Verification

- Run device/simulator tests for offline, lifecycle termination, Pencil/touch, camera import, and sync.
- Review accessibility and memory/battery behavior on representative hardware.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** iOS lead owns native UI and bindings. Shared runtime contracts remain owned by platform teams.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
