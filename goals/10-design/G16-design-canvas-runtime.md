# G16 — Design canvas rendering and interaction

> **Phase:** Design  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G11, G15

## Outcome

**[REQUIRED]** Make the Design canvas responsive and usable for real documents.

## Scope

- Implement viewport navigation, zoom/pan, hit testing, selection handles, transform gestures, snapping basics, and incremental scene loading.
- Use the renderer selected by G11 and cache/virtualize only with measurable invalidation rules.
- Define fallback behavior for unsupported GPU paths.

## Exclusions

- Do not couple renderer state to canonical graph state.
- Do not add advanced effects before correctness and scale behavior are measured.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Canvas interaction remains correct for representative multi-frame and large-object fixtures.
- **[REQUIRED]** Edits use normal commands and are not lost on renderer restart.
- **[REQUIRED]** Rendering performance and memory limits have documented test thresholds.

## Verification

- Run golden rendering and hit-test geometry fixtures across supported platforms.
- Profile cold open, zoom/pan, selection, and edit under representative large-document fixtures.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Graphics lead owns scene evaluation and rendering contracts; input/gesture implementation can parallelize after those contracts are accepted.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
