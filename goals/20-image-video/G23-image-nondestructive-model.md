# G23 — Nondestructive Image object model

> **Phase:** Image and Video  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G01, G03, G05, G06, G07

## Outcome

**[REQUIRED]** Represent image edits as editable graph operations while preserving source bytes.

## Scope

- Define image source, layer, mask, crop, adjustment, retouch, effect, and output objects for the first professional workflow.
- Represent parameters, provenance, source revision, and mask links as project data.
- Define unsupported/expensive operation handling and render invalidation.

## Exclusions

- Do not implement every Photoshop feature or flatten edits into a replacement image.
- Do not use model-generated pixels as the only record of an AI edit.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Original image bytes remain unchanged and supported edits are reproducible from graph state.
- **[REQUIRED]** A user can adjust, disable, reorder, and undo supported transforms.
- **[REQUIRED]** History identifies source and parameters for each derived result.

## Verification

- Replay edits from historical fixtures and compare canonical graph state.
- Test mask references, reordering, undo, corrupt source, and missing derived cache.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Image model lead owns node/command semantics. Rendering agents use a frozen graph contract.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
