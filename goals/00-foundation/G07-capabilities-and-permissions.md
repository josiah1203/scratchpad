# G07 — Capability and authorization foundation

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G03

## Outcome

**[REQUIRED]** Enforce user, service, AI, and plugin permissions at the command boundary.

## Scope

- Define project roles and capability checks for read, design write, media generation, render creation, and external actions.
- Provide a local policy interface and authoritative service-side check contract.
- Add audit metadata for denied and sensitive actions without logging creative content.

## Exclusions

- Do not rely on hidden/disabled UI controls for authorization.
- Do not grant agents or plugins broad ambient credentials.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Every command path can be checked against actor, project, target, capability, and policy context.
- **[REQUIRED]** External-impact actions have separate capabilities and can require approval.
- **[REQUIRED]** Tests prove cross-project and cross-tenant denial.

## Verification

- Run authorization matrix tests for human, service, AI, plugin, guest, and revoked actor states.
- Test policy changes while a client is offline and when it reconnects.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** A security/authorization owner controls capability names and policy semantics. Workspace teams contribute required capabilities through review.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
