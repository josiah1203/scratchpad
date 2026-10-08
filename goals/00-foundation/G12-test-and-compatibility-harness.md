# G12 — Test, fuzz, and compatibility harness

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G00

## Outcome

**[REQUIRED]** Make reliability checks repeatable before the first product editor grows.

Progress evidence (partial; G12 remains planned): [`G12 evidence`](../../docs/evidence/G12-test-compatibility-harness.md).

## Scope

- Create synthetic project fixtures, schema compatibility fixtures, golden render harness, operation replay harness, and fuzz targets for parsers/contracts.
- Add CI tiers for fast checks and deeper scheduled or release checks.
- Define safe handling and licensing for imported test fixtures.

## Exclusions

- Do not make every long-running fuzz/performance job block every local edit.
- Do not add tests that only mirror private implementation details.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A contributor can run relevant test tiers locally and CI reports them clearly.
- **[REQUIRED]** Historical schema fixtures and synthetic media are available to downstream goals.
- **[REQUIRED]** Fuzz and golden tests produce reproducible artifacts and understandable failure reports.

## Verification

- Run fast and release test tiers from documented commands.
- Confirm malformed fixtures fail safely and remain as regression cases.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Test-infrastructure lead owns runners and fixture policy. Product agents may add fixtures under the documented ownership and licensing rules.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
