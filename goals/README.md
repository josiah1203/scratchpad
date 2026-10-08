# Unified Creative Platform — Agent Goal Map

**Source of truth:** root AGENTS.md. Read it before taking a goal.  
**Status:** no goal is accepted yet; implementation progress is tracked in per-goal evidence, and readiness depends on satisfied dependencies and required ADRs.  
**Use:** assign one goal at a time to a lead agent or coordinated swarm. These files describe verifiable milestones, not a mandate to build every workspace before validating the product.

## Requirement labels

- **[REQUIRED]** Must be met unless a superseding decision explicitly changes it.
- **[DEFAULT]** Start here unless evidence or repository constraints justify another choice.
- **[ADR REQUIRED]** Evaluate and record before making the choice an implementation commitment.
- **[ASPIRATION]** Long-term direction, not a current delivery promise.

## How to execute a goal

1. Read AGENTS.md, this index, the goal file, and the goal's dependency files.
2. Confirm its dependencies are accepted and required ADRs are resolved. If not, produce the blocking evidence or ADR before dependent implementation.
3. Make the smallest cohesive change that satisfies the goal; update schemas, docs, fixtures, migrations, and tests with the change.
4. Keep one contract owner for shared schemas, command envelopes, storage interfaces, and permission semantics. Parallel workers implement against an accepted contract, not competing versions.
5. A swarm lead owns integration, acceptance evidence, and the final handoff. Assign workers to disjoint modules/files or independent test/benchmark work. Avoid concurrent edits to shared contract files.
6. Report exact verification commands, results, known limitations, and follow-up decisions. Never mark a goal done because a demo works if its acceptance criteria are unproven.

Suggested sizing: **S** is a narrow change; **M** is one bounded capability or integration milestone. If an M goal becomes several independently reviewable capabilities, split it into implementation tasks while keeping this goal as the integration gate. Use ADRs for consequential choices.

## Dependency waves

Dependencies indicate readiness, not a requirement to run goals serially. Within a wave, work may proceed in parallel when ownership boundaries are respected.

| Wave | Goals | Gate |
|---|---|---|
| 0 | G00 | Repository can be built and tested reproducibly. |
| 1 | G01, G11, G12 | Graph contract, renderer investigation, and test harness can start after baseline. |
| 2 | G02, G03 | Storage evaluation and command contract build on the graph model. |
| 3 | G04, G05, G07 | Local durability, asset identity, and permissions build on the command contract. |
| 4 | G06, G08, G09, G13 | History/recovery, sync, module API, and privacy-aware observability use accepted contracts. |
| 5 | G10 | Desktop shell integrates the module API. |
| 6 | G14 | Integrate the complete foundation vertical slice before broad editor work. |
| 7 | G15–G29 | Design, Image, Video, AI, and plugin goals proceed against shared runtime contracts. |
| 8 | G30–G38 | Add specialized workspaces and business workflows when their dependencies are ready. |
| 9 | G39–G40 | Enterprise hardening and release-scale proof follow production-like workflows. |

## Goal catalog

| ID | Goal | Phase | Dependencies | File |
|---|---|---|---|---|
| G00 | Repository baseline and agent workflow | Foundation | — | [G00-repository-baseline.md](00-foundation/G00-repository-baseline.md) |
| G01 | Project graph schema and object identity | Foundation | G00 | [G01-project-graph-schema.md](00-foundation/G01-project-graph-schema.md) |
| G02 | Cloud project-store benchmark and ADR | Foundation | G01 | [G02-cloud-store-evaluation.md](00-foundation/G02-cloud-store-evaluation.md) |
| G03 | Typed creative command model | Foundation | G01 | [G03-command-model.md](00-foundation/G03-command-model.md) |
| G04 | Local durable operation journal | Foundation | G03 | [G04-local-journal.md](00-foundation/G04-local-journal.md) |
| G05 | Asset identity, ingest, and local object store | Foundation | G03 | [G05-asset-identity-store.md](00-foundation/G05-asset-identity-store.md) |
| G06 | History, replay, snapshots, and recovery | Foundation | G04, G05 | [G06-history-replay-recovery.md](00-foundation/G06-history-replay-recovery.md) |
| G07 | Capability and authorization foundation | Foundation | G03 | [G07-capabilities-and-permissions.md](00-foundation/G07-capabilities-and-permissions.md) |
| G08 | Offline queue and deterministic sync | Foundation | G02, G04, G05, G07 | [G08-offline-sync.md](00-foundation/G08-offline-sync.md) |
| G09 | Workspace module contract and SDK | Foundation | G01, G03, G07 | [G09-workspace-module-sdk.md](00-foundation/G09-workspace-module-sdk.md) |
| G10 | Unified desktop shell and workspace tabs | Foundation | G00, G09 | [G10-desktop-shell.md](00-foundation/G10-desktop-shell.md) |
| G11 | 2D renderer and GPU architecture spike | Foundation | G00 | [G11-renderer-architecture-spike.md](00-foundation/G11-renderer-architecture-spike.md) |
| G12 | Test, fuzz, and compatibility harness | Foundation | G00 | [G12-test-and-compatibility-harness.md](00-foundation/G12-test-and-compatibility-harness.md) |
| G13 | Privacy-aware observability foundation | Foundation | G00, G07 | [G13-observability-and-privacy.md](00-foundation/G13-observability-and-privacy.md) |
| G14 | Foundation end-to-end vertical slice | Foundation | G01, G03, G04, G05, G06, G07, G08, G09, G10, G11, G12, G13 | [G14-foundation-vertical-slice.md](00-foundation/G14-foundation-vertical-slice.md) |
| G15 | Design objects and authoring commands | Design | G14 | [G15-design-object-model.md](10-design/G15-design-object-model.md) |
| G16 | Design canvas rendering and interaction | Design | G11, G15 | [G16-design-canvas-runtime.md](10-design/G16-design-canvas-runtime.md) |
| G17 | Professional text and layout foundation | Design | G15, G16 | [G17-design-typography-layout.md](10-design/G17-design-typography-layout.md) |
| G18 | Reusable components and brand system | Design | G15, G17 | [G18-components-and-brand-system.md](10-design/G18-components-and-brand-system.md) |
| G19 | Design collaboration, comments, and review | Design | G07, G08, G15, G18 | [G19-design-collaboration-review.md](10-design/G19-design-collaboration-review.md) |
| G20 | Design import, export, and portable project slice | Design | G15, G16, G17, G18 | [G20-design-interchange.md](10-design/G20-design-interchange.md) |
| G21 | Native iOS and iPadOS Design client | Design | G09, G15, G16 | [G21-ios-ipados-client.md](10-design/G21-ios-ipados-client.md) |
| G22 | Native Android Design client | Design | G09, G15, G16 | [G22-android-client.md](10-design/G22-android-client.md) |
| G23 | Nondestructive Image object model | Image and Video | G01, G03, G05, G06, G07 | [G23-image-nondestructive-model.md](20-image-video/G23-image-nondestructive-model.md) |
| G24 | Image I/O, color, tiles, and rendering | Image and Video | G11, G23 | [G24-image-io-color-rendering.md](20-image-video/G24-image-io-color-rendering.md) |
| G25 | Video/audio ingest, index, and proxies | Image and Video | G05, G11, G12 | [G25-video-media-ingest.md](20-image-video/G25-video-media-ingest.md) |
| G26 | Video timeline and editing semantics | Image and Video | G07, G25 | [G26-video-timeline-semantics.md](20-image-video/G26-video-timeline-semantics.md) |
| G27 | Durable render and export pipeline | Image and Video | G05, G11, G25, G26 | [G27-render-and-export-pipeline.md](20-image-video/G27-render-and-export-pipeline.md) |
| G28 | AI command planner and model router | Platform expansion | G03, G07, G14 | [G28-ai-command-planner.md](30-platform/G28-ai-command-planner.md) |
| G29 | Plugin SDK and capability sandbox | Platform expansion | G07, G09, G12 | [G29-plugin-sdk-sandbox.md](30-platform/G29-plugin-sdk-sandbox.md) |
| G30 | Docs workspace and PDF workflows | Platform expansion | G05, G07, G15, G17 | [G30-docs-workspace.md](30-platform/G30-docs-workspace.md) |
| G31 | Present workspace and live deck delivery | Platform expansion | G15, G18, G27 | [G31-present-workspace.md](30-platform/G31-present-workspace.md) |
| G32 | Motion workspace and live compositions | Platform expansion | G16, G26, G27 | [G32-motion-workspace.md](30-platform/G32-motion-workspace.md) |
| G33 | Audio workspace and transcript editing | Platform expansion | G25, G26, G27 | [G33-audio-workspace.md](30-platform/G33-audio-workspace.md) |
| G34 | Publish campaigns and channel delivery | Platform expansion | G07, G27, G28 | [G34-publish-workspace.md](30-platform/G34-publish-workspace.md) |
| G35 | Spatial workspace and 3D interchange | Platform expansion | G05, G11, G15 | [G35-spatial-workspace.md](30-platform/G35-spatial-workspace.md) |
| G36 | Live production and post-stream clipping | Platform expansion | G25, G27, G29 | [G36-live-workspace.md](30-platform/G36-live-workspace.md) |
| G37 | Creator and agency business workflows | Platform expansion | G07, G30, G34 | [G37-business-workspace.md](30-platform/G37-business-workspace.md) |
| G38 | Cross-workspace automation and agent runs | Platform expansion | G27, G28, G29, G34, G37 | [G38-automation-workspace.md](30-platform/G38-automation-workspace.md) |
| G39 | Enterprise identity, governance, and data controls | Enterprise | G07, G08, G13, G29, G37 | [G39-enterprise-controls.md](40-enterprise/G39-enterprise-controls.md) |
| G40 | Release, reliability, and large-project readiness | Enterprise | G12, G13, G14, G27, G39 | [G40-release-reliability-scale.md](40-enterprise/G40-release-reliability-scale.md) |

## Phase exit rules

- **Foundation:** G14 is incomplete until edits are locally durable, replayable, permissioned, offline-capable, convergent, asset-safe, and rendered/exported with test evidence.
- **Design/Image/Video:** Do not promote these as professional workflows until object semantics, import/export limits, renderer/media behavior, history, collaboration, and recovery are tested.
- **Platform expansion:** Every workspace needs its own object model, permissions matrix, import/export story, failure handling, and cross-workspace example.
- **Enterprise:** No compliance, durability, or scale claim without evidence from security review, fault injection, restore drills, and measured workloads.

## Goal-file standard

Every goal states outcome, scope, exclusions, dependencies, acceptance criteria, verification, and agent ownership. Update the index if a goal is split, superseded, or blocked by a new architectural decision. Preserve goal IDs once referenced; mark superseded files rather than silently reusing IDs.
