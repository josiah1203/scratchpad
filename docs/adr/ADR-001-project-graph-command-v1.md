# ADR-001: Project graph and command envelope v1

- Status: proposed
- Date: 2026-10-07
- Owners: Runtime/contract lead
- Related: G01, G03, G06, G08

## Context

All workspaces need stable object identity and one attributable state-change path. The first implementation needs a portable graph contract and typed commands so journal, history, authorization, and sync can build against shared shapes. Snapshot/compaction rules and cross-version migration evidence are not complete, so the full persistence contract is not ready for acceptance.

## Decision

Use JSON Schema 2020-12 documents for the initial interchange envelope and Rust serde types for runtime validation. Project and node identities are UUID strings. Assets use SHA-256 content identifiers. Nodes are typed, may name a parent, and may reference cross-workspace objects. Unknown node properties are retained in an extension map; unknown top-level properties fail with diagnostics.

Treat the project as a durable aggregate whose records have distinct authority boundaries: the editable content graph, append-only operation history, immutable assets, snapshots/named-version references, and project membership/access policy. `ProjectGraph` is the content-graph envelope, not by itself the complete portable project. Membership identifies a principal within a tenant/project; it is distinct from a capability grant. A proposed current-record shape is captured in [`project-membership-v1.schema.json`](../../schemas/project-membership-v1.schema.json): stable membership, tenant, and project IDs; an opaque principal kind/ID; and active/revoked status. It deliberately does not choose roles, capability mapping, invitations, or mutation-audit representation. Membership mutations and authoritative authorization belong to a trusted service and must not be accepted from an imported graph. Offline policy snapshots may support UX decisions but do not grant remote authority.

Named versions remain immutable history records that reference snapshots, rather than mutable graph nodes. The v1 portable-project manifest and directory exporter enumerate graph data, operations, snapshots/named versions, and required assets. The proposed default interchange is content/history-only: it omits live membership and capability grants, and importing it must never recreate access. The current exporter retains operation actor IDs as opaque history provenance; its privacy policy and user disclosure require separate review before user-facing release.

Commands have a unique operation ID, project/target IDs, schema version, actor, causal client sequence and Lamport value, optional parent operation, and a tagged typed command. The initial command families are create, set/clear property, move/reorder, delete, and restore. Undo is represented by a compensating operation. Large media bytes remain outside graphs and operation payloads.

The runtime includes an initial graph reducer, authorized local commit boundary, journal replay, checksummed snapshot records, and immutable named-version references. Production snapshot/compaction, graph migration, and historical fixture evidence are incomplete. These schemas and the operation ledger remain implementation-stage contracts until compatibility and retention policy is decided.

## Alternatives considered

- A per-workspace file format would duplicate project identity and make cross-workspace references fragile.
- An untyped JSON patch would make validation, authorization, replay, and migration behavior ambiguous.
- Rejecting every unknown node field would prevent compatible readers from preserving future node data.
- Using wall-clock timestamps as the only operation order would not be deterministic across clients.
- Embedding authoritative membership and capability grants in an offline-editable graph would let client-supplied state masquerade as current authorization and couple content interchange to account identity.
- Treating capability grants as membership would conflate a project relationship with the current, revocable permissions derived from it.
- Exporting account membership by default would leak tenant/project access metadata and would not make vendor-specific principal IDs portable.

## Consequences

Every state-changing client must call the shared typed command interface. Services and UI code need a registry that maps command/node types to permissions. Graph schema changes require compatibility fixtures and migration evidence. The v1 exporter/verifier/importer are implementation prototypes, not an accepted portability guarantee; the membership record schema is proposed, with no authoritative persistence implementation. The ownership split and content-only import default above remain proposals. Snapshot compaction, long-term operation retention, and full recovery orchestration remain unresolved.

## Validation and revisit trigger

The synthetic campaign fixture exercises Design, media, and brand references. Runtime tests cover invalid IDs, duplicates, missing references, parent cycles, unknown node field round-tripping, typed command validation, compensation, retry identity, and portable export/import/replay; test-only Draft 2020-12 validation now covers the graph, membership, and portable-manifest schemas. Before accepting this ADR, implement and migrate authoritative membership persistence, define invitation/role/audit semantics, add adversarial/fuzz coverage for bundle import, and review actor-ID privacy; import must preserve graph, operation, snapshot, named-version, and asset references while excluding access grants by default. Revisit before the first portable project release, before discarding operation history, or when cross-version client/service compatibility is specified.
