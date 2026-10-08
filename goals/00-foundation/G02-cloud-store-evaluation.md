# G02 — Cloud project-store benchmark and ADR

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G01

## Outcome

**[ADR REQUIRED]** Choose a cloud project-state store using representative workloads rather than architectural preference.

## Scope

- Build a repeatable workload from the graph fixture and expected read/write, transaction, conflict, paging, and recovery patterns.
- Compare PostgreSQL as the initial default against FoundationDB for correctness, schema/query effort, operational needs, recovery, scale path, and cost drivers.
- Record results, limits, recommendation, and a revisit trigger in an ADR.

## Exclusions

- Do not build the production project service on an unevaluated candidate.
- Do not optimize synthetic throughput while ignoring operational burden and data migration.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** The ADR names the selected first production store or explicitly defers the decision with a safe default.
- **[REQUIRED]** Benchmark code and workload assumptions are repeatable and checked in.
- **[REQUIRED]** The choice preserves an application-level storage interface and project semantics.

## Verification

- Run the benchmark from documented setup and retain raw results with hardware/configuration notes.
- Test transactional correctness and failure/restart behavior for each candidate that reaches the comparison stage.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** One storage lead owns the workload and decision. Agents can build isolated adapters or benchmark reports, but only the lead changes the ADR and abstraction contract.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
