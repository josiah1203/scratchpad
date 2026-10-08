# G10 — Unified desktop shell and workspace tabs

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G00, G09

## Outcome

**[REQUIRED]** Create the first unified desktop window for project navigation and workspace modules.

## Scope

- Build app frame with project switcher, workspace navigation, tabs, contextual panels, search placeholder, and account/settings entry points.
- Isolate shell UI from creative rendering and expose only typed module APIs.
- Define crash boundaries and restart behavior for workspace/render subprocesses.

## Exclusions

- Do not implement every workspace panel or mature collaboration chrome.
- Do not render large creative scenes as thousands of DOM elements.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A user can open a project and switch between two sample workspaces without losing project context.
- **[REQUIRED]** Tab state survives normal restart where appropriate; a workspace failure does not corrupt the journal.
- **[REQUIRED]** Shell keyboard navigation, focus, and accessibility basics are documented.

## Verification

- Run startup, navigation, module failure/restart, and project context tests.
- Profile representative shell load to detect accidental scene rendering in the DOM.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Shell lead owns global navigation and shared chrome. Workspace agents own editor surfaces inside the agreed module boundary.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
