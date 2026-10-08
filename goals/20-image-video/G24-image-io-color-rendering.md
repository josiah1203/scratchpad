# G24 — Image I/O, color, tiles, and rendering

> **Phase:** Image and Video  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G11, G23

## Outcome

**[REQUIRED]** Handle professional image sizes and color behavior without loading every pixel into RAM.

## Scope

- Integrate approved image I/O and color libraries, tile/pyramid cache, preview resolution, color transforms, and export.
- Define RAW ingest scope and license/configuration review before distribution.
- **[ADR REQUIRED]** Record the supported image/RAW formats, color pipeline, codec configuration, and distribution obligations.
- Build deterministic CPU fallback and renderer invalidation around nondestructive edits.

## Exclusions

- Do not make GPU-only output the source of truth.
- Do not ship an unreviewed codec or image-library build.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Large images open and edit with bounded working-set behavior.
- **[REQUIRED]** Color intent and output profiles are explicit and testable.
- **[REQUIRED]** Missing codecs or unsupported RAW features produce diagnostics.

## Verification

- Run format, profile, tile-cache, memory-pressure, CPU/GPU fallback, and golden image tests.
- Generate a dependency/license report for shipped codecs and libraries.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Image pipeline lead owns format and color configuration. Each format adapter can be developed separately against common fixtures.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
