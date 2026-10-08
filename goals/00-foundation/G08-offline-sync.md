# G08 — Offline queue and deterministic sync

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G02, G04, G05, G07

## Outcome

**[REQUIRED]** Let clients continue editing offline and converge safely after reconnect.

## Scope

- Define sync cursor, upload/download protocol, idempotency, retries, backpressure, conflict reporting, and auth-refresh behavior.
- Implement operation queueing, resumable asset upload, remote acknowledgment, and deterministic merge for first object types.
- Keep presence separate from durable project state.

**[ADR REQUIRED]** Record merge semantics for each first-supported object family before shipping sync behavior.

## Exclusions

- Do not apply one CRDT to every project object.
- Do not silently discard conflicts or present remote durability as local durability.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Two clients converge after offline edits, duplicate delivery, delay, and reconnect.
- **[REQUIRED]** Permission revocation and expired authentication stop remote mutation while preserving recoverable local work.
- **[REQUIRED]** Sync resumes after process restart and partial upload.

## Verification

- Simulate loss, duplication, reordering, delay, server restart, token expiry, and conflicting property edits.
- Verify operation and asset idempotency across repeated sync attempts.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Sync protocol owner defines cursor and retry contracts. Client and service agents can implement adapters once contracts are frozen.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
