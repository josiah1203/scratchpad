# G39 — Enterprise identity, governance, and data controls

> **Phase:** Enterprise  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G07, G08, G13, G29, G37

## Outcome

**[REQUIRED]** Support enterprise-managed identity, policy, audit, residency, and scoped guest access.

## Scope

- Implement OIDC/SAML and SCIM integration plan, organization roles, project/object relationship scoping, audit export, retention controls, and plugin/AI policy.
- Define customer-managed key and data-residency architecture; ship only after security validation.
- Provide guest/agency access to selected project/workspace/object scopes.

## Exclusions

- Do not claim compliance certifications without independent assessment.
- Do not make tenant policies dependent on client cooperation.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Organization admins can provision/deprovision users and enforce policies server-side.
- **[REQUIRED]** Guests are restricted to explicit scopes and cannot infer unrelated projects or financial records.
- **[REQUIRED]** Audit, retention, key, and residency controls have documented operational behavior.

## Verification

- Run tenant-isolation, revoked-user, guest-scope, key rotation, export, retention, and backup/restore tests.
- Complete threat model and security review before enterprise availability.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Enterprise security lead owns policy model and rollout. Identity, data, and workspace teams integrate only through approved contracts.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
