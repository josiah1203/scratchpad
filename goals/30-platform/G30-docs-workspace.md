# G30 — Docs workspace and PDF workflows

> **Phase:** Platform expansion  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G05, G07, G15, G17

## Outcome

**[REQUIRED]** Provide structured document editing and practical PDF review/interchange over shared projects.

## Scope

- Implement structured pages, text/layout blocks, comments, annotations, version history, and a limited review/signature-ready workflow boundary.
- Use PDFium for supported PDF parsing/rendering and preserve original input.
- **[ADR REQUIRED]** Confirm PDFium build, update, security, and distribution obligations before shipping it.
- Define PDF export fidelity, accessibility, and document security behavior.

## Exclusions

- Do not treat PDF as the canonical editable document model.
- Do not perform legally binding signing or claim compliance without separate product/security review.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Users can create/edit/review a structured document, import/view supported PDFs, and export a PDF tied to project revision.
- **[REQUIRED]** Unsupported/encrypted/corrupt PDFs are isolated and explained.
- **[REQUIRED]** Permissions and audit history cover document review actions.

## Verification

- Run PDF parser fuzz fixtures, page rendering goldens, text extraction, permissions, and export tests.
- Review PDFium build dependencies and distribution obligations.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Docs lead owns internal document schema; PDF adapter work can be isolated behind import/render/export APIs.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
