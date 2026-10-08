# Capability authorization

`permissions.rs` defines named capabilities, grants scoped to an exact tenant and project, optional object-level narrowing, actor revocation, and operation-specific approval. Policy defaults to deny. Its audit record includes operation, actor, target, policy version, capability, and decision, but no creative payload.

`CommandCapabilityResolver` maps a validated command to its required capability from trusted workspace/runtime registration. A client-supplied capability string must not be used for this mapping. `PolicyEvaluator` is the shared evaluation interface. `PolicySnapshot` supports offline UX decisions; server-side command handlers must implement that interface with current authoritative policy and revalidate before remote mutation.

Project membership is a separate tenant/project/principal relationship, not a capability grant. The proposed current-record shape is in [`project-membership-v1.schema.json`](../../schemas/project-membership-v1.schema.json); it does not define role-to-capability mapping or authoritative persistence. Those service decisions remain open under G01/G07.

The initial capability set covers project/assets reads, Design/Image/Video writes, media generation, rendering, external publishing, signing, invoice sending, funds transfer, project deletion, and business writes. External-impact operations can require approval for a specific operation ID.
