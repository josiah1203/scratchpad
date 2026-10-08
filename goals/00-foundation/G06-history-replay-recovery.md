# G06 — History, replay, snapshots, and recovery

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G04, G05

## Outcome

**[REQUIRED]** Reconstruct a valid project from snapshots, operations, and immutable assets.

## Scope

- Implement operation replay and snapshot checkpoints with explicit schema/runtime versions.
- Add named versions/branch references and recovery diagnostics without replacing append-only history.
- Define compaction rules that preserve auditability and portable export requirements.

## Exclusions

- Do not delete original asset bytes or operations as an optimization without a proven retention policy.
- Do not promise bit-identical GPU output until renderer determinism is measured.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A materialized project can be rebuilt from a valid snapshot plus later operations.
- **[REQUIRED]** A damaged materialized view is detected and repaired without losing valid history.
- **[REQUIRED]** Named versions resolve to exact graph and asset references.

## Verification

- Replay historical fixtures across supported schema migrations and compare canonical graph state.
- Inject snapshot and materialization corruption and verify recovery or explicit safe failure.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** One history owner controls snapshot format and replay order. Renderer-specific golden output tests belong to renderer goals.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
