# G00 — Repository baseline and agent workflow

> **Phase:** Foundation  \
> **Status:** In progress  \
> **Suggested size:** M  \
> **Depends on:** None

## Outcome

**[REQUIRED]** Establish a buildable, reviewable repository that coding agents can change safely.

Progress evidence (partial; G00 remains unaccepted): [`G00 evidence`](../../docs/evidence/G00-repository-baseline.md).

## Scope

- Choose and document the initial monorepo layout, toolchains, dependency pinning, code ownership, formatting, linting, and local developer setup.
- Add CI for the chosen languages and a small build/test smoke path.
- Create README, ADR index, contribution guidance, and a minimal sample project fixture.

## Exclusions

- Do not create placeholder implementations for every future workspace.
- Do not add a large CI matrix before the first supported target platforms are selected.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** A new contributor can install prerequisites, build, run tests, and understand repository boundaries from documented steps.
- **[REQUIRED]** CI runs formatting, lint, build, and smoke tests on the default branch.
- **[REQUIRED]** Agent task notes identify acceptance criteria, changed areas, commands run, and limitations.

## Verification

- Run the documented clean setup from a fresh checkout or clean environment.
- Verify CI fails on a deliberate formatting or test failure, then revert the deliberate failure.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** One lead owns toolchain and repository conventions. Other agents may draft docs or fixtures in separate paths, but do not independently choose package managers or formatters.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
