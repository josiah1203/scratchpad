# G36 — Live production and post-stream clipping

> **Phase:** Platform expansion  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G25, G27, G29

## Outcome

**[REQUIRED]** Support a reliable live scene/source workflow with recordings and immediate clip creation.

## Scope

- Implement scene/source composition, capture configuration, stream state, recording, and clip-from-live-buffer objects.
- Reuse media ingest, codecs, project assets, and render/export services.
- Keep network credentials and external destinations capability-scoped.

## Exclusions

- Do not build every streaming protocol or hardware capture device at once.
- Do not let an encoder failure affect project durability.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A bounded live session can capture, stream to one destination, record locally, and create a linked clip.
- **[REQUIRED]** Encoder/network failure is visible and does not corrupt recording or project state.
- **[REQUIRED]** Required network and device permissions are explicit.

## Verification

- Test dropped network, encoder restart, local disk pressure, source disconnect, credential revocation, and clip provenance.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Live lead owns streaming state; media pipeline and plugin workers supply adapters behind stable contracts.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
