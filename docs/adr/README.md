# Architecture decision records

ADRs capture decisions that are costly to reverse. A candidate technology or architecture in `AGENTS.md` is not an implementation commitment until the relevant evidence and decision are recorded here.

## Index

## Records

- [ADR-001: Project graph and command envelope v1](ADR-001-project-graph-command-v1.md) — proposed; project aggregate/membership export boundaries, snapshot, compaction, and complete migration evidence remain open.
- [ADR-002: Local SQLite journal configuration](ADR-002-local-sqlite-journal.md) — proposed; fault-injection, abrupt-power-loss, and encrypted-storage decisions remain open.
- [ADR-003: Snapshot, named-version, and recovery contract](ADR-003-history-snapshot-recovery.md) — proposed; production recovery evidence and portable-export policy remain open.
- [ADR-004: Initial 2D renderer and GPU boundary](ADR-004-renderer-and-gpu-boundary.md) — proposed; current evidence covers one Intel macOS host and does not validate GPU-scene rendering or cross-platform packaging.

The next expected ADRs cover:

- project graph schema, operation envelope, snapshots, and compatibility;
- local SQLite durability, encryption, and recovery;
- snapshot integrity, named versions, and safe compaction;
- portable project manifest, asset closure, and identity/access-control export policy;
- cloud project-state storage benchmark;
- collaboration semantics by object family;
- cloud project-state storage benchmark;
- desktop shell and process isolation;
- dependency licensing and import/export fidelity.

## Template

```markdown
# ADR-NNN: Short decision title

- Status: proposed | accepted | superseded
- Date: YYYY-MM-DD
- Owners: team or role
- Related: links to requirements, PRs, or prior ADRs

## Context
## Decision
## Alternatives considered
## Consequences
## Validation and revisit trigger
```
