# G01 — Project graph schema and object identity

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G00

## Outcome

**[REQUIRED]** Define the versioned shared project model that every workspace will use.

Progress evidence (partial; G01 remains planned): [`G01 evidence`](../../docs/evidence/G01-project-graph.md).

## Scope

- Specify project, node, relation, asset-reference, membership, metadata, and version-reference concepts.
- Define stable UUID rules, typed node registration, required invariants, unknown-field behavior, and graph paging boundaries.
- Publish a small synthetic campaign graph fixture with Design, media, document, and brand objects.

## Exclusions

- Do not implement all twelve workspace node types.
- Do not couple the graph schema to any one editor UI or storage vendor.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Schema and invariants are documented and machine-readable where practical.
- **[REQUIRED]** Historical/unknown node data can be preserved or rejected with explicit diagnostics.
- **[REQUIRED]** Fixture validates graph traversal, object identity, cross-workspace references, and partial loading.

## Verification

- Schema validation tests cover valid, invalid, unknown, and large fixture cases.
- Review dependency direction: shared graph types do not import workspace UI code.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** A single schema owner merges changes. Workspace agents may propose node schemas, but additions are reviewed together to prevent incompatible identity and relation conventions.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
