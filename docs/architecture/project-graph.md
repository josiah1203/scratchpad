# Project graph v1

The project graph is the shared, portable model for every workspace. It is not a Design-, Image-, or storage-vendor-owned file. The machine-readable contract lives at [`schemas/project-v1.schema.json`](../../schemas/project-v1.schema.json); runtime validation is implemented in `crates/platform-runtime/src/graph.rs`.

## Contract

- `project_id` and every node `id` are stable UUIDs.
- `nodes` contain typed objects and explicit parent/reference edges.
- Nodes can carry workspace properties, an explicit sibling-order key, and a soft-delete marker; deleted ancestors make their descendants inactive without erasing them.
- `asset_refs` identify immutable bytes by content-addressed ID; large bytes do not belong in the graph.
- The root envelope is strict so malformed or unsupported top-level data fails visibly.
- Node extensions are retained and round-trip so a compatible reader can preserve newer fields it does not understand.
- `schema_version` is explicit. Additive changes or migrations must be tested against historical fixtures before becoming accepted contracts.

## Incremental loading

Clients should load the active workspace and the dependencies needed to render or edit it. `ProjectGraph::load_subgraph` takes root IDs and returns those roots plus their parents and explicit references in source order. It builds one temporary ID index and visits each reachable node at most once, avoiding a full linear search for every dependency. A regression test adds 2,048 unrelated nodes and verifies that loading the Design document still returns only its four-node dependency closure. The current API accepts an already materialized `ProjectGraph`; storage-backed paging and streaming parse are not implemented.

## Membership and version references

The current graph v1 does not serialize membership or permissions. A separate proposed [membership record schema](../../schemas/project-membership-v1.schema.json) describes the tenant/project/principal relationship and active/revoked status; it intentionally contains no role or capability grant. It is not persisted by an authoritative service, and invitation lifecycle, role mapping, mutation audit, and storage uniqueness remain open. Membership must not be inferred from client-supplied graph data. Named versions currently live in the local history tables and reference immutable snapshots, not graph nodes. The proposed [ADR-001 aggregate boundary](../adr/ADR-001-project-graph-command-v1.md) treats these as related project records and proposes content/history-only export by default, with no access grants recreated on import. A v1 portable manifest, exporter, verifier, and importer now exist as prototypes; membership persistence, security review, and full acceptance evidence remain open, and the ADR is not accepted.

## Current fixture

`tests/fixtures/synthetic-campaign/project.json` is a synthetic graph containing a Design document/frame, a media asset, and a brand style. The document references the media and brand nodes, so the fixture exercises a cross-workspace dependency traversal without using real customer content.

## Deliberate limits

This is the first graph contract, not the final operation envelope, snapshot format, permission model, or storage schema. Those require their own evidence and ADRs. The runtime validator enforces identity, non-blank project names, node types, property keys, asset kinds and provenance, reference and asset integrity, and parent-cycle invariants; command application additionally enforces active-node and mutation preconditions. The test tier runs a Draft 2020-12 JSON Schema validator against the graph and proposed membership schemas, separately from runtime validation.

Partial G01 implementation evidence and the remaining compatibility gaps are recorded in [`../evidence/G01-project-graph.md`](../evidence/G01-project-graph.md).
