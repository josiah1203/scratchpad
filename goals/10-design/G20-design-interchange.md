# G20 — Design import, export, and portable project slice

> **Phase:** Design  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G15, G16, G17, G18

## Outcome

**[REQUIRED]** Provide a useful route into and out of Design without making proprietary files canonical.

## Scope

- Implement first prioritized import/export formats, preserve source originals, and report unsupported features.
- Add a documented portable project bundle for a small project with manifest, graph, operations/version references, and assets.
- Define fidelity guarantees and warnings for lossy conversions.

## Exclusions

- Do not promise full round-trip parity for every legacy format.
- Do not silently flatten editable data when a structured option exists.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Portable bundle can be exported, moved, imported, and validated without cloud account access.
- **[REQUIRED]** Import failures are isolated and do not corrupt existing projects.
- **[REQUIRED]** Format fidelity and limits are documented.

## Verification

- Use licensed or synthetic fixtures for valid, malformed, large, and unsupported input.
- Test export/import graph identity, asset hashes, and golden render comparisons.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Interchange lead owns format priorities and security boundary. Parser work can be delegated by format with a shared fixture policy.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
