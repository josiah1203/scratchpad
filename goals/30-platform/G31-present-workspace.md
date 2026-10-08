# G31 — Present workspace and live deck delivery

> **Phase:** Platform expansion  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G15, G18, G27

## Outcome

**[REQUIRED]** Turn shared Design objects into a focused slide authoring and presenting workflow.

## Scope

- Implement deck, slide, speaker notes, transitions basics, presenter mode, and linked Design/brand components.
- Support audience and presenter displays with local resilience.
- Export slides to a documented interchange/output format.

## Exclusions

- Do not build a general word processor or full office suite.
- Do not silently update approved decks when source components change.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A user can author a branded deck, present it, and export it using shared project history.
- **[REQUIRED]** Linked objects preserve identity and update policy.
- **[REQUIRED]** Presenter mode remains usable with temporary network loss.

## Verification

- Test slide ordering, linked component revisions, presenter/audience state, export goldens, and offline presentation.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Presentation UX lead owns deck semantics; Design systems agents supply component contracts, not copies.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
