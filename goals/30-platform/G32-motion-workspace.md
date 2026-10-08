# G32 — Motion workspace and live compositions

> **Phase:** Platform expansion  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G16, G26, G27

## Outcome

**[REQUIRED]** Add editable motion graphics and compositions that remain live when placed in Video.

## Scope

- Define composition, layer, keyframe, effect, transform, track, and timebase contracts.
- Implement a bounded motion-graphics workflow and embed a Motion composition as a live object in Video.
- Reuse renderer, assets, history, and render jobs.

## Exclusions

- Do not reproduce the full After Effects feature surface.
- Do not require an intermediary flattened export for supported live compositions.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A Motion composition can be edited independently and evaluated inside a Video sequence.
- **[REQUIRED]** Composition versioning, missing dependencies, and render provenance are clear.
- **[REQUIRED]** Unsupported effects have safe fallback or explicit error behavior.

## Verification

- Test keyframe evaluation, timeline sync, version changes, renderer goldens, and export consistency.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Motion lead owns composition semantics; Video integration works against the accepted composition contract.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
