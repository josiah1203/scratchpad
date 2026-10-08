# G37 — Creator and agency business workflows

> **Phase:** Platform expansion  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G07, G30, G34

## Outcome

**[REQUIRED]** Connect creative deliverables to clients, deals, rights, invoices, and payment status.

## Scope

- Define client, deal, contract/right, deliverable, invoice, and payment-status entities with scoped access.
- Link rights and usage windows to assets and published campaigns.
- Implement a bounded client approval and invoice workflow; integrate payment providers only behind explicit authorization.

## Exclusions

- Do not hold funds or imply regulated payment capabilities without separate legal/product work.
- Do not expose financial records to creative guests by default.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A team can link a client deal and rights record to deliverables, obtain approval, and track invoice status.
- **[REQUIRED]** Commercial records have stricter permissions and append-only audit events.
- **[REQUIRED]** Rights warnings can inform publish eligibility.

## Verification

- Test guest isolation, rights expiry, invoice retries, audit history, and project deletion/retention interactions.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Business lead owns commercial data model and access rules. Integrations are separate tasks behind idempotent APIs.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
