# Unified Creative Platform — Agent Build Guide

**Status:** Product and architecture direction for implementation planning  
**Audience:** Coding agents, engineers, reviewers, and technical leads  
**Purpose:** Give every contributor the same product context, architectural boundaries, build order, and quality bar.

This file is the repository-wide agent guide. Read it before making implementation changes, then read the nearest `AGENTS.md`, README, architecture decision records (ADRs), and module documentation that apply to the files being changed. More specific repository instructions may add detail; they must not silently weaken the invariants in this guide. If the repository’s actual state differs from an assumption here, inspect it, preserve existing user work, and record the needed decision rather than pretending the assumption is already implemented.

## Requirement labels

Use these labels consistently throughout this guide:

- **[REQUIRED]** A firm product, architectural, security, reliability, or process requirement. Do not weaken or bypass it without an explicit superseding decision.
- **[DEFAULT]** The recommended starting point when no stronger repository-specific decision exists. A default may be changed through normal technical judgment, evidence, or an ADR when the change is consequential.
- **[ADR REQUIRED]** A candidate decision or unresolved choice that must be evaluated and recorded in an ADR before becoming an implementation commitment.
- **[ASPIRATION]** A long-term goal or direction. It is not a current delivery commitment unless a phase plan, product decision, or ADR promotes it.

When a statement has more than one status, use the strongest applicable label. Repository-specific instructions may add requirements but must not silently weaken **[REQUIRED]** items.

## 1. Mission

Build one professional creative platform with specialized workspaces over a shared, durable project system. Users should experience one application across Design, Image, Video, Motion, Audio, Spatial, Docs, Present, Publish, Live, Business, and Automate. The platform should work across desktop, tablet, phone, and cloud services while keeping project identity, assets, history, permissions, and collaboration coherent.

**[REQUIRED]** This is not a project to reproduce Adobe Creative Cloud’s application boundaries or to promise feature parity with twelve mature products. The product thesis is that a shared project graph, creative runtime, command system, and resilient local/cloud state make each workspace more useful and make cross-workspace work feel native.

**[REQUIRED]** The user-facing shell may look like one application with tabs and contextual panels. Internally, the platform is a set of isolated clients, engines, and services connected through versioned contracts.

## 2. Product promise

The system should make these statements true:

1. **[REQUIRED] Acknowledged edits are recoverable.** An edit is durably recorded on the device before the interface confirms it. Cloud availability is not required to continue creating.
2. **[REQUIRED] A project is shared across workspaces.** No workspace owns the project’s canonical logo, photograph, clip, document, or other asset.
3. **[REQUIRED] History is continuous and understandable.** Edits, branches, named versions, AI actions, approvals, and publications can be traced to exact project state.
4. **[REQUIRED] Creative work remains editable.** Adjustments, effects, generated changes, and cross-workspace compositions should remain represented as structured operations or graph nodes whenever technically practical.
5. **[REQUIRED] AI and extensions use governed capabilities.** They invoke the same validated creative commands available to the product, subject to explicit permissions.
6. **[REQUIRED] The project can outlive a device or vendor.** Define a documented portable project format and migration policy; do not make cloud account access the only way to preserve a project’s contents.
7. **[REQUIRED] Mobile is a real creative client.** Share project semantics and runtime operations with desktop while using touch-, camera-, Pencil-, and platform-native interactions.

## 3. Product language and design rules

Use the following product concepts consistently:

| Prefer | Meaning |
|---|---|
| Project | A graph of creative and business objects, operations, assets, membership, and metadata. |
| Workspace | A focused editor or product surface over project objects, such as Design or Video. |
| Object / node | A stable, typed entity in the project graph. |
| Operation / command | A validated, attributable change to project state. |
| Asset | Immutable source or rendered bytes identified by content and metadata. |
| Version | A human-meaningful reference to a project revision or branch point. |
| Publish / export | An explicit creation of an interchange or delivery artifact. |

Product UX rules:

- **[REQUIRED]** Do not expose “autosave” as a product concept. Every acknowledged edit is already durable locally; cloud replication is a separate state. Show sync state only when it helps the user understand availability or collaboration.
- **[REQUIRED]** Do not make users manage `final_FINAL_v7` copies. Use history, named versions, branches, forks, and duplicates. Duplicating remains available when it is useful.
- **[REQUIRED]** Do not create workspace-specific asset ownership or filesystem paths as canonical identity. Paths may be import/export locations, never the only link between project objects.
- **[REQUIRED]** Do not flatten a structured object merely to move it between workspaces. A live Motion composition inserted in Video remains a composition; a Design component embedded in Present retains identity and update policy.
- **[REQUIRED]** Destructive-looking actions must be undoable or explicitly confirmed when recovery is impossible. Preserve originals for nondestructive media workflows.
- **[REQUIRED]** Design desktop, tablet, and phone as different interaction surfaces over common semantics. Do not ship mobile as a remote control or reduced desktop viewport.
- **[REQUIRED]** Keep the shell focused. Workspaces may differ substantially in interaction model while sharing navigation, tabs, project context, comments, permissions, search, notifications, and history.

## 4. Product family and sequence

The intended first-party family has twelve workspaces. The list describes long-term scope, not a requirement to implement them all in the first release.

| Workspace | Primary objects | Product responsibility |
|---|---|---|
| Design | Documents, canvases, frames/pages, vectors, text, components | UI, illustration, layout, typography, brand, print, packaging, and social graphics |
| Image | Images, layers, masks, adjustments, color transforms | Photo ingest, RAW workflow, retouching, compositing, and nondestructive image editing |
| Video | Sequences, clips, tracks, transitions, semantic segments | Nonlinear editing, captions, proxy workflow, color, and delivery renditions |
| Motion | Compositions, keyframes, effects, tracking data | Animation, VFX, compositing, kinetic type, and procedural motion |
| Audio | Sessions, tracks, waveforms, transcripts, speakers | Recording, cleanup, sound design, multitrack editing, and podcast production |
| Spatial | Scenes, objects, materials, lights, cameras | 3D composition, product visualization, AR, and basic animation |
| Docs | Structured documents, pages, signatures, rights records | Document editing, review, PDF interchange, and signing workflows |
| Present | Decks, slides, stories, presenter state | Presentation authoring and delivery, built on shared Design objects |
| Publish | Campaigns, channels, renditions, schedules, metrics | Distribution, scheduling, metadata, and performance analytics |
| Live | Scenes, sources, streams, recordings, clips | Live production, multicamera composition, streaming, and post-stream clipping |
| Business | Clients, deals, rights, deliverables, invoices | Creator and agency commercial operations tied to creative work |
| Automate | Goals, workflows, agent runs, approvals | Governed cross-workspace automation and repeatable production pipelines |

### Recommended delivery order

1. **[DEFAULT] Foundation:** project graph, operation/command model, local journal, assets, recovery, permissions foundation, sync contract, shell, rendering boundary, test/release infrastructure.
2. **[DEFAULT] Design:** use it to prove the shared project model, collaboration, components, history, export, and cross-device behavior.
3. **[DEFAULT] Image and Video:** build the next editors over shared assets, operations, rendering/media services, and project history.
4. **[DEFAULT] Public beta:** Design, Image, Video, collaboration, AI-assisted native commands, desktop and useful native mobile workflows, plus dependable offline creation.
5. **[DEFAULT] Paid professional V1:** deepen typography, color, masks, RAW, codecs, proxy/render workflow, brand systems, team administration, and plugin support.
6. **[ASPIRATION] Platform expansion:** Motion, Audio, Present, Docs, Publish; then Spatial, Live, Business, Automate, marketplace, and developer platform.
7. **[ASPIRATION] Enterprise maturity:** scale, security, residency, key management, audit, retention, DLP, migration, guest/agency access, and long-term support.

The original planning ranges were approximately 0–2 months for foundation, 2–4 for Design, 4–6 for Image and Video, 6–9 for beta, 9–12 for paid V1, 12–18 for expansion, 18–24 for enterprise, and 24–36 for the broader platform. **[REQUIRED]** Treat these as planning hypotheses that depend on team size, scope, and evidence—not as delivery promises. Advance on exit criteria, not calendar dates.

## 5. Architectural shape

Keep these boundaries explicit:

```text
Desktop / Web / iOS / Android clients
                 │
       shell and workspace modules
                 │
     typed creative command API
                 │
       shared creative runtime
    ┌────────────┼────────────┐
 project graph  assets   render/media
    │            │            │
 local journal ─ sync ─ cloud services
                 │
        workflows / analytics
```

- **[REQUIRED]** The desktop shell owns navigation, tabs, panels, menus, account UI, search, comments, notifications, and collaboration chrome.
- **[REQUIRED]** The desktop shell must not render a professional canvas as a large collection of DOM nodes.
- **[DEFAULT]** React/TypeScript is appropriate for shell UI; native or GPU-backed engines own high-volume creative rendering and media work.

### 5.1 Shared creative runtime

- **[REQUIRED]** The runtime must provide shared semantics for graph operations, history, assets, sync, rendering interfaces, typography/color services, permissions, commands, and plugin/automation capabilities.
- **[DEFAULT]** The runtime should be reusable from desktop, server-side jobs, and mobile through stable native bindings and versioned contracts.
- **[ADR REQUIRED]** Rust is the leading implementation candidate for shared performance-sensitive runtime code. Evaluate it against team capability, interoperability, build complexity, debugging, and platform requirements before committing.
- **[REQUIRED]** Do not create one giant runtime crate with unrestricted dependencies.
- **[REQUIRED]** Define module boundaries, public APIs, ownership, and allowed dependency direction.
- **[REQUIRED]** Workspaces register typed node definitions, commands, editors, inspector panels, renderers, importers, and exporters through a module contract.

### 5.2 Project graph and operation model

- **[REQUIRED]** The project graph, not a proprietary per-application file, is the canonical project model.
- **[REQUIRED]** It includes typed nodes and relations, asset references, permissions, metadata, version references, and an operation history.
- **[REQUIRED]** Every persistent object has a stable UUID.
- **[REQUIRED]** Graph loading must be incremental: load the active workspace, visible objects, and required dependencies rather than the whole project into memory.
- **[REQUIRED]** Every accepted state change goes through a typed command and becomes an attributable operation.
- **[REQUIRED]** At minimum, an operation records a unique operation ID, project and target IDs, command/schema version, actor or service identity, causal context or sequence, validated arguments, and logical ordering data.
- **[REQUIRED]** Wall-clock time must not be the sole ordering rule.
- **[REQUIRED]** Large media bytes belong in the asset system, not in operation payloads.
- **[REQUIRED]** Operations must be idempotent where retries are possible.
- **[REQUIRED]** Every command family must define validation, authorization, conflict behavior, undo/compensation, replay, and migration.
- **[REQUIRED]** Do not mutate canonical state through direct UI writes, AI-specific database paths, plugin-specific state paths, or unvalidated SQL.
- **[REQUIRED]** Use snapshots/materialized views to make reads efficient while preserving enough operation and asset history to reconstruct and audit state.
- **[REQUIRED]** Schema evolution must be additive or explicitly migrated, tested on historical fixtures, and must preserve unknown fields or original data where practical.
- **[REQUIRED]** The portable project format must document its manifest, graph, operations, assets, metadata, compatibility guarantees, and export/import behavior.
- **[ADR REQUIRED]** Define the exact graph schema, operation envelope, snapshot/compaction rules, and backward-compatibility policy in an ADR before treating them as stable contracts.

### 5.3 Durability, local state, and recovery

- **[REQUIRED]** The local operation journal is the first durability boundary.
- **[DEFAULT]** SQLite in WAL mode is the initial candidate for local metadata and journal storage.
- **[REQUIRED]** Configure transaction and sync behavior deliberately.
- **[REQUIRED]** The product may acknowledge an edit only after the defined local commit boundary succeeds.
- **[REQUIRED]** If an optimistic UI previews an edit before commit, it must clearly roll it back or report failure if durability fails.
- **[REQUIRED]** Offline creation is a normal operating mode.
- **[REQUIRED]** Cloud outage, auth refresh, or sync conflict must not erase or block local edits.
- **[REQUIRED]** Reconnection must apply deterministic merge/conflict policy by object type.
- **[REQUIRED]** Never imply remote durability merely because local durability succeeded; keep those states distinct.
- **[REQUIRED]** Recovery uses independent layers: current materialized graph, operation history, periodic snapshots, immutable assets, asset integrity metadata, and cross-region copies where configured.
- **[REQUIRED]** A corrupt materialized view must be rebuildable from a valid snapshot and later operations.
- **[REQUIRED]** A workspace process crash must not terminate the journal/sync process or discard unrelated workspace state.
- **[ADR REQUIRED]** Decide local SQLite durability configuration, encrypted local storage, and recovery behavior across abrupt power loss through an ADR supported by fault-injection evidence.

The sequence is:

```text
User intent → validated command → local durable journal → UI acknowledgment
                                               └────→ queued cloud replication
```

### 5.4 Assets and media

- **[REQUIRED]** Treat source assets as immutable blobs.
- **[REQUIRED]** Identify content with a cryptographic digest such as SHA-256.
- **[REQUIRED]** Keep identity, type, provenance, rights, variants, and references in project metadata.
- **[REQUIRED]** Do not perform cross-tenant deduplication in a way that leaks asset existence or weakens tenant isolation.
- **[DEFAULT]** Store large media in durable object storage behind an internal provider interface.
- **[ADR REQUIRED]** Allow enterprise or self-host adapters only after confirming that they meet the same correctness, security, and recovery contract.
- **[REQUIRED]** Use tiling, proxies, thumbnails, pyramids, and lazy fetches for large imagery and video.
- **[REQUIRED]** Verify hashes on ingest and during integrity checks.
- **[REQUIRED]** Derived artifacts must name their source revision and transform parameters.
- **[REQUIRED]** A missing or unavailable remote asset must be a recoverable state with clear diagnostics, not a mysterious broken link.

### 5.5 Collaboration semantics

- **[REQUIRED]** Choose synchronization semantics per object, not by applying one CRDT everywhere.

| Data | Starting model |
|---|---|
| Rich text and simultaneous text editing | **[DEFAULT]** CRDT or equivalent well-tested merge model |
| Ordered collaborative collections | **[DEFAULT]** Sequence CRDT or explicit deterministic order operations |
| Comments and presence | **[DEFAULT]** Append/event model plus ephemeral pub/sub |
| Property edits | **[DEFAULT]** Validated deterministic operations with conflict policy |
| Video timelines | **[DEFAULT]** Ordered operation log with explicit time/ripple semantics |
| Image revisions | **[DEFAULT]** Immutable revision graph and parameterized transforms |
| Business transactions and permissions | **[REQUIRED]** Strongly consistent transactional state |

- **[REQUIRED]** Presence and cursors are ephemeral; they must not be required to reconstruct project state.
- **[REQUIRED]** Permission changes and sensitive business actions require authoritative checks at the service boundary.
- **[ADR REQUIRED]** Define collaboration and merge policy for each object family, including concurrent timeline edits and shared component updates.

### 5.6 Client and process isolation

- **[ADR REQUIRED]** Electron is the current desktop direction, but startup/runtime behavior, security, update behavior, and memory spikes must be documented and evaluated before it becomes a durable commitment.
- **[REQUIRED]** Keep rendering engines independent of Electron and the web DOM.
- **[REQUIRED]** Use process/service boundaries for decoders, GPU work, plugin hosting, sync, and risky workspace work where crashes or untrusted input could take down the shell.
- **[REQUIRED]** A Video decoder crash must not destroy an open Design session or its journal.
- **[REQUIRED]** Mobile uses native iOS/iPadOS and Android interfaces over shared runtime semantics.
- **[DEFAULT]** Swift/SwiftUI/UIKit and Kotlin/Compose are the leading UI choices.
- **[ADR REQUIRED]** Tauri 2 may be evaluated for appropriate surfaces, but adoption requires evidence that it does not compromise platform-native editing interactions.
- **[REQUIRED]** Touch, stylus, camera, media capture, background work, memory pressure, and interrupted network behavior must be first-class mobile requirements.

## 6. Technology direction and decision discipline

The following are preferred candidates, not permission to skip benchmarks, security review, licensing review, or ADRs.

- **[REQUIRED]** Pin versions and configurations in the repository; do not depend on unbounded “latest” versions.
- **[ADR REQUIRED]** Treat each technology below as a candidate until the relevant workload, security, licensing, operational, and compatibility evidence is recorded.

| Concern | Preferred direction / candidate |
|---|---|
| Desktop shell | Electron + React/TypeScript initially; validate packaging, memory, renderer isolation, update behavior, and security |
| Shared runtime | Rust for graph, operations, local journal integration, sync primitives, and performance-sensitive shared code |
| GPU abstraction | `wgpu` where it fits; isolate renderer backends behind a stable interface |
| 2D/vector primitives | Evaluate Skia for mature primitives; do not rebuild solved drawing algorithms without evidence |
| Typography | HarfBuzz, FreeType, ICU, plus platform-specific layout tests and a proprietary composition/constraint layer |
| Image I/O/color | OpenImageIO, OpenColorIO, LittleCMS, and format-specific RAW support subject to license and platform review |
| Video/media | FFmpeg plus platform-native decode/encode acceleration where appropriate; review exact build flags and obligations before distribution |
| 3D interchange | OpenUSD for composable scenes and glTF for runtime interchange/delivery where suitable |
| PDF | PDFium for initial rendering/interchange; native structured documents remain the editable model |
| Local journal | SQLite WAL with tested durability and recovery settings |
| Business/control-plane data | PostgreSQL is the default candidate |
| Project graph cloud store | Benchmark FoundationDB against the actual graph, transaction, recovery, and operational requirements before committing; retain a storage interface |
| Large objects | Managed S3/GCS/Azure-compatible object storage through an internal abstraction |
| Workflow orchestration | Temporal for durable, long-running render/publish/automation jobs after a focused operational evaluation |
| Service contracts | Protobuf and gRPC for typed internal service contracts where useful |
| Telemetry | OpenTelemetry instrumentation from the first vertical slice |
| Analytics/search/cache/streaming | ClickHouse, OpenSearch, Valkey, and Kafka are candidates for demonstrated needs; do not deploy them all before workload and operations justify them |
| Orchestration/IaC | Kubernetes and Terraform/OpenTofu are deployment candidates; choose hosting based on reliability, team operations, and cost evidence |

- **[DEFAULT]** Start with a small number of deployable components.
- **[REQUIRED]** Establish logical module boundaries first; split into independent services only when isolation, scale, release cadence, or reliability evidence justifies it.
- **[REQUIRED]** Avoid a ten-language backend.
- **[DEFAULT]** The likely language split is Rust for runtime and latency-sensitive work, Go for conventional control-plane services, and Python for model training and experimentation.
- **[ADR REQUIRED]** Record language and service-boundary decisions when they are difficult to reverse or affect shared contracts.

### Open-source and compatibility policy

- **[REQUIRED]** Reuse solved primitives; own the semantics that differentiate the platform.
- **[DEFAULT]** Candidate reuse includes codecs, file parsers, font shaping, graphics primitives, databases, workflow infrastructure, telemetry, container infrastructure, 3D interchange, and PDF compatibility.
- **[REQUIRED]** The platform’s differentiated work is the project graph, command semantics, editing workflows, collaboration behavior, history, cross-workspace interoperability, AI action model, asset semantics, enterprise policy, and user experience.
- **[REQUIRED]** Before adopting or distributing a dependency, record its source, version, license, enabled features, security posture, update policy, notices/attributions, and whether modifications must be published.
- **[REQUIRED]** FFmpeg, PDFium, fonts, codecs, and import/export libraries require especially careful configuration and licensing review.
- **[REQUIRED]** Avoid implementing a proprietary copy of a solved standard unless a measured product requirement demands it.
- **[REQUIRED]** Import/export compatibility is a product feature, not the internal architecture.
- **[REQUIRED]** Preserve originals during import, test parsers against malformed inputs, and document fidelity limits.
- **[DEFAULT]** Prioritize migration paths for common creative formats without making pixel-perfect round trips a precondition for the shared project model.

## 7. Cloud and service architecture

- **[REQUIRED]** Separate business/control-plane state from creative project data and immutable media.
- **[DEFAULT]** The expected logical services are identity/organization, project and sync, asset ingest/delivery, render/media jobs, realtime collaboration, and analytics. They may initially be modules or a small set of deployables.
- **[DEFAULT]** The proposed cloud building blocks are PostgreSQL for conventional business state, object storage for media, a project-state store selected by benchmark, a durable workflow engine for long jobs, and telemetry via OpenTelemetry.
- **[REQUIRED]** Cross-region replication, residency, backups, restore testing, tenant isolation, and recovery objectives must be designed and tested before enterprise claims are made.
- **[REQUIRED]** Use typed, versioned service contracts.
- **[REQUIRED]** Define timeouts, retries, idempotency, backpressure, cancellation, quotas, and failure behavior.
- **[REQUIRED]** A job retry must not create duplicate publications, invoices, or asset references.
- **[REQUIRED]** Rendering and publishing workflows should checkpoint completed steps and resume safely after worker or provider failures.
- **[ADR REQUIRED]** Record initial hosting, deployment shape, queue/workflow infrastructure, and thresholds for adding Kafka, ClickHouse, OpenSearch, or Kubernetes.

## 8. AI, automation, and plugins

### 8.1 AI command path

Product AI follows this path:

```text
Intent → context builder → planner → permission check → command plan
       → deterministic validation → normal creative command API → operation history
```

- **[REQUIRED]** The model must not receive direct database mutation access or a privileged state bypass.
- **[REQUIRED]** AI-produced operations are attributed to the initiating user and model/agent run.
- **[REQUIRED]** AI-produced operations are validated like human and plugin actions, previewable when appropriate, and undoable where feasible.
- **[REQUIRED]** Generated changes should preserve masks, parameters, prompts/provenance, source revisions, and editable graph structure rather than silently flattening work.
- **[DEFAULT]** Use model routing: deterministic algorithms for deterministic tasks, smaller/local models for bounded tasks, and frontier reasoning models for complex plans.
- **[REQUIRED]** Record model choice, input/output policy, cost limits, latency, and data handling.
- **[REQUIRED]** Never send customer project content to a provider without the applicable user/organization permission and product policy.
- **[ADR REQUIRED]** Define AI data handling, model routing, attribution, retention, and approval behavior before enabling external-impact or high-risk AI workflows.

### 8.2 Permissions and approvals

- **[REQUIRED]** Represent capabilities explicitly, for example `project.read`, `assets.read`, `design.write`, `video.write`, `media.generate`, and `render.create`.
- **[REQUIRED]** External-impact actions require separately granted capabilities and, where configured, human approval.
- **[REQUIRED]** Examples include external publishing, signing a contract, deleting a project, sending an invoice, or transferring funds.
- **[REQUIRED]** Enforce permissions in the command/service boundary, not just in UI visibility.

### 8.3 Plugins

- **[REQUIRED]** Run third-party extensions in a WASM capability sandbox or an equivalently constrained isolation boundary.
- **[REQUIRED]** A plugin requests narrow capabilities such as project read, selected object read, Design write, or access to a named domain.
- **[REQUIRED]** A plugin does not inherit the host filesystem, all projects, all network access, or credentials.
- **[REQUIRED]** Version and sign manifests.
- **[REQUIRED]** Allow organizations to allowlist or denylist plugins.
- **[REQUIRED]** Audit sensitive capability grants.
- **[DEFAULT]** First-party workspaces should use the same module registration model as third-party extensions where practical.
- **[REQUIRED]** Keep the core shell independent of specific workspace implementations.
- **[ADR REQUIRED]** Define the plugin SDK, sandbox boundary, signing model, capability model, and compatibility policy before third-party distribution.

## 9. Security, enterprise, and privacy

- **[REQUIRED]** Build tenant isolation, least privilege, secure local storage, authenticated sync, encryption in transit and at rest, key rotation, and auditability into the design.
- **[REQUIRED]** Security-sensitive authorization must be checked by a trusted service even when the client has cached project state.
- **[ASPIRATION]** Enterprise requirements to design for include OIDC/SAML, SCIM, role and relationship-based authorization, customer-managed keys, tenant home region/data residency, append-only audit events, retention and legal hold, DLP policy hooks, device policy, export controls, plugin allowlists, and organization-level AI settings.
- **[REQUIRED]** Implement enterprise capabilities in stages without making the core data model assume a single personal user owns everything.
- **[REQUIRED]** Agency and guest access must be scoped to specific project/workspace/object relationships.
- **[REQUIRED]** A guest authorized for `Campaign X / Video / Europe` must not gain access to unrelated campaigns, financial records, or master libraries by inference.
- **[REQUIRED]** Minimize source upload and telemetry.
- **[REQUIRED]** Clearly separate required service data, user-configured sync, AI-provider transfers, and optional diagnostics.
- **[REQUIRED]** Redact secrets and personal content from logs.
- **[REQUIRED]** Make deletion, export, retention, backup expiration, and legal hold behavior explicit and testable.
- **[ADR REQUIRED]** Define the identity provider, tenant authorization model, encryption/key management, residency, and backup architecture before enterprise claims or customer-managed keys are supported.

## 10. Target repository organization

Use this as a target shape, adapted to the actual repository through ADRs. **[REQUIRED]** Do not move existing code merely to match a diagram.

```text
apps/
  desktop/                 # Electron shell and desktop integration
  web/                     # browser surfaces and admin where needed
  ios/                     # native iOS/iPadOS client
  android/                  # native Android client
runtime/
  graph/ commands/ history/ assets/ sync/ permissions/ rendering/
engines/
  design/ image/ video/ motion/ audio/ spatial/ docs/
products/
  present/ publish/ live/ business/ automate/
services/
  identity/ project/ realtime/ assets/ rendering/ analytics/
schemas/                    # Protobuf, node schemas, migrations
sdk/                        # plugin and integration SDKs
tests/
  fixtures/ compatibility/ property/ fuzz/ golden/ replay/
docs/
  product/ architecture/ adr/ api/ operations/ security/
infrastructure/
```

- **[REQUIRED]** Keep generated code out of hand-edited source directories.
- **[REQUIRED]** Keep sample projects and test assets synthetic or licensed for testing.
- **[REQUIRED]** Avoid hidden dependencies between workspaces.
- **[REQUIRED]** Dependency direction should flow from workspace modules to stable runtime contracts, not from the shared runtime to each product.
- **[DEFAULT]** Adapt the target shape to the actual repository rather than forcing a wholesale reorganization.

## 11. Build phases and exit criteria

### Phase 0 — Repository and vertical foundation

- **[REQUIRED]** Establish build tooling, dependency policy, CI, formatting/lint rules, code ownership, docs/ADR templates, telemetry conventions, and test fixtures.
- **[REQUIRED]** Define the first graph schema, command envelope, local journal, asset identity, versioning, permission checks, and sync contract.

**Exit gate [REQUIRED]:** A Design object can be created, changed through a second workspace/module or test client using the same command API, survive forced process termination/reopen, be edited offline, and converge after reconnect without loss or duplication. Include operation replay, schema compatibility, hash verification, and recovery tests before starting broad editor work.

### Phase 1 — Design proving ground

- **[DEFAULT]** Implement document/canvas/frame/page, vector, text, image references, groups/components, styles, constraints, selection, inspector, comments, history, collaboration, import/export, and a useful mobile editing path.
- **[REQUIRED]** Focus on a narrow set of polished professional workflows.

**Exit gate [REQUIRED]:** Common Design workflows work with large but bounded projects, concurrent collaborators, undo/history, interrupted network, accessibility basics, and deterministic export. A component can be reused in another workspace without losing identity.

### Phase 2 — Image and Video

Image:

- **[DEFAULT]** Nondestructive layer/mask/adjustment graph, crop, color pipeline, retouching foundations, RAW/format ingest, tile rendering, and derived exports.

Video:

- **[DEFAULT]** Media index, proxies, timeline/clip/track operations, trim/split/ripple semantics, transitions, captions/transcripts, audio basics, hardware decode/encode, and render jobs.

**Exit gate [REQUIRED]:** Original media remains unchanged; project history can replay; render outputs match golden tests within documented tolerances; decoders and render jobs are isolated; interruption and low-storage behavior are understandable and recoverable.

### Phase 3 — Public beta

- **[DEFAULT]** Deliver Design, Image, and Video as workspaces in one application, with reliable collaboration, meaningful AI command workflows, and mobile clients that can complete real tasks.
- **[REQUIRED]** Keep the beta scope small enough to support well.

**Exit gate [REQUIRED]:** Upgrade, rollback, recovery, privacy, security review, crash/telemetry visibility, support diagnostics, and staged rollout are proven. Do not call the beta “always durable” until crash and fault-injection evidence supports the claim.

### Phase 4 — Paid professional V1

- **[DEFAULT]** Deepen professional typography, color, masks, RAW, codecs, performance, brand systems, team management, commercial AI controls, and plugin SDK.
- **[REQUIRED]** Establish service targets and support processes from observed workloads.

**Exit gate [REQUIRED]:** Billing entitlements, export and data policies, incident response, dependency licensing, compatibility guarantees, and support escalation are documented and tested.

### Phase 5 — Expand the platform

- **[ASPIRATION]** Add Motion, Audio, Present, Docs, and Publish using shared runtime primitives.
- **[ASPIRATION]** Then add Spatial, Live, Business, Automate, marketplace, and developer platform.
- **[REQUIRED]** New workspaces must reuse project identity, commands, permissions, history, assets, sync, render jobs, and telemetry instead of creating parallel infrastructure.

**Exit gate for every workspace [REQUIRED]:** It has a documented object model, command contract, permissions matrix, import/export story, quality gates, migration story, and cross-workspace example before it is called production-ready.

### Phase 6 — Enterprise scale

- **[ASPIRATION]** Invest in very large projects, tenant isolation, SSO/SCIM, audit, residency, customer-managed keys, retention, legal hold, DLP, LTS, guest/agency access, migration tooling, and restore drills.
- **[REQUIRED]** Set explicit scale limits and service objectives from production evidence.

## 12. Quality and reliability gates

- **[REQUIRED]** Testing is part of the feature, not a follow-up cleanup.
- **[REQUIRED]** Select tests based on the changed contract and risk; do not add tests that only duplicate implementation details.
- **[REQUIRED]** Every substantial change must use the applicable gates below:

- **Unit and integration tests:** command validation, graph invariants, permissions, storage, sync, import/export, service boundaries.
- **Property tests:** operation idempotency, replay determinism, graph validity, merge invariants, asset references, and undo/redo properties.
- **Fuzz tests:** project graphs, protobuf/messages, plugin manifests, SVG/PDF/font/image/media parsers, and network boundaries.
- **Compatibility tests:** historical schemas/projects, old clients against new services where supported, additive migrations, import/export fixtures.
- **Crash and replay tests:** terminate at each durability/sync boundary; restart and verify graph, operation order, and assets.
- **Offline/network fault tests:** loss, delay, duplication, reordering, auth refresh failure, partial upload, reconnect, and conflict behavior.
- **Golden render tests:** fixed fixtures, deterministic settings, documented GPU tolerances, visual diff review for intentional changes.
- **Performance/resource tests:** representative large projects, memory pressure, cold/warm load, rendering latency, battery/thermal behavior on mobile.
- **Security tests:** authorization boundaries, tenant isolation, plugin sandbox escapes, secret handling, dependency and artifact scanning.
- **Operational tests:** backup restore, job retry/idempotency, regional failover where supported, staged release and rollback.

- **[REQUIRED]** Each release must support staged rollout, feature flags for risky capabilities, a kill switch where appropriate, compatibility checks, and rollback.
- **[DEFAULT]** Enterprise users should have a slower/stable channel before significant workflow changes.
- **[REQUIRED]** Keep telemetry vendor-neutral through OpenTelemetry.
- **[REQUIRED]** Ensure a user edit can be traced across UI, command, local journal, sync, cloud storage, and remote collaborators without logging private creative content.

## 13. Coding-agent operating protocol

For every task:

1. **[REQUIRED]** Read this guide and the scoped instructions; inspect the repository, relevant docs, module boundaries, and working-tree status before editing.
2. **[REQUIRED]** Restate the concrete behavior and acceptance criteria in the task notes or PR description. If the requested change is ambiguous, make independent progress and record the smallest unresolved decision.
3. **[REQUIRED]** Identify affected contracts, data migrations, permissions, reliability properties, platform surfaces, and tests before changing implementation.
4. **[REQUIRED]** Make the smallest cohesive change that respects the shared runtime and existing module boundaries. Do not duplicate project state or add a workspace-specific bypass.
5. **[REQUIRED]** Update schemas, migrations, fixtures, docs, and compatibility handling with the implementation. Record architectural choices that are hard to reverse in an ADR.
6. **[REQUIRED]** Run the narrowest meaningful tests first, then the required broader checks for the touched subsystem. Report exact commands and outcomes; never claim a check passed if it was not run.
7. **[REQUIRED]** Review the diff for unrelated edits, generated artifacts, secrets, telemetry of user content, licensing effects, and accidental API/schema breaks.
8. **[REQUIRED]** Finish with a concise handoff: behavior changed, files/areas changed, verification performed, known limitations, and any follow-up decision.

- **[REQUIRED]** Agent-generated code receives the same or stronger validation as human-written code.
- **[REQUIRED]** Do not widen scope, silently change product semantics, fabricate benchmark results, weaken a failing test, remove compatibility behavior, or replace a deterministic command with model output to make a demo pass.
- **[ADR REQUIRED]** When a dependency or architecture choice is unresolved, create a short evidence-based spike and ADR rather than scattering speculative abstractions throughout the codebase.

## 14. Documentation and ADR standards

- **[REQUIRED]** Keep documentation near the code it governs and provide a small navigable index from the root README.
- **[REQUIRED]** Use Markdown for product, architecture, API, operational, and contributor documents.
- **[DEFAULT]** Diagrams should be maintained as Mermaid or another source format that can be reviewed in diffs; avoid screenshots as the sole architecture record.
- **[REQUIRED]** Update docs in the same change as behavior; stale docs are a defect.

### Required document types

- **README [REQUIRED]:** what the repo builds, prerequisites, setup, common commands, test commands, module map, links to product/architecture docs.
- **Product brief [REQUIRED]:** target user, problem, core workflow, scope, non-goals, success measures, accessibility and mobile requirements.
- **Architecture overview [REQUIRED]:** component responsibilities, data flow, trust boundaries, failure modes, and links to deeper designs.
- **ADR [REQUIRED WHEN APPLICABLE]:** decisions that are costly to reverse (runtime boundary, graph format, persistence, sync semantics, rendering backend, desktop shell, major dependencies).
- **API/schema docs [REQUIRED]:** ownership, versioning, validation, error behavior, compatibility rules, permissions, and examples.
- **Runbook [REQUIRED FOR OPERATIONALLY SIGNIFICANT SERVICES]:** symptoms, dashboards/logs, safe mitigations, recovery, escalation, and rollback.
- **Migration guide [REQUIRED WHEN SCHEMAS OR APIS CHANGE]:** old/new schema or API behavior, rollout order, backfill, rollback, and historical fixture coverage.

### ADR template

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

- **[REQUIRED]** Write docs for the reader’s task, define unfamiliar terms, state assumptions, and give executable examples where useful.
- **[REQUIRED]** Keep claims time-bounded when they depend on a library or service version.
- **[REQUIRED]** Document rationale and operational consequences, not only the selected technology.

## 15. Initial decision register

### Product and architecture principles already set

The following are **[REQUIRED]**:

- One platform with twelve planned first-party workspaces and one shared project model.
- Local durability precedes edit acknowledgment; offline creation is required.
- Shared project graph, typed creative command system, asset identity, history, and permissions are foundational.
- Workspace rendering is not a DOM-node canvas; shell UI and creative engines are separate.
- Mobile must support real creative work over shared semantics and native interactions.
- AI, automation, and plugins use governed commands and explicit capabilities.
- Open source should be reused aggressively for solved primitives, with licensing review; proprietary effort focuses on shared semantics and user experience.
- Product scope expands in stages; reliability and compatibility gates control progression.

### Decisions requiring evidence and ADRs

The following are **[ADR REQUIRED]**:

- Exact project graph schema, operation envelope, snapshot/compaction rules, and backward-compatibility policy.
- Local SQLite durability configuration, encrypted local storage, and recovery behavior across abrupt power loss.
- Cloud project-state store: benchmark FoundationDB against realistic project workloads and operational needs before committing.
- Electron shell, renderer/process boundaries, updater, and desktop security model.
- `wgpu`/Skia division and GPU fallback behavior across Apple, Windows, Linux, iOS, and Android.
- Collaboration/merge policy for each object family, including concurrent timeline edits and shared component updates.
- Identity provider, tenant authorization model, encryption/key management, residency, and backup architecture.
- FFmpeg/PDFium/font/codec configurations and distribution obligations.
- Initial hosting, deployment shape, queue/workflow infrastructure, and thresholds for adding Kafka, ClickHouse, OpenSearch, or Kubernetes.
- First supported import/export formats and fidelity promises.
- Mobile workspace scope for beta, based on device capabilities and user research.

- **[REQUIRED]** Do not turn candidates in this file into unreviewed implementation commitments.
- **[REQUIRED]** An ADR should state the tested workload, success criteria, alternatives, risks, license/security implications, and a clear revisit trigger.

## 16. First vertical slice specification

The first end-to-end proof is deliberately smaller than a creative suite:

1. **[REQUIRED]** Create a project and a typed Design frame with a text or vector child.
2. **[REQUIRED]** Apply a property change through a typed command; validate actor authorization and operation schema.
3. **[REQUIRED]** Commit the operation to the local durable journal before acknowledging it in the UI.
4. **[REQUIRED]** Terminate the editor process at controlled points, reopen the project, replay state, and verify the edit and history.
5. **[REQUIRED]** Disconnect networking, make more edits, reconnect, and verify idempotent convergence with a second client.
6. **[REQUIRED]** Reference one immutable asset by content identity and verify missing/corrupt asset behavior.
7. **[REQUIRED]** Open the same object through a second workspace module or test harness using the shared command and graph contracts.
8. **[REQUIRED]** Export a deterministic artifact and compare it with a golden fixture.
9. **[REQUIRED]** Exercise undo/redo, a named version, permission denial, and a schema upgrade using a historical fixture.
10. **[REQUIRED]** Emit traceable operational metadata without logging the creative content itself.

The slice is complete only when the tests prove data recovery, authorization, replay, offline editing, convergence, asset integrity, compatibility, and render behavior. **[REQUIRED]** A polished mockup or a cloud-only save path is not a substitute.

## 17. Definition of done

A change is ready for review when:

- **[REQUIRED]** Its user and system behavior match explicit acceptance criteria.
- **[REQUIRED]** It uses the shared project/command/asset model where applicable.
- **[REQUIRED]** Local durability, remote sync, undo/history, and permissions have defined behavior.
- **[REQUIRED]** Data formats and APIs are versioned and migration/compatibility impact is addressed.
- **[REQUIRED]** Failure, retry, cancellation, offline, and recovery paths are considered for relevant work.
- **[REQUIRED]** Meaningful tests and fixtures cover the changed contract and important regressions.
- **[REQUIRED]** Accessibility, mobile behavior, performance, privacy, and licensing are reviewed where relevant.
- **[REQUIRED]** Documentation and ADRs reflect the resulting behavior and decision.
- **[REQUIRED]** Required checks have run, the diff is focused, and the handoff names any limitations.

## 18. Product thesis in one sentence

**[ASPIRATION]** Use open-source infrastructure to avoid rebuilding solved primitives, use frontier AI to accelerate implementation and verification, and focus the human team on architecture, creative interaction, and judgment—so every editor, device, collaborator, plugin, and AI agent can operate on the same durable creative project.

## 19. Goal files and swarm execution

The assignable milestone catalog lives in `goals/README.md`; each goal has its own Markdown file with scope, exclusions, dependencies, acceptance criteria, verification, and agent ownership boundaries. Before implementing a goal, read the index, the goal file, and its dependency goals. Treat the dependency graph as the readiness contract and the goal's acceptance criteria as the completion gate. A lead agent owns integration and evidence; swarm workers must use accepted shared contracts and avoid concurrent edits to files owned by another worker.
