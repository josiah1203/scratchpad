# G09 — Workspace module contract and SDK

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G01, G03, G07

## Outcome

**[REQUIRED]** Allow first-party workspaces to register editors over shared project objects without shell coupling.

## Scope

- Define module registration for node types, commands, editors, inspectors, renderers, importers/exporters, and required capabilities.
- Provide a minimal sample workspace and contract tests.
- Document lifecycle, versioning, feature discovery, and failure isolation expectations.

## Exclusions

- Do not build a third-party marketplace or promise a stable public SDK yet.
- Do not let modules bypass command and permission layers.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A sample module registers and edits a project object through the shared runtime.
- **[REQUIRED]** Unknown or incompatible module versions fail with actionable diagnostics.
- **[REQUIRED]** The shell can list/open a module without depending on internal implementation.

## Verification

- Contract tests validate registration, command dispatch, missing capabilities, and module load failure.
- Build a sample module in isolation from the desktop shell.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** One platform lead owns the module API. Workspace agents can build test modules, but must not define private shell APIs.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
