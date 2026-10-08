# G14 — Foundation end-to-end vertical slice

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G01, G03, G04, G05, G06, G07, G08, G09, G10, G11, G12, G13

## Outcome

**[REQUIRED]** Prove one project edit is durable, replayable, permissioned, syncable, and renderable.

## Scope

- Create a Design frame and child object through a command, journal it, render it, terminate/reopen, and recover history.
- Make edits offline on two clients, reconnect, and verify convergence without duplicates or lost operations.
- Export one deterministic artifact and exercise a named version and denied permission.

## Exclusions

- Do not expand editor scope beyond the smallest coherent frame/text/vector scenario.
- Do not proceed on mock durability or cloud-only save behavior.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** All Phase 0 exit criteria in AGENTS.md are demonstrated by automated evidence.
- **[REQUIRED]** Failure at each local commit and sync boundary yields recovery or an explicit safe error.
- **[REQUIRED]** The vertical slice becomes a fixture for future workspaces.

## Verification

- Run crash/replay, offline convergence, authorization, asset integrity, compatibility, and render golden suites.
- Attach exact commands, environment, evidence, and known limits.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** One integration lead owns the end-to-end path. Workers can fix subsystem failures, but the lead owns acceptance evidence and release decision.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
