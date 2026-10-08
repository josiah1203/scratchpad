//! Typed, attributable creative commands.
//!
//! Commands are the only accepted state-change path. This module validates the
//! envelope and representative reversible command families; applying commands
//! to a durable graph is intentionally owned by later foundation goals.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    graph::{GraphError, Node, ObjectId, ProjectGraph},
    journal::{AppendOutcome, JournalCursor, JournalError, LocalJournal},
    permissions::{
        AuthorizationAudit, AuthorizationError, CommandCapabilityResolver, PolicyEvaluator,
        authorize_command,
    },
};

const CURRENT_COMMAND_SCHEMA_VERSION: u32 = 1;

/// The principal responsible for a command.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Actor {
    pub kind: ActorKind,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<ObjectId>,
}

/// The class of principal that issued a command.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    User,
    Service,
    Ai,
    Plugin,
}

impl Actor {
    /// Creates a human-user actor.
    #[must_use]
    pub fn user(id: impl Into<String>) -> Self {
        Self { kind: ActorKind::User, id: id.into(), run_id: None }
    }

    /// Creates an AI actor attributed to an agent/model run.
    #[must_use]
    pub fn ai(id: impl Into<String>, run_id: ObjectId) -> Self {
        Self { kind: ActorKind::Ai, id: id.into(), run_id: Some(run_id) }
    }

    fn validate(&self) -> Result<(), CommandError> {
        if self.id.trim().is_empty() {
            return Err(CommandError::EmptyActorId);
        }
        if matches!(self.kind, ActorKind::Ai | ActorKind::Plugin) && self.run_id.is_none() {
            return Err(CommandError::MissingRunId(self.kind.clone()));
        }
        if let Some(run_id) = &self.run_id {
            run_id.validate("actor.run_id")?;
        }
        Ok(())
    }
}

/// Causal and logical ordering metadata for an operation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CausalContext {
    pub client_id: String,
    pub client_sequence: u64,
    pub lamport: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_operation_id: Option<ObjectId>,
}

impl CausalContext {
    /// Creates causal metadata for a client operation stream.
    #[must_use]
    pub fn new(client_id: impl Into<String>, client_sequence: u64, lamport: u64) -> Self {
        Self { client_id: client_id.into(), client_sequence, lamport, parent_operation_id: None }
    }

    fn validate(&self) -> Result<(), CommandError> {
        if self.client_id.trim().is_empty() {
            return Err(CommandError::EmptyClientId);
        }
        if self.client_sequence == 0 || self.lamport == 0 {
            return Err(CommandError::InvalidOrdering);
        }
        if self.client_sequence > i64::MAX as u64 || self.lamport > i64::MAX as u64 {
            return Err(CommandError::OrderingOutOfRange);
        }
        if let Some(parent_operation_id) = &self.parent_operation_id {
            parent_operation_id.validate("causal.parent_operation_id")?;
        }
        Ok(())
    }
}

/// The typed command envelope shared by human, AI, plugin, and service clients.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandEnvelope {
    pub operation_id: ObjectId,
    pub project_id: ObjectId,
    pub target_id: ObjectId,
    pub schema_version: u32,
    pub actor: Actor,
    pub causal: CausalContext,
    pub command: CreativeCommand,
}

impl CommandEnvelope {
    /// Validates all envelope and command invariants.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError`] when identity, attribution, ordering, schema,
    /// or typed-command validation fails.
    pub fn validate(&self) -> Result<(), CommandError> {
        if self.schema_version != CURRENT_COMMAND_SCHEMA_VERSION {
            return Err(CommandError::UnsupportedSchema(self.schema_version));
        }
        self.operation_id.validate("operation_id")?;
        self.project_id.validate("project_id")?;
        self.target_id.validate("target_id")?;
        self.actor.validate()?;
        self.causal.validate()?;
        self.command.validate(&self.target_id)
    }
}

/// The first command families supported by the shared contract.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "args", rename_all = "snake_case", deny_unknown_fields)]
pub enum CreativeCommand {
    CreateNode {
        node_id: ObjectId,
        node_type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_id: Option<ObjectId>,
        #[serde(default)]
        properties: BTreeMap<String, Value>,
    },
    SetProperty {
        node_id: ObjectId,
        property: String,
        value: Value,
        previous_state: PropertyState,
    },
    ClearProperty {
        node_id: ObjectId,
        property: String,
        previous_value: Value,
    },
    MoveNode {
        node_id: ObjectId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_id: Option<ObjectId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        before_node_id: Option<ObjectId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        previous_parent_id: Option<ObjectId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        previous_before_node_id: Option<ObjectId>,
    },
    DeleteNode {
        node_id: ObjectId,
    },
    RestoreNode {
        node_id: ObjectId,
    },
}

/// The exact prior state expected by a property-setting command.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case", deny_unknown_fields)]
pub enum PropertyState {
    Absent,
    Present(Value),
}

impl CreativeCommand {
    /// Returns the target object for this command.
    #[must_use]
    pub fn target_id(&self) -> &ObjectId {
        match self {
            Self::CreateNode { node_id, .. }
            | Self::SetProperty { node_id, .. }
            | Self::ClearProperty { node_id, .. }
            | Self::MoveNode { node_id, .. }
            | Self::DeleteNode { node_id }
            | Self::RestoreNode { node_id } => node_id,
        }
    }

    /// Produces a compensating command for the supported command families.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::MissingUndoContext`] when an inverse needs
    /// prior state that was not recorded in the original command.
    pub fn inverse(&self) -> Result<Self, CommandError> {
        match self {
            Self::CreateNode { node_id, .. } | Self::RestoreNode { node_id } => {
                Ok(Self::DeleteNode { node_id: node_id.clone() })
            }
            Self::SetProperty { node_id, property, value, previous_state } => {
                match previous_state {
                    PropertyState::Present(previous_value) => Ok(Self::SetProperty {
                        node_id: node_id.clone(),
                        property: property.clone(),
                        value: previous_value.clone(),
                        previous_state: PropertyState::Present(value.clone()),
                    }),
                    PropertyState::Absent => Ok(Self::ClearProperty {
                        node_id: node_id.clone(),
                        property: property.clone(),
                        previous_value: value.clone(),
                    }),
                }
            }
            Self::ClearProperty { node_id, property, previous_value } => Ok(Self::SetProperty {
                node_id: node_id.clone(),
                property: property.clone(),
                value: previous_value.clone(),
                previous_state: PropertyState::Absent,
            }),
            Self::MoveNode {
                node_id,
                parent_id,
                before_node_id,
                previous_parent_id,
                previous_before_node_id,
            } => Ok(Self::MoveNode {
                node_id: node_id.clone(),
                parent_id: previous_parent_id.clone(),
                before_node_id: previous_before_node_id.clone(),
                previous_parent_id: parent_id.clone(),
                previous_before_node_id: before_node_id.clone(),
            }),
            Self::DeleteNode { node_id } => Ok(Self::RestoreNode { node_id: node_id.clone() }),
        }
    }

    fn validate(&self, target_id: &ObjectId) -> Result<(), CommandError> {
        if self.target_id() != target_id {
            return Err(CommandError::TargetMismatch {
                envelope_target: target_id.clone(),
                command_target: self.target_id().clone(),
            });
        }

        self.target_id().validate("command.target_id")?;
        match self {
            Self::CreateNode { node_type, parent_id, properties, .. } => {
                if node_type.trim().is_empty() {
                    return Err(CommandError::EmptyNodeType);
                }
                validate_optional_id(parent_id.as_ref(), "create.parent_id")?;
                if properties.keys().any(|key| key.trim().is_empty()) {
                    return Err(CommandError::EmptyProperty);
                }
            }
            Self::SetProperty { property, .. } | Self::ClearProperty { property, .. } => {
                if property.trim().is_empty() {
                    return Err(CommandError::EmptyProperty);
                }
            }
            Self::MoveNode {
                node_id,
                parent_id,
                before_node_id,
                previous_parent_id,
                previous_before_node_id,
            } => {
                validate_optional_id(parent_id.as_ref(), "move.parent_id")?;
                validate_optional_id(before_node_id.as_ref(), "move.before_node_id")?;
                validate_optional_id(previous_parent_id.as_ref(), "move.previous_parent_id")?;
                validate_optional_id(
                    previous_before_node_id.as_ref(),
                    "move.previous_before_node_id",
                )?;
                if before_node_id.as_ref() == Some(node_id)
                    || previous_before_node_id.as_ref() == Some(node_id)
                {
                    return Err(CommandError::SelfReference);
                }
            }
            Self::DeleteNode { .. } | Self::RestoreNode { .. } => {}
        }
        Ok(())
    }
}

fn validate_optional_id(id: Option<&ObjectId>, context: &str) -> Result<(), CommandError> {
    if let Some(id) = id {
        id.validate(context)?;
    }
    Ok(())
}

/// Result of recording a command operation ID in an idempotency ledger.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordOutcome {
    FirstSeen,
    Duplicate,
}

/// In-memory duplicate guard used by command tests and adapters.
///
/// The durable operation journal owns persistence and restart behavior in G04;
/// this type makes the retry contract explicit without pretending to be that journal.
#[derive(Default)]
pub struct OperationLedger {
    operation_ids: BTreeSet<ObjectId>,
}

impl OperationLedger {
    /// Validates and records an operation ID exactly once for this process.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError`] if the envelope is invalid.
    pub fn record(&mut self, envelope: &CommandEnvelope) -> Result<RecordOutcome, CommandError> {
        envelope.validate()?;
        if self.operation_ids.insert(envelope.operation_id.clone()) {
            Ok(RecordOutcome::FirstSeen)
        } else {
            Ok(RecordOutcome::Duplicate)
        }
    }
}

/// Command validation failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandError {
    Graph(GraphError),
    UnsupportedSchema(u32),
    EmptyActorId,
    MissingRunId(ActorKind),
    EmptyClientId,
    InvalidOrdering,
    OrderingOutOfRange,
    TargetMismatch { envelope_target: ObjectId, command_target: ObjectId },
    EmptyNodeType,
    EmptyProperty,
    SelfReference,
    MissingUndoContext,
}

impl From<GraphError> for CommandError {
    fn from(error: GraphError) -> Self {
        Self::Graph(error)
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Graph(error) => error.fmt(formatter),
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported command schema {version}")
            }
            Self::EmptyActorId => formatter.write_str("actor ID must not be empty"),
            Self::MissingRunId(kind) => write!(formatter, "{kind:?} commands require a run ID"),
            Self::EmptyClientId => formatter.write_str("client ID must not be empty"),
            Self::InvalidOrdering => {
                formatter.write_str("client sequence and Lamport clock must be positive")
            }
            Self::OrderingOutOfRange => {
                formatter.write_str("client sequence and Lamport clock must fit SQLite integers")
            }
            Self::TargetMismatch { envelope_target, command_target } => write!(
                formatter,
                "envelope target {} does not match command target {}",
                envelope_target.as_str(),
                command_target.as_str()
            ),
            Self::EmptyNodeType => formatter.write_str("node type must not be empty"),
            Self::EmptyProperty => formatter.write_str("property names must not be empty"),
            Self::SelfReference => formatter.write_str("a node cannot be ordered before itself"),
            Self::MissingUndoContext => {
                formatter.write_str("command lacks the state needed to undo it")
            }
        }
    }
}

impl std::error::Error for CommandError {}

/// Failures while applying a validated operation to the materialized graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandApplyError {
    Graph(GraphError),
    InvalidCommand(String),
    ProjectMismatch { expected: ObjectId, actual: ObjectId },
    NodeAlreadyExists(ObjectId),
    MissingNode(ObjectId),
    DeletedNode(ObjectId),
    NodeIsNotDeleted(ObjectId),
    ParentPreconditionFailed(ObjectId),
    SiblingPreconditionFailed(ObjectId),
    PropertyPreconditionFailed { node_id: ObjectId, property: String },
    InvalidParent(ObjectId),
    InvalidBeforeNode(ObjectId),
    OrderExhausted,
}

impl From<GraphError> for CommandApplyError {
    fn from(error: GraphError) -> Self {
        Self::Graph(error)
    }
}

impl fmt::Display for CommandApplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Graph(error) => error.fmt(formatter),
            Self::InvalidCommand(error) => write!(formatter, "invalid command envelope: {error}"),
            Self::ProjectMismatch { expected, actual } => write!(
                formatter,
                "command project {} does not match graph project {}",
                expected.as_str(),
                actual.as_str()
            ),
            Self::NodeAlreadyExists(id) => write!(formatter, "node {} already exists", id.as_str()),
            Self::MissingNode(id) => write!(formatter, "node {} does not exist", id.as_str()),
            Self::DeletedNode(id) => write!(formatter, "node {} is deleted", id.as_str()),
            Self::NodeIsNotDeleted(id) => write!(formatter, "node {} is not deleted", id.as_str()),
            Self::ParentPreconditionFailed(id) => {
                write!(formatter, "node {} has changed parent", id.as_str())
            }
            Self::SiblingPreconditionFailed(id) => {
                write!(formatter, "node {} has changed sibling order", id.as_str())
            }
            Self::PropertyPreconditionFailed { node_id, property } => write!(
                formatter,
                "node {} property {property} no longer has the expected value",
                node_id.as_str()
            ),
            Self::InvalidParent(id) => {
                write!(formatter, "parent {} is missing or deleted", id.as_str())
            }
            Self::InvalidBeforeNode(id) => {
                write!(formatter, "before-node {} is missing or in another parent", id.as_str())
            }
            Self::OrderExhausted => formatter.write_str("sibling order key range is exhausted"),
        }
    }
}

impl std::error::Error for CommandApplyError {}

/// Result returned only after local journal commit succeeds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalCommandReceipt {
    pub append: AppendOutcome,
    pub authorization: AuthorizationAudit,
}

/// Errors from the local command boundary.
#[derive(Debug)]
pub enum CommandServiceError {
    AuthorizationBoundary(AuthorizationError),
    AuthorizationRejected(AuthorizationAudit),
    Apply(CommandApplyError),
    Journal(JournalError),
}

impl fmt::Display for CommandServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthorizationBoundary(error) => {
                write!(formatter, "authorization check failed: {error}")
            }
            Self::AuthorizationRejected(audit) => {
                write!(formatter, "command was not authorized: {:?}", audit.decision)
            }
            Self::Apply(error) => {
                write!(formatter, "command cannot apply to current graph: {error}")
            }
            Self::Journal(error) => write!(formatter, "local journal commit failed: {error}"),
        }
    }
}

impl std::error::Error for CommandServiceError {}

impl From<CommandApplyError> for CommandServiceError {
    fn from(error: CommandApplyError) -> Self {
        Self::Apply(error)
    }
}

impl From<JournalError> for CommandServiceError {
    fn from(error: JournalError) -> Self {
        Self::Journal(error)
    }
}

/// Shared local command boundary and operation-replay entry point.
pub struct CommandService;

impl CommandService {
    /// Validates, authorizes, stages the graph change, journals it, and then
    /// exposes the new graph state. A caller must acknowledge only on `Ok`.
    ///
    /// If validation, authorization, graph application, or journal commit
    /// fails, the supplied graph is unchanged. A duplicate delivery returns a
    /// receipt for a previously committed operation; callers must
    /// recover the graph from the journal before accepting new commands after
    /// process restart.
    ///
    /// # Errors
    ///
    /// Returns [`CommandServiceError`] for invalid commands, denied policy,
    /// conflicting graph preconditions, or journal failure.
    pub fn commit_local<P: PolicyEvaluator, R: CommandCapabilityResolver>(
        graph: &mut ProjectGraph,
        journal: &mut LocalJournal,
        envelope: &CommandEnvelope,
        authenticated_tenant_id: &str,
        resolver: &R,
        policy: &P,
    ) -> Result<LocalCommandReceipt, CommandServiceError> {
        let authorization = authorize_command(envelope, authenticated_tenant_id, resolver, policy)
            .map_err(CommandServiceError::AuthorizationBoundary)?;
        if !authorization.decision.is_allowed() {
            return Err(CommandServiceError::AuthorizationRejected(authorization));
        }

        if let Some(existing) = journal.read(&envelope.operation_id)? {
            if existing.envelope == *envelope {
                return Ok(LocalCommandReceipt {
                    append: AppendOutcome::AlreadyCommitted,
                    authorization,
                });
            }
            return Err(CommandServiceError::Journal(JournalError::OperationIdCollision(
                envelope.operation_id.clone(),
            )));
        }

        let mut candidate = graph.clone();
        candidate.apply_envelope_to_staged(envelope)?;
        let append = journal.append_with_materialized(envelope, graph, &candidate)?;
        if append == AppendOutcome::Committed {
            *graph = candidate;
        }
        Ok(LocalCommandReceipt { append, authorization })
    }

    /// Replays locally committed operations after an optional snapshot cursor.
    /// The graph is replaced only after every operation and invariant validates.
    ///
    /// # Errors
    ///
    /// Returns [`CommandApplyError`] if an operation conflicts with the graph,
    /// or [`JournalError`] if reading or decoding the journal fails.
    pub fn replay_from(
        graph: &mut ProjectGraph,
        journal: &LocalJournal,
        project_id: &ObjectId,
        after: Option<JournalCursor>,
    ) -> Result<usize, ReplayError> {
        if graph.project_id != *project_id {
            return Err(CommandApplyError::ProjectMismatch {
                expected: project_id.clone(),
                actual: graph.project_id.clone(),
            }
            .into());
        }

        let mut candidate = graph.clone();
        let mut cursor = after;
        let mut replayed = 0;
        loop {
            let page = journal.read_range(project_id, cursor.as_ref(), 1_000)?;
            if page.is_empty() {
                break;
            }
            for entry in &page {
                candidate.apply_envelope_to_staged(&entry.envelope)?;
                candidate.validate()?;
                cursor = Some(JournalCursor {
                    lamport: entry.envelope.causal.lamport,
                    client_id: entry.envelope.causal.client_id.clone(),
                    client_sequence: entry.envelope.causal.client_sequence,
                });
                replayed += 1;
            }
            if page.len() < 1_000 {
                break;
            }
        }
        *graph = candidate;
        Ok(replayed)
    }
}

/// Replay failure from invalid state transition or a damaged journal payload.
#[derive(Debug)]
pub enum ReplayError {
    Apply(CommandApplyError),
    Journal(JournalError),
}

impl fmt::Display for ReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Apply(error) => write!(formatter, "operation replay failed: {error}"),
            Self::Journal(error) => write!(formatter, "journal read failed during replay: {error}"),
        }
    }
}

impl std::error::Error for ReplayError {}

impl From<CommandApplyError> for ReplayError {
    fn from(error: CommandApplyError) -> Self {
        Self::Apply(error)
    }
}

impl From<GraphError> for ReplayError {
    fn from(error: GraphError) -> Self {
        Self::Apply(CommandApplyError::Graph(error))
    }
}

impl From<JournalError> for ReplayError {
    fn from(error: JournalError) -> Self {
        Self::Journal(error)
    }
}

impl ProjectGraph {
    #[cfg(test)]
    fn apply_envelope(&mut self, envelope: &CommandEnvelope) -> Result<(), CommandApplyError> {
        let mut candidate = self.clone();
        candidate.apply_envelope_to_staged(envelope)?;
        *self = candidate;
        Ok(())
    }

    pub(crate) fn apply_envelope_to_staged(
        &mut self,
        envelope: &CommandEnvelope,
    ) -> Result<(), CommandApplyError> {
        envelope
            .validate()
            .map_err(|error| CommandApplyError::InvalidCommand(error.to_string()))?;
        if self.project_id != envelope.project_id {
            return Err(CommandApplyError::ProjectMismatch {
                expected: self.project_id.clone(),
                actual: envelope.project_id.clone(),
            });
        }
        self.apply_command_unchecked(&envelope.command)
    }

    fn apply_command_unchecked(
        &mut self,
        command: &CreativeCommand,
    ) -> Result<(), CommandApplyError> {
        match command {
            CreativeCommand::CreateNode { node_id, node_type, parent_id, properties } => {
                self.apply_create_node(node_id, node_type, parent_id.as_ref(), properties)?;
            }
            CreativeCommand::SetProperty { node_id, property, value, previous_state } => {
                self.apply_set_property(node_id, property, value, previous_state)?;
            }
            CreativeCommand::ClearProperty { node_id, property, previous_value } => {
                self.apply_clear_property(node_id, property, previous_value)?;
            }
            CreativeCommand::MoveNode {
                node_id,
                parent_id,
                before_node_id,
                previous_parent_id,
                previous_before_node_id,
            } => self.apply_move_node(
                node_id,
                parent_id.as_ref(),
                before_node_id.as_ref(),
                previous_parent_id.as_ref(),
                previous_before_node_id.as_ref(),
            )?,
            CreativeCommand::DeleteNode { node_id } => self.apply_delete_node(node_id)?,
            CreativeCommand::RestoreNode { node_id } => self.apply_restore_node(node_id)?,
        }
        self.validate()?;
        Ok(())
    }

    fn apply_create_node(
        &mut self,
        node_id: &ObjectId,
        node_type: &str,
        parent_id: Option<&ObjectId>,
        properties: &BTreeMap<String, Value>,
    ) -> Result<(), CommandApplyError> {
        if self.node(node_id).is_some() {
            return Err(CommandApplyError::NodeAlreadyExists(node_id.clone()));
        }
        if let Some(parent_id) = parent_id {
            self.require_active(parent_id)
                .map_err(|_| CommandApplyError::InvalidParent(parent_id.clone()))?;
        }
        let max_order = self
            .nodes
            .iter()
            .filter(|node| {
                node.parent_id.as_ref() == parent_id && !self.effectively_deleted(&node.id)
            })
            .map(|node| node.order)
            .max()
            .unwrap_or(-1024);
        let order = max_order.checked_add(1024).ok_or(CommandApplyError::OrderExhausted)?;
        self.nodes.push(Node {
            id: node_id.clone(),
            node_type: node_type.to_owned(),
            parent_id: parent_id.cloned(),
            references: Vec::new(),
            name: None,
            asset_ref: None,
            properties: properties.clone(),
            order,
            deleted: false,
            extensions: BTreeMap::new(),
        });
        Ok(())
    }

    fn apply_set_property(
        &mut self,
        node_id: &ObjectId,
        property: &str,
        value: &Value,
        previous_state: &PropertyState,
    ) -> Result<(), CommandApplyError> {
        let node = self.active_node_mut(node_id)?;
        let matches_previous_state = match previous_state {
            PropertyState::Absent => !node.properties.contains_key(property),
            PropertyState::Present(expected) => node.properties.get(property) == Some(expected),
        };
        if !matches_previous_state {
            return Err(CommandApplyError::PropertyPreconditionFailed {
                node_id: node_id.clone(),
                property: property.to_owned(),
            });
        }
        node.properties.insert(property.to_owned(), value.clone());
        Ok(())
    }

    fn apply_clear_property(
        &mut self,
        node_id: &ObjectId,
        property: &str,
        previous_value: &Value,
    ) -> Result<(), CommandApplyError> {
        let node = self.active_node_mut(node_id)?;
        if node.properties.get(property) != Some(previous_value) {
            return Err(CommandApplyError::PropertyPreconditionFailed {
                node_id: node_id.clone(),
                property: property.to_owned(),
            });
        }
        node.properties.remove(property);
        Ok(())
    }

    fn apply_move_node(
        &mut self,
        node_id: &ObjectId,
        parent_id: Option<&ObjectId>,
        before_node_id: Option<&ObjectId>,
        previous_parent_id: Option<&ObjectId>,
        previous_before_node_id: Option<&ObjectId>,
    ) -> Result<(), CommandApplyError> {
        self.require_active(node_id)?;
        let node =
            self.node(node_id).ok_or_else(|| CommandApplyError::MissingNode(node_id.clone()))?;
        let current_parent = node.parent_id.clone();
        if current_parent.as_ref() != previous_parent_id {
            return Err(CommandApplyError::ParentPreconditionFailed(node_id.clone()));
        }
        let old_siblings = self.ordered_sibling_ids(current_parent.as_ref());
        let current_position = old_siblings
            .iter()
            .position(|id| id == node_id)
            .ok_or_else(|| CommandApplyError::SiblingPreconditionFailed(node_id.clone()))?;
        if old_siblings.get(current_position + 1) != previous_before_node_id {
            return Err(CommandApplyError::SiblingPreconditionFailed(node_id.clone()));
        }
        if let Some(parent_id) = parent_id {
            self.require_active(parent_id)
                .map_err(|_| CommandApplyError::InvalidParent(parent_id.clone()))?;
        }
        if let Some(before_id) = before_node_id {
            let before = self
                .node(before_id)
                .ok_or_else(|| CommandApplyError::InvalidBeforeNode(before_id.clone()))?;
            if before.parent_id.as_ref() != parent_id || self.effectively_deleted(before_id) {
                return Err(CommandApplyError::InvalidBeforeNode(before_id.clone()));
            }
        }

        let mut target_siblings = self
            .ordered_sibling_ids(parent_id)
            .into_iter()
            .filter(|id| id != node_id)
            .collect::<Vec<_>>();
        let insert_at = before_node_id.map_or(Ok(target_siblings.len()), |before_id| {
            target_siblings
                .iter()
                .position(|id| id == before_id)
                .ok_or_else(|| CommandApplyError::InvalidBeforeNode(before_id.clone()))
        })?;
        target_siblings.insert(insert_at, node_id.clone());

        if current_parent.as_ref() != parent_id {
            let remaining_old =
                old_siblings.into_iter().filter(|id| id != node_id).collect::<Vec<_>>();
            self.assign_sibling_order(&remaining_old)?;
        }
        self.nodes
            .iter_mut()
            .find(|node| node.id == *node_id)
            .ok_or_else(|| CommandApplyError::MissingNode(node_id.clone()))?
            .parent_id
            .clone_from(&parent_id.cloned());
        self.assign_sibling_order(&target_siblings)
    }

    fn apply_delete_node(&mut self, node_id: &ObjectId) -> Result<(), CommandApplyError> {
        self.active_node_mut(node_id)?.deleted = true;
        Ok(())
    }

    fn apply_restore_node(&mut self, node_id: &ObjectId) -> Result<(), CommandApplyError> {
        let node = self
            .nodes
            .iter_mut()
            .find(|node| node.id == *node_id)
            .ok_or_else(|| CommandApplyError::MissingNode(node_id.clone()))?;
        if !node.deleted {
            return Err(CommandApplyError::NodeIsNotDeleted(node_id.clone()));
        }
        node.deleted = false;
        Ok(())
    }

    fn active_node_mut(&mut self, node_id: &ObjectId) -> Result<&mut Node, CommandApplyError> {
        self.require_active(node_id)?;
        self.nodes
            .iter_mut()
            .find(|node| node.id == *node_id)
            .ok_or_else(|| CommandApplyError::MissingNode(node_id.clone()))
    }

    fn require_active(&self, id: &ObjectId) -> Result<(), CommandApplyError> {
        if self.node(id).is_none() {
            return Err(CommandApplyError::MissingNode(id.clone()));
        }
        if self.effectively_deleted(id) {
            Err(CommandApplyError::DeletedNode(id.clone()))
        } else {
            Ok(())
        }
    }

    fn effectively_deleted(&self, id: &ObjectId) -> bool {
        let mut current = Some(id);
        while let Some(current_id) = current {
            let Some(node) = self.node(current_id) else {
                return true;
            };
            if node.deleted {
                return true;
            }
            current = node.parent_id.as_ref();
        }
        false
    }

    fn ordered_sibling_ids(&self, parent_id: Option<&ObjectId>) -> Vec<ObjectId> {
        let mut siblings = self
            .nodes
            .iter()
            .filter(|node| {
                node.parent_id.as_ref() == parent_id && !self.effectively_deleted(&node.id)
            })
            .collect::<Vec<_>>();
        siblings.sort_by(|left, right| {
            left.order.cmp(&right.order).then_with(|| left.id.cmp(&right.id))
        });
        siblings.into_iter().map(|node| node.id.clone()).collect()
    }

    fn assign_sibling_order(&mut self, ids: &[ObjectId]) -> Result<(), CommandApplyError> {
        for (index, id) in ids.iter().enumerate() {
            let index = i64::try_from(index).map_err(|_| CommandApplyError::OrderExhausted)?;
            let order = index.checked_mul(1024).ok_or(CommandApplyError::OrderExhausted)?;
            self.nodes
                .iter_mut()
                .find(|node| &node.id == id)
                .ok_or_else(|| CommandApplyError::MissingNode(id.clone()))?
                .order = order;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process::{Command, Stdio},
        sync::atomic::{AtomicU64, Ordering},
        thread,
        time::{Duration, Instant},
    };

    use super::*;
    use crate::{
        SAMPLE_PROJECT,
        asset_store::LocalAssetStore,
        history::RecoverySource,
        permissions::{Capability, CapabilityGrant, PolicySnapshot, principal},
    };

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

    struct TestDb {
        directory: tempfile::TempDir,
        path: PathBuf,
    }

    impl TestDb {
        fn new() -> Self {
            let directory = tempfile::tempdir().expect("temporary directory");
            let path = directory.path().join(format!(
                "commands-{}-{}.sqlite",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            Self { directory, path }
        }
    }

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

    fn set_property_envelope() -> CommandEnvelope {
        CommandEnvelope {
            operation_id: id(1),
            project_id: id(2),
            target_id: id(10),
            schema_version: 1,
            actor: Actor::user("user-1"),
            causal: CausalContext::new("desktop-1", 1, 1),
            command: CreativeCommand::SetProperty {
                node_id: id(10),
                property: "opacity".to_owned(),
                value: Value::from(0.5),
                previous_state: PropertyState::Present(Value::from(1.0)),
            },
        }
    }

    fn fixture_graph() -> ProjectGraph {
        ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture graph")
    }

    fn process_recovery_graph() -> ProjectGraph {
        ProjectGraph {
            schema_version: 1,
            project_id: id(1),
            name: "Crash recovery fixture".to_owned(),
            nodes: vec![Node {
                id: id(10),
                node_type: "design.document".to_owned(),
                parent_id: None,
                references: Vec::new(),
                name: Some("Recovery document".to_owned()),
                asset_ref: None,
                properties: BTreeMap::new(),
                order: 0,
                deleted: false,
                extensions: BTreeMap::new(),
            }],
            asset_refs: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    fn process_recovery_envelope() -> CommandEnvelope {
        fixture_envelope(
            90,
            1,
            10,
            CreativeCommand::SetProperty {
                node_id: id(10),
                property: "fill".to_owned(),
                value: Value::from("#2b8090"),
                previous_state: PropertyState::Absent,
            },
        )
    }

    fn fixture_envelope(
        operation: u16,
        sequence: u64,
        target: u16,
        command: CreativeCommand,
    ) -> CommandEnvelope {
        CommandEnvelope {
            operation_id: id(operation),
            project_id: id(1),
            target_id: id(target),
            schema_version: 1,
            actor: Actor::user("user-1"),
            causal: CausalContext::new("desktop-1", sequence, sequence),
            command,
        }
    }

    fn authorized_policy() -> PolicySnapshot {
        let actor = Actor::user("user-1");
        let mut policy = PolicySnapshot::new(1);
        policy.grant(CapabilityGrant::project(
            principal(actor.kind, actor.id),
            "tenant-a",
            id(1),
            Capability::DesignWrite,
        ));
        policy
    }

    #[test]
    fn validates_and_serializes_a_typed_envelope() {
        let envelope = set_property_envelope();
        envelope.validate().expect("command should validate");
        let encoded = serde_json::to_string(&envelope).expect("command should serialize");
        assert!(encoded.contains("operation_id"));
        assert!(encoded.contains("set_property"));
        assert!(encoded.contains("client_sequence"));
    }

    #[test]
    fn rejects_target_mismatch_and_invalid_ordering() {
        let mut envelope = set_property_envelope();
        envelope.target_id = id(11);
        assert!(matches!(envelope.validate(), Err(CommandError::TargetMismatch { .. })));

        let mut envelope = set_property_envelope();
        envelope.causal.client_sequence = 0;
        assert_eq!(envelope.validate(), Err(CommandError::InvalidOrdering));

        let mut envelope = set_property_envelope();
        envelope.causal.lamport = u64::MAX;
        assert_eq!(envelope.validate(), Err(CommandError::OrderingOutOfRange));
    }

    #[test]
    fn inverse_preserves_previous_state_and_delete_restore_pair() {
        let command = set_property_envelope().command;
        let inverse = command.inverse().expect("previous value makes undo possible");
        assert!(matches!(
            inverse,
            CreativeCommand::SetProperty {
                value: Value::Number(_),
                previous_state: PropertyState::Present(Value::Number(_)),
                ..
            }
        ));

        let delete = CreativeCommand::DeleteNode { node_id: id(10) };
        assert_eq!(
            delete.inverse().expect("delete is reversible"),
            CreativeCommand::RestoreNode { node_id: id(10) }
        );
    }

    #[test]
    fn property_preconditions_and_undo_preserve_explicit_json_null() {
        let mut graph = fixture_graph();
        let frame = id(11);
        let set_null = fixture_envelope(
            7,
            7,
            11,
            CreativeCommand::SetProperty {
                node_id: frame.clone(),
                property: "nullable".to_owned(),
                value: Value::Null,
                previous_state: PropertyState::Absent,
            },
        );
        graph.apply_envelope(&set_null).expect("set property to null");

        let replace_null = fixture_envelope(
            8,
            8,
            11,
            CreativeCommand::SetProperty {
                node_id: frame.clone(),
                property: "nullable".to_owned(),
                value: Value::String("present".to_owned()),
                previous_state: PropertyState::Present(Value::Null),
            },
        );
        let encoded = serde_json::to_string(&replace_null).expect("serialize operation");
        let decoded: CommandEnvelope =
            serde_json::from_str(&encoded).expect("decode operation with null precondition");
        assert_eq!(decoded, replace_null);
        graph.apply_envelope(&decoded).expect("replace null property");

        let undo = fixture_envelope(9, 9, 11, decoded.command.inverse().expect("undo update"));
        graph.apply_envelope(&undo).expect("restore explicit null");
        assert_eq!(graph.node(&frame).unwrap().properties["nullable"], Value::Null);
    }

    #[test]
    fn duplicate_delivery_is_observed_without_double_recording() {
        let envelope = set_property_envelope();
        let mut ledger = OperationLedger::default();
        assert_eq!(
            ledger.record(&envelope).expect("first delivery is valid"),
            RecordOutcome::FirstSeen
        );
        assert_eq!(ledger.record(&envelope).expect("retry is valid"), RecordOutcome::Duplicate);
    }

    #[test]
    fn command_reducer_applies_compensation_move_and_soft_delete() {
        let mut graph = fixture_graph();
        let frame = id(11);
        let set = fixture_envelope(
            1,
            1,
            11,
            CreativeCommand::SetProperty {
                node_id: frame.clone(),
                property: "opacity".to_owned(),
                value: Value::from(0.5),
                previous_state: PropertyState::Absent,
            },
        );
        graph.apply_envelope(&set).expect("apply property change");
        assert_eq!(graph.node(&frame).unwrap().properties["opacity"], Value::from(0.5));

        let inverse = set.command.inverse().expect("previous absence is reversible");
        let undo = fixture_envelope(2, 2, 11, inverse);
        graph.apply_envelope(&undo).expect("apply compensating operation");
        assert!(!graph.node(&frame).unwrap().properties.contains_key("opacity"));

        let move_frame = fixture_envelope(
            3,
            3,
            11,
            CreativeCommand::MoveNode {
                node_id: frame.clone(),
                parent_id: None,
                before_node_id: Some(id(10)),
                previous_parent_id: Some(id(10)),
                previous_before_node_id: None,
            },
        );
        graph.apply_envelope(&move_frame).expect("move frame to root");
        assert_eq!(graph.node(&frame).unwrap().parent_id, None);
        assert!(graph.node(&frame).unwrap().order < graph.node(&id(10)).unwrap().order);

        let delete = fixture_envelope(4, 4, 10, CreativeCommand::DeleteNode { node_id: id(10) });
        graph.apply_envelope(&delete).expect("soft delete document");
        assert!(graph.effectively_deleted(&id(10)));
        let restore = fixture_envelope(5, 5, 10, CreativeCommand::RestoreNode { node_id: id(10) });
        graph.apply_envelope(&restore).expect("restore document");
        assert!(!graph.effectively_deleted(&id(10)));
    }

    #[test]
    fn failed_graph_application_does_not_partially_mutate_state() {
        let mut graph = fixture_graph();
        let before = graph.clone();
        let move_document_into_its_descendant = fixture_envelope(
            6,
            6,
            10,
            CreativeCommand::MoveNode {
                node_id: id(10),
                parent_id: Some(id(11)),
                before_node_id: None,
                previous_parent_id: None,
                previous_before_node_id: Some(id(12)),
            },
        );

        assert!(matches!(
            graph.apply_envelope(&move_document_into_its_descendant),
            Err(CommandApplyError::Graph(GraphError::ParentCycle(_)))
        ));
        assert_eq!(graph, before);
    }

    #[test]
    fn local_command_boundary_commits_then_replays_and_deduplicates() {
        let database = TestDb::new();
        let mut journal = LocalJournal::open(&database.path).expect("open journal");
        let base = fixture_graph();
        let mut graph = base.clone();
        let envelope = fixture_envelope(
            100,
            1,
            11,
            CreativeCommand::SetProperty {
                node_id: id(11),
                property: "opacity".to_owned(),
                value: Value::from(0.5),
                previous_state: PropertyState::Absent,
            },
        );
        let resolver = FixedCapabilityResolver(Capability::DesignWrite);
        let policy = authorized_policy();
        let receipt = CommandService::commit_local(
            &mut graph,
            &mut journal,
            &envelope,
            "tenant-a",
            &resolver,
            &policy,
        )
        .expect("commit command");
        assert_eq!(receipt.append, AppendOutcome::Committed);
        assert_eq!(graph.node(&id(11)).unwrap().properties["opacity"], Value::from(0.5));

        let duplicate = CommandService::commit_local(
            &mut graph,
            &mut journal,
            &envelope,
            "tenant-a",
            &resolver,
            &policy,
        )
        .expect("duplicate command");
        assert_eq!(duplicate.append, AppendOutcome::AlreadyCommitted);

        let mut recovered = base;
        assert_eq!(
            CommandService::replay_from(&mut recovered, &journal, &id(1), None)
                .expect("replay operations"),
            1
        );
        assert_eq!(recovered, graph);
    }

    #[test]
    fn killed_process_recovers_local_command_graph_and_pending_operation() {
        const DATABASE_ENV: &str = "CHROMAKOPIA_COMMAND_CRASH_TEST_DATABASE";
        const READY_ENV: &str = "CHROMAKOPIA_COMMAND_CRASH_TEST_READY";
        const ASSET_SCOPE: &str = "command-crash-test";

        if let Some(database_path) = std::env::var_os(DATABASE_ENV) {
            let ready_path = std::env::var_os(READY_ENV).expect("child ready marker path");
            let mut journal = LocalJournal::open(database_path).expect("child opens journal");
            let mut graph = process_recovery_graph();
            let operation = process_recovery_envelope();
            let resolver = FixedCapabilityResolver(Capability::DesignWrite);
            let policy = authorized_policy();
            let receipt = CommandService::commit_local(
                &mut graph,
                &mut journal,
                &operation,
                "tenant-a",
                &resolver,
                &policy,
            )
            .expect("child commits through the authorized command API");
            assert_eq!(receipt.append, AppendOutcome::Committed);
            assert_eq!(graph.node(&id(10)).unwrap().properties["fill"], Value::from("#2b8090"));
            fs::write(ready_path, b"local command commit returned")
                .expect("publish child readiness");

            loop {
                thread::park();
            }
        }

        let database = TestDb::new();
        let base = process_recovery_graph();
        let assets = LocalAssetStore::open(database.directory.path(), ASSET_SCOPE)
            .expect("open empty test asset store");
        {
            let mut journal = LocalJournal::open(&database.path).expect("open journal");
            journal
                .create_snapshot(&id(91), &base, &assets)
                .expect("checkpoint base graph before child process");
        }

        let ready_path = database.directory.path().join("local-command-committed");
        let mut child = Command::new(std::env::current_exe().expect("test executable path"))
            .arg("--exact")
            .arg(
                "command::tests::killed_process_recovers_local_command_graph_and_pending_operation",
            )
            .env(DATABASE_ENV, &database.path)
            .env(READY_ENV, &ready_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start command child process");

        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready_path.exists() {
            if let Some(status) = child.try_wait().expect("check child process") {
                panic!("child exited before committing the command: {status}");
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("child did not report its command commit within 10 seconds");
            }
            thread::sleep(Duration::from_millis(10));
        }

        child.kill().expect("force-terminate child after local command commit");
        let status = child.wait().expect("wait for terminated child");
        assert!(!status.success(), "forced child termination should be abnormal");

        let mut journal = LocalJournal::open(&database.path).expect("reopen after process kill");
        let recovery = journal
            .recover_project(&base.project_id, &assets)
            .expect("recover committed graph after process kill");
        assert_eq!(recovery.source, RecoverySource::MaterializedView);
        assert_eq!(
            recovery.graph.node(&id(10)).unwrap().properties["fill"],
            Value::from("#2b8090")
        );

        let operation = process_recovery_envelope();
        let recovered_entry = journal
            .read(&operation.operation_id)
            .expect("read recovered command")
            .expect("command survives process termination");
        assert_eq!(recovered_entry.envelope, operation);
        assert_eq!(recovered_entry.replication, crate::journal::ReplicationState::Pending);
        assert!(journal.health().expect("journal health").meets_configured_durability());

        let resolver = FixedCapabilityResolver(Capability::DesignWrite);
        let policy = authorized_policy();
        let mut recovered_graph = recovery.graph;
        let retry = CommandService::commit_local(
            &mut recovered_graph,
            &mut journal,
            &operation,
            "tenant-a",
            &resolver,
            &policy,
        )
        .expect("retry command after recovery");
        assert_eq!(retry.append, AppendOutcome::AlreadyCommitted);
        assert_eq!(
            recovered_graph.node(&id(10)).unwrap().properties["fill"],
            Value::from("#2b8090")
        );
    }

    #[test]
    fn denied_command_changes_neither_graph_nor_journal() {
        let database = TestDb::new();
        let mut journal = LocalJournal::open(&database.path).expect("open journal");
        let mut graph = fixture_graph();
        let before = graph.clone();
        let envelope = fixture_envelope(
            100,
            1,
            11,
            CreativeCommand::SetProperty {
                node_id: id(11),
                property: "opacity".to_owned(),
                value: Value::from(0.5),
                previous_state: PropertyState::Absent,
            },
        );
        let resolver = FixedCapabilityResolver(Capability::DesignWrite);
        let policy = PolicySnapshot::new(1);
        assert!(matches!(
            CommandService::commit_local(
                &mut graph,
                &mut journal,
                &envelope,
                "tenant-a",
                &resolver,
                &policy
            ),
            Err(CommandServiceError::AuthorizationRejected(_))
        ));
        assert_eq!(graph, before);
        assert!(journal.read(&envelope.operation_id).expect("read journal").is_none());
    }

    #[test]
    fn materialization_failure_rolls_back_the_journaled_operation() {
        let database = TestDb::new();
        let mut journal = LocalJournal::open(&database.path).expect("open journal");
        journal
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_materialized_state BEFORE INSERT ON materialized_projects
                 BEGIN SELECT RAISE(ABORT, 'injected materialization failure'); END;",
            )
            .expect("install failure injector");
        let mut graph = fixture_graph();
        let before = graph.clone();
        let envelope = fixture_envelope(
            101,
            1,
            11,
            CreativeCommand::SetProperty {
                node_id: id(11),
                property: "opacity".to_owned(),
                value: Value::from(0.5),
                previous_state: PropertyState::Absent,
            },
        );
        let resolver = FixedCapabilityResolver(Capability::DesignWrite);
        let policy = authorized_policy();

        assert!(matches!(
            CommandService::commit_local(
                &mut graph,
                &mut journal,
                &envelope,
                "tenant-a",
                &resolver,
                &policy,
            ),
            Err(CommandServiceError::Journal(_))
        ));
        assert_eq!(graph, before);
        assert!(journal.read(&envelope.operation_id).expect("read journal").is_none());
    }
}
