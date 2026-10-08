# G27 — Durable render and export pipeline

> **Phase:** Image and Video  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G05, G11, G25, G26

## Outcome

**[REQUIRED]** Render exact project revisions into delivery artifacts with recoverable long-running jobs.

## Scope

- Create render jobs that pin project revision, source assets, codec/profile, transform versions, and output destinations.
- Support cancellation, progress, retry, checkpoint/restart, and deterministic output metadata.
- Keep export explicit; record artifact provenance and link published artifacts to exact project revisions.

## Exclusions

- Do not build the entire cloud rendering farm in this goal.
- Do not retry external side effects without idempotency keys.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A job resumes or reports safe failure after interruption without duplicate artifacts.
- **[REQUIRED]** Export is reproducible within documented format/GPU tolerances.
- **[REQUIRED]** Users can find which project revision produced an artifact.

## Verification

- Kill workers at each stage and verify recovery/idempotency.
- Run render goldens, cancellation, low-disk, and exact-revision provenance tests.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Render orchestration lead owns job schema and retries. Engine adapters can work independently against frozen render contracts.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
