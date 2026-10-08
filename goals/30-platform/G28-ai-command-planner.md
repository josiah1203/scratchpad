# G28 — AI command planner and model router

> **Phase:** Platform expansion  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G03, G07, G14

## Outcome

**[REQUIRED]** Let AI propose useful creative edits through the same validated command path as users.

## Scope

- Build context selection, intent classification, structured plan generation, permission checks, deterministic validation, preview/review, apply, and undo.
- Route deterministic, local, hosted, and frontier models by task and policy; record run provenance and budgets.
- Start with bounded Design tasks and one Image/Video-adjacent workflow.

## Exclusions

- Do not grant a model direct SQL/project mutation or silent external publishing capability.
- Do not require one provider or model family.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** AI plans are typed, previewable where appropriate, authorized, attributable, and replayable through normal commands.
- **[REQUIRED]** Rejected/invalid plans make no state change and explain the issue.
- **[REQUIRED]** Provider, model, data transfer, and cost controls are visible to policy.

## Verification

- Test prompt injection, unauthorized actions, malformed plans, cancellation, model timeout, and rollback.
- Compare AI-generated command sequences against deterministic acceptance fixtures.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** AI platform lead owns planner/command interface. Workspace agents provide typed tools and examples; provider adapters can be separate workers.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
