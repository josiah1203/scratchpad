# G29 — Plugin SDK and capability sandbox

> **Phase:** Platform expansion  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G07, G09, G12

## Outcome

**[REQUIRED]** Allow constrained extensions without ambient access to user projects or devices.

## Scope

- Define versioned manifests, module lifecycle, capability requests, command APIs, resource limits, signing/verification, and audit events.
- Prototype WASM execution with Wasmtime or another approved runtime and narrowly granted filesystem/network access.
- **[ADR REQUIRED]** Select the runtime and component/capability model after sandbox threat-model and platform evaluation.
- Provide one sample plugin and organization allow/deny policy hooks.

## Exclusions

- Do not expose unrestricted filesystem, credentials, all-project reads, or arbitrary network by default.
- Do not launch a marketplace before sandbox and upgrade policies are proven.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A sample plugin can read selected data and perform an allowed command only after capability grant.
- **[REQUIRED]** Denied or revoked capabilities are enforced at runtime and service boundaries.
- **[REQUIRED]** Plugin crash or resource exhaustion cannot terminate project journal/shell.

## Verification

- Run sandbox escape-oriented tests, capability denial, resource exhaustion, malicious manifest, signature, and update tests.
- Document threat model and residual risks.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Security lead owns sandbox threat model and capability API; SDK examples can be delegated after the interface is frozen.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
