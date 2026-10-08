# Portable project bundle v1 — proposed contract

- Status: v1 exporter, verifier, and importer implemented as runtime prototypes; acceptance review remains outstanding.
- Manifest schema: [`project-manifest-v1.schema.json`](../../schemas/project-manifest-v1.schema.json).
- Related: [ADR-001](../adr/ADR-001-project-graph-command-v1.md), [ADR-003](../adr/ADR-003-history-snapshot-recovery.md), G06 and G20.

## Purpose and boundary

This is a workspace-neutral package for moving a project without cloud access. It preserves the current graph, append-only operations, recovery snapshots, named-version references, and all immutable assets referenced by those included records, including typed operation payloads where needed for supported replay. Design-specific interchange and fidelity to external design formats remain G20 work.

The proposed container is a directory tree with a strict manifest and deterministic file layout. No archive dependency is selected. An archive wrapper may be considered later, but it must unpack safely and preserve the same relative layout.

```text
manifest.json
project.json
operations.jsonl
history/snapshots/<snapshot-id>.json
assets/sha256/<first-two-hex>/<remaining-hex>.blob
```

The manifest does not accept arbitrary file paths. Snapshot and asset paths are derived from validated IDs; graph and operation paths are fixed. This reduces path traversal risk and keeps the schema portable across filesystems.

## Manifest semantics

- `format` and `format_version` identify the bundle contract. V1 supports graph schema 1; each snapshot still carries its own snapshot and graph schema versions.
- `project_id` must match the graph, every operation, and every snapshot.
- `graph.sha256` is the SHA-256 digest of the exact `project.json` bytes. `graph.schema_version` repeats the graph version so a reader can reject unsupported data before decoding it.
- `operations.jsonl` contains one command envelope per line, sorted by `(lamport, client_id, client_sequence)`. The manifest records line count, raw-file digest, schema versions present, and the named sort order. Empty history is a zero-byte file with count 0 and an empty version list.
- Each snapshot file is the serialized snapshot record; the manifest records its ID and raw-file digest. Its internal `record_sha256` is also checked. At least one recovery snapshot is required.
- `named_versions` are immutable references to included snapshots. Import validation must reject duplicate version IDs/names, cross-project records, and dangling snapshot IDs.
- `assets` lists the digest and byte length of every immutable asset referenced by the current graph, any included snapshot, or typed operation payloads needed by supported replay. The blob path is derived from the lowercase SHA-256 ID. Import verifies both digest and length before publishing anything.
- `access_control` is explicitly content-only: memberships and capability grants are not included, and importing a bundle never grants access.
- Operation actor identities are retained as opaque historical provenance so the audit trail is not silently rewritten. They are not membership records and must never be used to authorize an import or remote action. This can contain user-identifying data; export UI and retention policy must disclose that before release.

## Current verifier/importer/exporter behavior

`platform_runtime::portable::export_project_bundle` writes the graph, every validated snapshot, named-version links, the ordered operation journal, and the asset closure to the layout above. It stages and hashes the content before publication, rejects an already-existing destination, and installs `manifest.json` last. A directory left without that manifest after an interrupted publication is incomplete and must not be consumed as a bundle. The caller must keep project writes quiescent while exporting because the current APIs do not hold one `SQLite` read snapshot across recovery and paged reads. Recovery may repair the materialized graph as a side effect.

`platform_runtime::portable::verify_project_bundle` independently checks the exact directory layout, file types, hashes, graph/snapshot/operation schemas, cross-record IDs and cursors, asset closure, and whether replay from the earliest included snapshot reproduces later snapshots and `project.json`. It rejects symlinks, extra paths, and unsupported formats. `BundleImportLimits` provides configurable limits; defaults cap entries at 120,000, total bytes at 8 GiB, non-asset bytes at 256 MiB, one file at 2 GiB, operations at 100,000, snapshots at 10,000, and assets at 100,000.

`platform_runtime::portable::import_project_bundle` verifies the entire bundle before writing. It ingests assets into the caller-selected local scope and commits operations, snapshots, named versions, and the materialized graph in one SQLite transaction. Existing project/operation/snapshot/version identity collisions are rejected. Imported operations begin locally pending; membership and capability grants are never imported. If a later database transaction fails, already-ingested immutable asset blobs can remain orphaned, but project rows roll back together. The bundle directory must remain unchanged while it is being verified and imported; hostile concurrent filesystem mutation is not fully serialized by this prototype.

The exporter includes asset IDs found as strings in typed command JSON as well as assets referenced by graph and snapshot records. The runtime currently has no richer typed asset-reference schema for command properties. Actor IDs remain in operation lines, so this is not yet a user-facing export feature without a reviewed privacy disclosure. There is no archive wrapper, signature/authenticated provenance, fuzz campaign, or cross-platform resource-limit review yet.

## Validation and import behavior

The portable-bundle test validates the actual exporter-produced manifest against the Draft 2020-12 JSON Schema and checks that membership inclusion and unknown top-level fields are rejected. The runtime verifier additionally checks cross-record invariants that JSON Schema cannot express: ID uniqueness, project consistency, operation count/order and schema-version declarations, named-version-to-snapshot references, snapshot cursors, replay consistency, graph/asset closure, raw-file checksums, internal snapshot checksums, and asset hashes/lengths. Broader adversarial/fuzz validation remains outstanding.

Treat every bundle as untrusted input. The verifier rejects symlinks, paths escaping the bundle root, unexpected files/directories, and unsupported versions; configurable file-count and byte-size limits are applied before parsing large content. The importer validates the complete package before changing project rows, then commits all database records together. V1 import does not merge or overwrite an existing project with the same ID; collision is a safe error. Importing as a duplicate with rewritten IDs is a separate operation and must not silently alter operation history. The source bundle must remain unchanged during verification/import; hostile concurrent mutation is not yet eliminated with descriptor-relative no-follow filesystem APIs.

The manifest hashes detect accidental or malicious content changes only when the manifest itself is trusted. They do not authenticate the exporter. Signatures, authenticated provenance, archive packaging, and access-control export are outside this proposal.

## Compatibility and open decisions

V1 uses a strict top-level manifest. Future incompatible changes require a new format version or an explicit migration. The current runtime has no graph migration framework, membership persistence, or signed-export support. The operation actor-ID privacy policy, default resource limits, and lack of authenticated provenance need product and security review before accepting this format.
