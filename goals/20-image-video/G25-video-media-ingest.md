# G25 — Video/audio ingest, index, and proxies

> **Phase:** Image and Video  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G05, G11, G12

## Outcome

**[REQUIRED]** Make source media inspectable and editable without requiring full-resolution playback.

## Scope

- Build media metadata index, waveform/thumbnail generation, proxy creation, cache lifecycle, and relinking by asset identity.
- Integrate approved FFmpeg/native codecs with explicit licensing and hardware acceleration configuration.
- **[ADR REQUIRED]** Record the exact FFmpeg build configuration, native codec boundaries, and licensing review before distribution.
- Isolate decoder and parser processes from the project journal.

## Exclusions

- Do not implement the full timeline editor in this goal.
- Do not silently substitute a proxy for original export media.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Representative files ingest, index, proxy, relink, and open with clear unsupported-format errors.
- **[REQUIRED]** Original assets remain immutable and proxy provenance is stored.
- **[REQUIRED]** A decoder crash does not corrupt project state.

## Verification

- Test malformed containers, partial files, variable frame rate, audio-only input, proxy regeneration, and decoder crash recovery.
- Review exact FFmpeg configuration and license obligations before distribution.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Media ingest lead owns codec manifest and fixture safety. Audio and Video workspaces reuse the index/proxy service.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
