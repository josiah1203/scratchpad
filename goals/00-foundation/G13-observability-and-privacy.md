# G13 — Privacy-aware observability foundation

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G00, G07

## Outcome

**[REQUIRED]** Trace an edit through the system without collecting creative content unnecessarily.

## Scope

- Instrument UI command, runtime, local journal, sync, asset, and background-job boundaries with vendor-neutral traces, metrics, and structured logs.
- Define correlation IDs, redaction rules, diagnostic export, retention, and opt-in crash-report behavior.
- Add a support bundle that excludes source media and secrets by default.

## Exclusions

- Do not log prompts, documents, media, or project names by default.
- Do not make a telemetry provider a product dependency.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A synthetic edit can be traced across local and remote boundaries with useful latency/failure details.
- **[REQUIRED]** Sensitive fields are redacted and diagnostics explain exactly what is included.
- **[REQUIRED]** Telemetry can be governed by user or organization policy where applicable.

## Verification

- Inspect emitted logs/traces for synthetic secrets and creative text; assert redaction.
- Test trace continuity through retries and ensure duplicate operations remain identifiable.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** One observability/privacy owner defines shared fields and redaction. Service teams instrument their boundaries against the contract.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
