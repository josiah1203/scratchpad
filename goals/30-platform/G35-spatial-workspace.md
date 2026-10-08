# G35 — Spatial workspace and 3D interchange

> **Phase:** Platform expansion  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G05, G11, G15

## Outcome

**[REQUIRED]** Provide a practical 3D scene workflow using established authoring and delivery formats.

## Scope

- Build basic scene/object/material/light/camera objects and asset references.
- Prototype OpenUSD for composition/interchange and glTF for efficient delivery where appropriate.
- **[ADR REQUIRED]** Select supported USD/glTF schemas and interchange behavior based on target workflows and interoperability tests.
- Define deferred asset loading, versioning, renderer boundaries, and mobile camera/spatial ingest.

## Exclusions

- Do not invent a proprietary 3D interchange format or build a complete DCC tool.
- Do not assume every USD feature is supported by every renderer.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A sample product scene can be assembled, edited, exported, reimported within documented fidelity, and delivered as glTF.
- **[REQUIRED]** Large scenes load incrementally and retain source asset identity.
- **[REQUIRED]** Unsupported schema/renderer features are diagnosed.

## Verification

- Use synthetic scene fixtures, format validation, interoperability samples, and render goldens.
- Review OpenUSD and dependency licenses for the chosen build.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Spatial lead owns format semantics; asset and render agents contribute through existing service contracts.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
