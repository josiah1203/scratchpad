# G40 — Release, reliability, and large-project readiness

> **Phase:** Enterprise  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G12, G13, G14, G27, G39

## Outcome

**[REQUIRED]** Prove the platform can upgrade, recover, scale, and roll back without jeopardizing professional projects.

## Scope

- Establish service objectives from observed workloads, staged rollout, kill switches, LTS channel, backup/restore drills, migration rehearsal, and incident runbooks.
- Create large synthetic project/media fixtures and replay production-like operation streams.
- Verify cross-region recovery and tenant-specific data policies before making claims.

## Exclusions

- Do not declare scale limits without measurements or expose user project data in synthetic fixtures.
- Do not ship a schema migration without rollback/recovery planning.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Release artifacts are signed/provenanced according to policy, staged, observable, and rollback tested.
- **[REQUIRED]** Large-project limits and performance budgets are published internally and measured in CI/release checks.
- **[REQUIRED]** Operators can restore a project and explain data-loss boundaries under each failure scenario.

## Verification

- Run replay, migration, load, failure injection, backup restore, staged rollout, and rollback exercises.
- Document results, environment, known limits, and release blockers.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Reliability lead coordinates a release exercise; engine and service agents own component budgets and runbooks.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
