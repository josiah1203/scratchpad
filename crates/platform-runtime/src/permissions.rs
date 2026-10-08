//! Capability-based command authorization.
//!
//! Policy evaluation is deny-by-default and scoped to an authenticated tenant,
//! project, and optionally one target object. A local snapshot supports offline
//! UX decisions; remote mutation must use an authoritative service-side
//! [`PolicyEvaluator`] implementation with current revocation state.

use std::{collections::BTreeSet, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    command::{Actor, ActorKind, CommandEnvelope},
    graph::{GraphError, ObjectId},
};

/// Named permissions enforced at command or service boundaries.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    ProjectRead,
    AssetsRead,
    DesignWrite,
    ImageWrite,
    VideoWrite,
    MediaGenerate,
    RenderCreate,
    ExternalPublish,
    DocumentSign,
    InvoiceSend,
    FundsTransfer,
    ProjectDelete,
    BusinessWrite,
}

impl Capability {
    /// Stable wire name for this capability.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProjectRead => "project.read",
            Self::AssetsRead => "assets.read",
            Self::DesignWrite => "design.write",
            Self::ImageWrite => "image.write",
            Self::VideoWrite => "video.write",
            Self::MediaGenerate => "media.generate",
            Self::RenderCreate => "render.create",
            Self::ExternalPublish => "external.publish",
            Self::DocumentSign => "document.sign",
            Self::InvoiceSend => "invoice.send",
            Self::FundsTransfer => "funds.transfer",
            Self::ProjectDelete => "project.delete",
            Self::BusinessWrite => "business.write",
        }
    }
}

/// Identity of a user, service, AI run, or plugin instance.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Principal {
    pub kind: ActorKind,
    pub id: String,
}

impl From<&Actor> for Principal {
    fn from(actor: &Actor) -> Self {
        Self { kind: actor.kind.clone(), id: actor.id.clone() }
    }
}

/// A grant limited to one tenant and project, with optional target-object narrowing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityGrant {
    pub principal: Principal,
    pub tenant_id: String,
    pub project_id: ObjectId,
    pub target_id: Option<ObjectId>,
    pub capability: Capability,
}

impl CapabilityGrant {
    /// Creates a project-scoped grant.
    #[must_use]
    pub fn project(
        principal: Principal,
        tenant_id: impl Into<String>,
        project_id: ObjectId,
        capability: Capability,
    ) -> Self {
        Self { principal, tenant_id: tenant_id.into(), project_id, target_id: None, capability }
    }

    /// Narrows this grant to one object.
    #[must_use]
    pub fn for_target(mut self, target_id: ObjectId) -> Self {
        self.target_id = Some(target_id);
        self
    }
}

/// Context derived from an authenticated command and trusted tenant scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationContext {
    pub actor: Actor,
    pub tenant_id: String,
    pub project_id: ObjectId,
    pub target_id: ObjectId,
    pub operation_id: ObjectId,
    pub capability: Capability,
}

impl AuthorizationContext {
    /// Builds context from the validated envelope and authenticated tenant.
    #[must_use]
    pub fn from_command(
        envelope: &CommandEnvelope,
        tenant_id: impl Into<String>,
        capability: Capability,
    ) -> Self {
        Self {
            actor: envelope.actor.clone(),
            tenant_id: tenant_id.into(),
            project_id: envelope.project_id.clone(),
            target_id: envelope.target_id.clone(),
            operation_id: envelope.operation_id.clone(),
            capability,
        }
    }
}

/// Final result of policy evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthorizationDecision {
    Allowed,
    Denied(DenialReason),
    ApprovalRequired,
}

impl AuthorizationDecision {
    /// True only when the action can proceed without further approval.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }
}

/// Reason a policy denied a command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DenialReason {
    RevokedPrincipal,
    NoMatchingGrant,
}

/// Content-free audit record for an authorization decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationAudit {
    pub policy_version: u64,
    pub operation_id: ObjectId,
    pub project_id: ObjectId,
    pub target_id: ObjectId,
    pub principal: Principal,
    pub capability: Capability,
    pub decision: AuthorizationDecision,
}

/// Policy interface. Server handlers implement this with authoritative current policy.
pub trait PolicyEvaluator: Send + Sync {
    /// Returns the policy revision used for this decision.
    fn version(&self) -> u64;

    /// Evaluates a command context using the implementation’s policy source.
    fn evaluate(&self, context: &AuthorizationContext) -> AuthorizationDecision;
}

/// Offline-safe, immutable-in-use snapshot of scoped grants and revocations.
#[derive(Clone, Debug, Default)]
pub struct PolicySnapshot {
    version: u64,
    grants: Vec<CapabilityGrant>,
    revoked: BTreeSet<Principal>,
    approvals_required: BTreeSet<Capability>,
    approved_operations: BTreeSet<ObjectId>,
}

impl PolicySnapshot {
    /// Creates an empty, deny-by-default policy snapshot.
    #[must_use]
    pub fn new(version: u64) -> Self {
        Self { version, ..Self::default() }
    }

    /// Adds a narrow capability grant.
    pub fn grant(&mut self, grant: CapabilityGrant) {
        self.grants.push(grant);
    }

    /// Revokes every grant belonging to a principal in this snapshot.
    pub fn revoke(&mut self, principal: Principal) {
        self.revoked.insert(principal);
    }

    /// Requires a separate approval for a capability.
    pub fn require_approval(&mut self, capability: Capability) {
        self.approvals_required.insert(capability);
    }

    /// Records approval of one operation, without broadly approving the capability.
    pub fn approve_operation(&mut self, operation_id: ObjectId) {
        self.approved_operations.insert(operation_id);
    }
}

impl PolicyEvaluator for PolicySnapshot {
    fn version(&self) -> u64 {
        self.version
    }

    fn evaluate(&self, context: &AuthorizationContext) -> AuthorizationDecision {
        let principal = Principal::from(&context.actor);
        if self.revoked.contains(&principal) {
            return AuthorizationDecision::Denied(DenialReason::RevokedPrincipal);
        }

        let grant_matches = self.grants.iter().any(|grant| {
            grant.principal == principal
                && grant.tenant_id == context.tenant_id
                && grant.project_id == context.project_id
                && grant.capability == context.capability
                && grant.target_id.as_ref().is_none_or(|id| id == &context.target_id)
        });
        if !grant_matches {
            return AuthorizationDecision::Denied(DenialReason::NoMatchingGrant);
        }

        if self.approvals_required.contains(&context.capability)
            && !self.approved_operations.contains(&context.operation_id)
        {
            AuthorizationDecision::ApprovalRequired
        } else {
            AuthorizationDecision::Allowed
        }
    }
}

/// Resolves required permissions from trusted workspace/runtime registration.
///
/// Implementations must consult registered node or command metadata, not a
/// capability string supplied by the client.
pub trait CommandCapabilityResolver: Send + Sync {
    /// Determines the permission needed for a validated command.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationError`] when the command has no registered
    /// permission mapping.
    fn required_capability(
        &self,
        envelope: &CommandEnvelope,
    ) -> Result<Capability, AuthorizationError>;
}

/// Authorization boundary failures. Denials remain decision records and are not errors.
#[derive(Debug)]
pub enum AuthorizationError {
    InvalidCommand(crate::command::CommandError),
    InvalidTenant,
    Graph(GraphError),
    UnregisteredCommand,
}

impl fmt::Display for AuthorizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCommand(error) => write!(formatter, "invalid command: {error}"),
            Self::InvalidTenant => formatter.write_str("authenticated tenant ID must not be empty"),
            Self::Graph(error) => error.fmt(formatter),
            Self::UnregisteredCommand => {
                formatter.write_str("command has no trusted capability registration")
            }
        }
    }
}

impl std::error::Error for AuthorizationError {}

impl From<GraphError> for AuthorizationError {
    fn from(error: GraphError) -> Self {
        Self::Graph(error)
    }
}

/// Validates a command, resolves its permission from trusted runtime metadata,
/// and emits a content-free policy audit record.
///
/// # Errors
///
/// Returns [`AuthorizationError`] for malformed commands, missing tenant
/// context, or an unregistered command family.
pub fn authorize_command<P: PolicyEvaluator, R: CommandCapabilityResolver>(
    envelope: &CommandEnvelope,
    authenticated_tenant_id: &str,
    resolver: &R,
    policy: &P,
) -> Result<AuthorizationAudit, AuthorizationError> {
    envelope.validate().map_err(AuthorizationError::InvalidCommand)?;
    if authenticated_tenant_id.trim().is_empty() {
        return Err(AuthorizationError::InvalidTenant);
    }
    let capability = resolver.required_capability(envelope)?;
    let context = AuthorizationContext::from_command(envelope, authenticated_tenant_id, capability);
    let decision = policy.evaluate(&context);
    Ok(AuthorizationAudit {
        policy_version: policy.version(),
        operation_id: envelope.operation_id.clone(),
        project_id: envelope.project_id.clone(),
        target_id: envelope.target_id.clone(),
        principal: Principal::from(&envelope.actor),
        capability,
        decision,
    })
}

/// Creates a principal from explicit authenticated identity fields.
#[must_use]
pub fn principal(kind: ActorKind, id: impl Into<String>) -> Principal {
    Principal { kind, id: id.into() }
}

/// Validates an object ID at a policy integration boundary.
///
/// # Errors
///
/// Returns [`AuthorizationError::Graph`] if the value is not a valid UUID.
pub fn validate_policy_object_id(id: &ObjectId) -> Result<(), AuthorizationError> {
    id.validate("authorization context")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{CausalContext, CreativeCommand, PropertyState};

    struct FixedCapabilityResolver(Capability);

    impl CommandCapabilityResolver for FixedCapabilityResolver {
        fn required_capability(
            &self,
            _envelope: &CommandEnvelope,
        ) -> Result<Capability, AuthorizationError> {
            Ok(self.0)
        }
    }

    fn id(number: u16) -> ObjectId {
        ObjectId::new(format!("00000000-0000-4000-8000-{number:012}"))
    }

    fn envelope(actor: Actor, operation_id: u16) -> CommandEnvelope {
        CommandEnvelope {
            operation_id: id(operation_id),
            project_id: id(2),
            target_id: id(10),
            schema_version: 1,
            actor,
            causal: CausalContext::new(
                "test-client",
                u64::from(operation_id),
                u64::from(operation_id),
            ),
            command: CreativeCommand::SetProperty {
                node_id: id(10),
                property: "opacity".to_owned(),
                value: serde_json::Value::from(0.5),
                previous_state: PropertyState::Absent,
            },
        }
    }

    fn grant_for(actor: &Actor, capability: Capability) -> CapabilityGrant {
        CapabilityGrant::project(Principal::from(actor), "tenant-a", id(2), capability)
    }

    #[test]
    fn authorizes_granted_actor_at_project_scope_and_records_no_content() {
        let command = envelope(Actor::user("user-1"), 1);
        let mut policy = PolicySnapshot::new(7);
        policy.grant(grant_for(&command.actor, Capability::DesignWrite));
        let audit = authorize_command(
            &command,
            "tenant-a",
            &FixedCapabilityResolver(Capability::DesignWrite),
            &policy,
        )
        .expect("valid command");
        assert_eq!(audit.decision, AuthorizationDecision::Allowed);
        assert_eq!(audit.policy_version, 7);
        assert_eq!(audit.capability.as_str(), "design.write");
    }

    #[test]
    fn defaults_to_deny_and_prevents_cross_project_and_cross_tenant_access() {
        let command = envelope(Actor::user("guest-1"), 1);
        let mut policy = PolicySnapshot::new(1);
        policy.grant(grant_for(&command.actor, Capability::ProjectRead));
        let resolver = FixedCapabilityResolver(Capability::ProjectRead);

        let cross_tenant = authorize_command(&command, "tenant-b", &resolver, &policy)
            .expect("policy decision should be returned");
        assert_eq!(
            cross_tenant.decision,
            AuthorizationDecision::Denied(DenialReason::NoMatchingGrant)
        );

        let mut other_project = command.clone();
        other_project.project_id = id(3);
        let cross_project = authorize_command(&other_project, "tenant-a", &resolver, &policy)
            .expect("policy decision should be returned");
        assert_eq!(
            cross_project.decision,
            AuthorizationDecision::Denied(DenialReason::NoMatchingGrant)
        );

        let no_grant =
            authorize_command(&envelope(Actor::user("unknown"), 2), "tenant-a", &resolver, &policy)
                .expect("policy decision should be returned");
        assert_eq!(no_grant.decision, AuthorizationDecision::Denied(DenialReason::NoMatchingGrant));
    }

    #[test]
    fn object_scopes_and_revocation_are_enforced_for_ai_and_plugins() {
        let ai = Actor::ai("planner", id(90));
        let plugin =
            Actor { kind: ActorKind::Plugin, id: "plugin-1".to_owned(), run_id: Some(id(91)) };
        let ai_command = envelope(ai.clone(), 1);
        let plugin_command = envelope(plugin.clone(), 2);
        let mut policy = PolicySnapshot::new(3);
        policy.grant(grant_for(&ai, Capability::DesignWrite).for_target(id(10)));
        policy.grant(
            CapabilityGrant::project(
                Principal::from(&plugin),
                "tenant-a",
                id(2),
                Capability::AssetsRead,
            )
            .for_target(id(11)),
        );
        let ai_result = authorize_command(
            &ai_command,
            "tenant-a",
            &FixedCapabilityResolver(Capability::DesignWrite),
            &policy,
        )
        .expect("AI command");
        assert_eq!(ai_result.decision, AuthorizationDecision::Allowed);
        let plugin_result = authorize_command(
            &plugin_command,
            "tenant-a",
            &FixedCapabilityResolver(Capability::AssetsRead),
            &policy,
        )
        .expect("plugin command");
        assert_eq!(
            plugin_result.decision,
            AuthorizationDecision::Denied(DenialReason::NoMatchingGrant)
        );

        policy.revoke(Principal::from(&ai));
        let revoked = authorize_command(
            &ai_command,
            "tenant-a",
            &FixedCapabilityResolver(Capability::DesignWrite),
            &policy,
        )
        .expect("revoked policy decision");
        assert_eq!(revoked.decision, AuthorizationDecision::Denied(DenialReason::RevokedPrincipal));
    }

    #[test]
    fn external_actions_require_operation_specific_approval() {
        let command = envelope(Actor::user("publisher"), 1);
        let mut policy = PolicySnapshot::new(2);
        policy.grant(grant_for(&command.actor, Capability::ExternalPublish));
        policy.require_approval(Capability::ExternalPublish);
        let resolver = FixedCapabilityResolver(Capability::ExternalPublish);

        assert_eq!(
            authorize_command(&command, "tenant-a", &resolver, &policy)
                .expect("policy decision")
                .decision,
            AuthorizationDecision::ApprovalRequired
        );
        policy.approve_operation(command.operation_id.clone());
        assert_eq!(
            authorize_command(&command, "tenant-a", &resolver, &policy)
                .expect("approved policy decision")
                .decision,
            AuthorizationDecision::Allowed
        );
    }
}
