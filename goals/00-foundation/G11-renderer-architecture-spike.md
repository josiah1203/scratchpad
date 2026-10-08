# G11 — 2D renderer and GPU architecture spike

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G00

## Outcome

**[ADR REQUIRED]** Choose a renderer path based on an interactive canvas prototype and target-device evidence.

Progress evidence (partial; not an acceptance claim): [`docs/evidence/G11-renderer-architecture-spike.md`](../../docs/evidence/G11-renderer-architecture-spike.md); proposed decision: [`ADR-004`](../../docs/adr/ADR-004-renderer-and-gpu-boundary.md).

## Scope

- Prototype core paths using Skia and evaluate wgpu for GPU operations that need it.
- Measure vector/text/image rendering, zoom/pan, large scene behavior, GPU fallback, packaging, and bindings on target platforms.
- Record a renderer ADR with ownership of text shaping, scene graph evaluation, caching, GPU tasks, and fallbacks.

## Exclusions

- Do not build a complete Design editor in the spike.
- Do not ship two competing full-scene renderers without an explicit reason.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** The chosen first renderer, GPU boundary, fallback strategy, and platform support are documented.
- **[REQUIRED]** Benchmarks and test images are reproducible; known limitations are explicit.
- **[REQUIRED]** Renderer API does not own project graph or command semantics.

## Verification

- Run prototype on release-target platforms or document unavailable targets.
- Compare golden images and cold/warm performance using the same fixtures.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** One graphics lead owns the ADR. Separate agents can prototype backends, but use one shared fixture set and do not merge backend-specific product APIs.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
