# G15 — Design objects and authoring commands

> **Phase:** Design  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G14

## Outcome

**[REQUIRED]** Deliver the first useful Design workspace over the shared project graph.

## Scope

- Implement document, canvas, frame/page, group, vector, text, image reference, style, and selection types for a narrow professional workflow.
- Define constraints, coordinate spaces, transforms, stacking, and workspace command behavior.
- Build inspector and object tree using the module SDK.

## Exclusions

- Do not chase Illustrator, InDesign, or Figma parity.
- Do not add workspace-only persistence or an alternate undo system.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Users can create, select, edit, group, reorder, duplicate, and delete core Design objects.
- **[REQUIRED]** Objects remain accessible through shared graph/history APIs and survive reopening.
- **[REQUIRED]** Accessibility and keyboard operation cover primary editing actions.

## Verification

- Test command replay, undo/redo, invalid geometry, selection after remote update, and object identity across workspace switch.
- Use golden fixtures for representative vector/text/image compositions.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Design lead owns object semantics. Canvas, typography, and components agents work in parallel only after coordinate and object contracts are stable.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
