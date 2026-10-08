# G34 — Publish campaigns and channel delivery

> **Phase:** Platform expansion  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G07, G27, G28

## Outcome

**[REQUIRED]** Prepare, approve, schedule, publish, and track creative renditions across channels.

## Scope

- Define campaign, channel, rendition, schedule, metadata, approval, external account, and publication result objects.
- Implement one or two channel integrations behind idempotent provider adapters.
- Tie every published artifact to exact project revision and retain returned platform IDs.

## Exclusions

- Do not let AI or retries publish externally without configured authorization and approval.
- Do not claim universal platform analytics coverage.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A campaign can render, review, approve, publish, verify provider response, and resume safely after failure.
- **[REQUIRED]** External actions are permissioned, logged, idempotent, and reversible where provider APIs allow.
- **[REQUIRED]** Analytics ingestion is explicit about freshness and provider limitations.

## Verification

- Test provider timeout after remote success, duplicate retry, revoked account, scheduling, approval, and revision provenance.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Publishing lead owns external side-effect and idempotency policy. Each channel integration is a separate worker using the same contract.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
