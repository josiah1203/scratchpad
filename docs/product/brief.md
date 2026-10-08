# Unified Creative Platform — working product brief

- Status: working hypothesis for foundation planning; not validated by user research.
- Scope: foundation and the Design proving ground, not a commitment to ship every planned workspace.
- Source of product and architecture requirements: [`AGENTS.md`](../../AGENTS.md).

## Product statement

Build one creative platform around a durable, portable project rather than a set of unrelated workspace files. Design, Image, Video, and future workspaces should share project identity, assets, history, permissions, and command semantics. The initial product work proves that foundation with a narrow Design workflow before broad editor development.

This is an architectural and product hypothesis from the project guide, not a claim that customer demand or willingness to pay has been validated.

## Target users and problem

Working target hypothesis: independent creators and small creative teams who move related work among design, image, and video workflows. Their project state, source assets, edits, and handoffs should remain connected when work crosses those boundaries.

The problem to validate is that separate workspace-owned files and save paths make shared assets, editability, history, permissions, and recovery difficult to preserve across tools and devices. The product should reduce this fragmentation without forcing users to adopt a single uniform editor interaction model.

## Core workflow

1. Create or open one project containing a typed Design document, frame, and text or vector object.
2. Reference an immutable asset from that project; preserve its identity and provenance rather than treating a workspace path as its identity.
3. Change the object through a validated, attributable command. Persist the operation locally before acknowledging it, and keep local commit distinct from remote replication.
4. Reopen after interruption and reconstruct the same editable graph and asset references from durable history.
5. Create and resolve a named version; later, open the same project through another workspace or client without flattening shared objects.
6. Export a documented portable project and import it elsewhere, with explicit compatibility and fidelity diagnostics.

Steps 4–6 describe the intended workflow and acceptance direction. The repository currently contains runtime prototypes, not a user-facing editor, sync client, or complete portable project format.

## Initial scope

The first phase establishes a synthetic project fixture, stable graph and asset identity, typed operations, local durability, recovery, permission checks, test/compatibility infrastructure, and an end-to-end Design proof. Work advances by the foundation exit gate in `AGENTS.md`, not by the number of workspaces implemented.

The Design proving ground should initially favor a small professional workflow over broad feature count: create a document/frame, place and edit text/vector content, reference an image, preserve history, reopen after interruption, and produce a deterministic export. Rendering, shell, sync, and mobile work must be justified by their relevant goals and evidence.

## Non-goals

- Reproducing the application boundaries or feature parity of a mature creative-suite vendor.
- Shipping all twelve planned workspaces in the initial product.
- A cloud-only save path, workspace-specific canonical asset copies, or direct state mutation outside the shared command path.
- Flattening editable objects simply to move them between workspaces.
- Calling a prototype production-ready or promising durability beyond tested failure conditions.
- Treating mobile as a reduced desktop viewport or a remote control.

## Success measures

These are proposed observable outcomes, not measured results or fixed service-level targets:

- Acknowledged edits survive controlled process termination and reopen with the same canonical graph state and operation history.
- Retried commands do not duplicate effects; offline edits converge deterministically after reconnect without loss.
- The same asset identity can be referenced by multiple workspace objects, and digest/length checks detect corruption or missing bytes with actionable diagnostics.
- A named version resolves to its exact graph and asset references.
- Portable export/import preserves graph structure, operation history, metadata, and source assets within documented compatibility limits; historical fixtures exercise supported migrations.
- A narrow Design task can be completed with keyboard access and a useful native touch/stylus workflow on supported mobile devices. Accessibility and device coverage must be verified rather than inferred from a desktop mockup.
- Product validation records whether the connected-project workflow is understandable and valuable to target users. No such research evidence is present in this repository yet.

## Accessibility and mobile requirements

The product must support keyboard navigation and editing, meaningful accessible names and focus behavior, readable contrast, and non-color-only status signals. Canvas interactions need accessible object navigation and inspector alternatives. These are requirements to test, not a claim that the current runtime prototype has a user interface.

Mobile is a native creative client over shared project semantics. Touch, stylus/Pencil, camera and media capture, background work, memory pressure, and interrupted connectivity are first-class design constraints. Mobile-specific interaction and the initial supported task set remain to be validated.

## Open product questions

- Which Design workflow and target user segment should anchor the first release?
- Which project interchange formats and fidelity guarantees matter first?
- What user-facing recovery, version, and sync language is clearest without exposing implementation jargon?
- What accessibility conformance target and supported device matrix should be release gates?
- Which collaboration and offline scenarios are essential for the initial proving ground?

Resolve these through user research and goal-specific evidence; do not turn the current hypotheses into product promises by repetition.
