# G03 — Typed creative command model

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G01

## Outcome

**[REQUIRED]** Provide one validated command path for human, AI, plugin, and service edits.

## Scope

- Define command envelopes with operation ID, project and target IDs, actor, schema version, causal/order metadata, and typed arguments.
- Define validation, authorization hook, idempotency, error shape, version compatibility, and command result semantics.
- Implement a small representative set: create node, set property, move/reorder node, and delete/restore as compensatable operations.

## Exclusions

- Do not allow direct UI-to-database state mutation.
- Do not encode media bytes in command payloads.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Commands are deterministic, attributable, validated, retry-safe where applicable, and suitable for replay.
- **[REQUIRED]** Undo is modeled as a compensating operation or documented inverse behavior; historical operations remain auditable.
- **[REQUIRED]** Human and test clients use the same API surface.

## Verification

- Test invalid targets, duplicate IDs, retries, unauthorized actors, ordering, and replay results.
- Run property tests for idempotent commands and graph invariants.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** The command contract has one owner. Agents may implement command families only after the envelope and error contract are accepted.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
