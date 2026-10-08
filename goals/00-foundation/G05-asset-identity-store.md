# G05 — Asset identity, ingest, and local object store

> **Phase:** Foundation  \
> **Status:** Planned  \
> **Suggested size:** M  \
> **Depends on:** G03

## Outcome

**[REQUIRED]** Give source and derived media stable identity independent of workspace paths.

## Scope

- Define immutable asset records with content digest, byte length, media kind, provenance, rights metadata, and references.
- Implement local ingest with streaming hash, atomic placement, duplicate detection within an authorized scope, and integrity verification.
- Define remote object-store interface and transfer states without requiring a production cloud provider.

## Exclusions

- Do not globally deduplicate across tenants in a way that reveals whether another tenant has an asset.
- Do not put large bytes in the project graph or operation log.

## Deliverables

- **[REQUIRED]** Implementation or validated decision artifact matching the goal outcome.
- **[REQUIRED]** Updated API/schema/ADR and module documentation affected by the work.
- **[REQUIRED]** Tests, fixtures, migration notes, and operational notes appropriate to the changed contract.
- **[REQUIRED]** A concise evidence report with exact checks run and known limitations.

## Acceptance criteria

- **[REQUIRED]** Ingested assets have stable identity and can be referenced by multiple workspaces.
- **[REQUIRED]** Partial uploads and corrupted bytes are detected and recoverable.
- **[REQUIRED]** Derived assets record source revision and transformation metadata.

## Verification

- Test same-byte ingest, changed-byte ingest, interrupted ingest, corrupted blob, and orphan cleanup.
- Verify hashes before and after copy/restore.

## Agent and swarm boundaries

- **[REQUIRED]** Read root AGENTS.md, goals/README.md, and dependency goals before editing.
- **[REQUIRED]** Keep ownership of shared contracts with one lead; parallel agents work against accepted interfaces and disjoint areas.
- **[REQUIRED]** Do not weaken acceptance criteria, skip a required ADR, or claim completion without evidence.
- **Suggested lead boundary:** Asset identity and storage interface have one owner. Image/video agents may define metadata needs through proposals, not parallel incompatible schemas.
- **[DEFAULT]** If work exceeds one reviewable change, split it into small tasks; keep this goal as the integration and acceptance gate.
