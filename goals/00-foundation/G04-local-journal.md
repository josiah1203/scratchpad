# G04 — Local durable operation journal

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G03

## Outcome

**[REQUIRED]** Persist accepted project edits locally before the interface acknowledges them.

## Scope

- Implement SQLite-backed local metadata and operation journaling with deliberate WAL and sync settings.
- Add transaction boundaries, journal indexing, disk-full handling, migration hooks, and clear local-versus-cloud durability state.
- Expose append, read, replay-range, and health APIs through the shared runtime.

## Exclusions

- Do not treat cloud sync as the local durability boundary.
- Do not claim protection against all hardware/filesystem failures without fault evidence.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** UI-facing acknowledgment follows the defined successful local commit boundary.
- **[REQUIRED]** Failed commits are surfaced and optimistic UI state can be rolled back or marked uncommitted.
- **[REQUIRED]** Restart recovers committed operations and does not apply an operation twice.

## Verification

- Inject process termination around append/commit/ack boundaries and verify recovery.
- Test disk full, corrupt journal, WAL recovery, and migration from the prior schema.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** One persistence owner controls journal schema and transaction semantics; UI agents can integrate against a mock until the API is stable.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
