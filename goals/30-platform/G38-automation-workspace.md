# G38 — Cross-workspace automation and agent runs

> **Phase:** Platform expansion  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G27, G28, G29, G34, G37

## Outcome

**[REQUIRED]** Compose governed multi-step production workflows that can pause for approval and resume safely.

## Scope

- Define workflow goals, steps, run state, budgets, step limits, artifacts, approvals, and audit records.
- Implement a sample pipeline: ingest recording, clean audio, transcribe, create cuts/renditions, request review, then publish after approval.
- Use durable jobs for long steps and the same command/permission system for edits.

## Exclusions

- Do not allow unbounded agent loops or automatic external publishing without approval.
- Do not place AI agent state outside observable, resumable run records.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A run can pause/resume after process failure, enforce step/cost limits, request approval, and expose every operation/artifact.
- **[REQUIRED]** Cancellation, retry, permission revalidation, and partial completion are explicit.
- **[REQUIRED]** The example workflow is reproducible from a project fixture.

## Verification

- Kill workers during each step; test approval denial, budget exhaustion, duplicate activity, credential revocation, and safe resume.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Automation lead owns workflow/run schema and safety limits; workspace teams expose bounded commands as tools.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
