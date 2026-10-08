# G18 — Reusable components and brand system

> **Phase:** Design  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G15, G17

## Outcome

**[REQUIRED]** Let teams define shared components, styles, colors, typography, and brand assets.

## Scope

- Implement component definitions, instances, variants/overrides, tokens/styles, and component update policy.
- Support version pinning or approval where an update could affect dependent campaigns.
- Expose reusable brand assets to Design and other workspaces.

## Exclusions

- Do not make every dependent use silently update on a breaking component change.
- Do not build a public marketplace.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A component can be reused in multiple objects and maintains stable identity to its definition.
- **[REQUIRED]** Users can review, accept, pin, or roll back component changes according to policy.
- **[REQUIRED]** Brand primitives can be referenced in a sample Present or Video composition.

## Verification

- Test instance propagation, overrides, version pinning, concurrent edits, and rollback.
- Compare exported artifacts before and after a component version update.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** One Design systems owner defines component semantics; integration examples can be built by other workspace agents.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
