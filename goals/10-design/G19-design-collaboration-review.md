# G19 — Design collaboration, comments, and review

> **Phase:** Design  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G07, G08, G15, G18

## Outcome

**[REQUIRED]** Enable multiple people to edit and review the same Design project safely.

## Scope

- Add collaborator presence, selections/cursors, object/frame comments, mentions/assignment basics, and review resolution state.
- Keep ephemeral presence separate from durable comments and commands.
- Define conflict handling for simultaneous component, layer-order, and property changes.

## Exclusions

- Do not treat presence as durable state.
- Do not implement organization-wide chat or unrelated notifications.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Two collaborators can edit and review a project with clear attribution and predictable conflict outcomes.
- **[REQUIRED]** Comments remain attached or safely reanchor when objects move or change.
- **[REQUIRED]** Revoked users cannot continue remote edits.

## Verification

- Run concurrent edit, offline merge, comment reanchor, permission revocation, and duplicate-delivery tests.
- Verify presence loss does not alter project history.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Realtime lead owns transport/presence; Design lead owns anchors and review UX. Integrate through sync contracts.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
