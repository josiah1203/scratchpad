# G17 — Professional text and layout foundation

> **Phase:** Design  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G15, G16

## Outcome

**[REQUIRED]** Support robust multilingual editable text and predictable layout in Design.

## Scope

- Integrate HarfBuzz/FreeType/ICU or approved platform services behind a text-layout abstraction.
- Implement editable text runs, paragraph basics, font fallback, line wrapping, variable-font metadata where supported, and text measurement.
- Create fixtures for complex scripts, bidirectional text, emoji, variable fonts, and missing fonts.

## Exclusions

- Do not invent script shaping or claim identical metrics across operating systems without evidence.
- Do not bundle fonts without rights review.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Text remains editable and stable through save/reopen/export.
- **[REQUIRED]** Major supported writing systems have shaping/layout coverage and missing-font behavior is understandable.
- **[REQUIRED]** Layout APIs are reusable by Present and Docs.

## Verification

- Run text golden tests across complex scripts, fallback fonts, zoom levels, and export.
- Review font licensing and record bundled/system font policy.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Typography lead owns shaping/layout abstraction. Workspace agents contribute fixtures and UI through that interface.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
