//! Versioned project graph primitives and validation.
//!
//! The graph is deliberately independent of UI and storage. It validates the
//! portable JSON representation and exposes only the relationships needed for
//! incremental loading. Commands, persistence, and sync build on this contract
//! in later foundation goals.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

const CURRENT_SCHEMA_VERSION: u32 = 1;

/// A stable UUID identifying a project object.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ObjectId(String);

impl ObjectId {
    /// Creates an ID without validation. Use [`ProjectGraph::from_json`] or
    /// [`ObjectId::validate`] at a contract boundary.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the underlying UUID string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn validate(&self, context: &str) -> Result<(), GraphError> {
        if is_uuid(&self.0) {
            Ok(())
        } else {
            Err(GraphError::InvalidObjectId { context: context.to_owned(), value: self.0.clone() })
        }
    }
}

/// A content-addressed asset identity.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct AssetId(String);

impl AssetId {
    /// Creates an asset ID without validation.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the content-addressed identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(&self) -> Result<(), GraphError> {
        let digest = self.0.strip_prefix("sha256:").unwrap_or_default();
        if digest.len() == 64
            && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(())
        } else {
            Err(GraphError::InvalidAssetId(self.0.clone()))
        }
    }
}

/// The canonical portable project graph envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGraph {
    pub schema_version: u32,
    pub project_id: ObjectId,
    pub name: String,
    pub nodes: Vec<Node>,
    pub asset_refs: Vec<AssetRef>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

/// A typed graph node. Unknown node fields are retained in `extensions` so a
/// reader can round-trip newer node data without pretending to understand it.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Node {
    pub id: ObjectId,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<ObjectId>,
    #[serde(default)]
    pub references: Vec<ObjectId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_ref: Option<AssetId>,
    #[serde(default)]
    pub properties: BTreeMap<String, Value>,
    #[serde(default)]
    pub order: i64,
    #[serde(default)]
    pub deleted: bool,
    #[serde(flatten)]
    pub extensions: BTreeMap<String, Value>,
}

/// Metadata for an immutable source or derived asset.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssetRef {
    pub id: AssetId,
    pub kind: String,
    pub byte_length: u64,
    pub provenance: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rights: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform: Option<Value>,
}

/// Validation failures for the graph contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphError {
    InvalidJson(String),
    UnsupportedSchema(u32),
    EmptyProjectName,
    InvalidObjectId { context: String, value: String },
    InvalidAssetId(String),
    EmptyAssetKind(AssetId),
    EmptyAssetProvenance(AssetId),
    EmptyNodeType(ObjectId),
    DuplicateNodeId(ObjectId),
    DuplicateAssetId(AssetId),
    EmptyPropertyName(ObjectId),
    MissingNodeReference { from: ObjectId, to: ObjectId },
    MissingAssetReference { node: ObjectId, asset: AssetId },
    ParentCycle(Vec<ObjectId>),
    UnknownNode(ObjectId),
}

impl fmt::Display for GraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(message) => write!(formatter, "invalid project JSON: {message}"),
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported project schema version {version}")
            }
            Self::EmptyProjectName => formatter.write_str("project name must not be empty"),
            Self::InvalidObjectId { context, value } => {
                write!(formatter, "invalid object ID for {context}: {value}")
            }
            Self::InvalidAssetId(value) => write!(formatter, "invalid asset ID: {value}"),
            Self::EmptyAssetKind(id) => {
                write!(formatter, "asset {} has an empty kind", id.as_str())
            }
            Self::EmptyAssetProvenance(id) => {
                write!(formatter, "asset {} has empty provenance", id.as_str())
            }
            Self::EmptyNodeType(id) => write!(formatter, "node {} has an empty type", id.as_str()),
            Self::DuplicateNodeId(id) => write!(formatter, "duplicate node ID: {}", id.as_str()),
            Self::DuplicateAssetId(id) => write!(formatter, "duplicate asset ID: {}", id.as_str()),
            Self::EmptyPropertyName(id) => {
                write!(formatter, "node {} has an empty property name", id.as_str())
            }
            Self::MissingNodeReference { from, to } => {
                write!(formatter, "node {} references missing node {}", from.as_str(), to.as_str())
            }
            Self::MissingAssetReference { node, asset } => write!(
                formatter,
                "node {} references missing asset {}",
                node.as_str(),
                asset.as_str()
            ),
            Self::ParentCycle(path) => write!(formatter, "parent cycle detected: {path:?}"),
            Self::UnknownNode(id) => write!(formatter, "unknown node ID: {}", id.as_str()),
        }
    }
}

impl std::error::Error for GraphError {}

impl ProjectGraph {
    /// Parses and validates a project graph from its portable JSON form.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] when JSON parsing or any graph invariant fails.
    pub fn from_json(json: &str) -> Result<Self, GraphError> {
        let graph = serde_json::from_str::<Self>(json)
            .map_err(|error| GraphError::InvalidJson(error.to_string()))?;
        graph.validate()?;
        Ok(graph)
    }

    /// Validates graph invariants without changing the graph.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] when an identity, reference, asset, or hierarchy
    /// invariant is violated.
    pub fn validate(&self) -> Result<(), GraphError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(GraphError::UnsupportedSchema(self.schema_version));
        }
        if self.name.trim().is_empty() {
            return Err(GraphError::EmptyProjectName);
        }
        self.project_id.validate("project_id")?;

        let mut nodes = HashMap::with_capacity(self.nodes.len());
        for node in &self.nodes {
            node.id.validate("node.id")?;
            if node.node_type.trim().is_empty() {
                return Err(GraphError::EmptyNodeType(node.id.clone()));
            }
            if node.properties.keys().any(|key| key.trim().is_empty()) {
                return Err(GraphError::EmptyPropertyName(node.id.clone()));
            }
            if nodes.insert(node.id.clone(), node).is_some() {
                return Err(GraphError::DuplicateNodeId(node.id.clone()));
            }
        }

        let mut assets = HashMap::with_capacity(self.asset_refs.len());
        for asset in &self.asset_refs {
            asset.id.validate()?;
            if asset.kind.trim().is_empty() {
                return Err(GraphError::EmptyAssetKind(asset.id.clone()));
            }
            if asset.provenance.trim().is_empty() {
                return Err(GraphError::EmptyAssetProvenance(asset.id.clone()));
            }
            if assets.insert(asset.id.clone(), asset).is_some() {
                return Err(GraphError::DuplicateAssetId(asset.id.clone()));
            }
        }

        for node in &self.nodes {
            if let Some(parent_id) = &node.parent_id {
                Self::require_node(&nodes, node.id.clone(), parent_id.clone())?;
            }
            for reference in &node.references {
                Self::require_node(&nodes, node.id.clone(), reference.clone())?;
            }
            if let Some(asset_id) = &node.asset_ref
                && !assets.contains_key(asset_id)
            {
                return Err(GraphError::MissingAssetReference {
                    node: node.id.clone(),
                    asset: asset_id.clone(),
                });
            }
        }

        Self::validate_parent_cycles(&nodes)?;
        Ok(())
    }

    /// Returns all nodes in source order.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter()
    }

    /// Returns one node by stable ID.
    #[must_use]
    pub fn node(&self, id: &ObjectId) -> Option<&Node> {
        self.nodes.iter().find(|node| &node.id == id)
    }

    /// Loads roots plus their parents and explicit references for incremental clients.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::UnknownNode`] when a requested root or dependency
    /// is not present in the graph.
    pub fn load_subgraph(&self, roots: &[ObjectId]) -> Result<Vec<&Node>, GraphError> {
        let nodes: HashMap<ObjectId, &Node> =
            self.nodes.iter().map(|node| (node.id.clone(), node)).collect();
        let mut selected = HashSet::new();
        let mut pending = roots.to_vec();

        while let Some(id) = pending.pop() {
            if !selected.insert(id.clone()) {
                continue;
            }
            let node = nodes.get(&id).ok_or_else(|| GraphError::UnknownNode(id.clone()))?;
            if let Some(parent_id) = &node.parent_id {
                pending.push(parent_id.clone());
            }
            pending.extend(node.references.iter().cloned());
        }

        Ok(self.nodes.iter().filter(|node| selected.contains(&node.id)).collect())
    }

    fn require_node(
        nodes: &HashMap<ObjectId, &Node>,
        from: ObjectId,
        to: ObjectId,
    ) -> Result<(), GraphError> {
        if nodes.contains_key(&to) {
            Ok(())
        } else {
            Err(GraphError::MissingNodeReference { from, to })
        }
    }

    fn validate_parent_cycles(nodes: &HashMap<ObjectId, &Node>) -> Result<(), GraphError> {
        let mut completed = HashSet::with_capacity(nodes.len());
        for id in nodes.keys() {
            let mut path = Vec::new();
            let mut positions = HashMap::new();
            let mut current = Some(id.clone());
            while let Some(current_id) = current {
                if completed.contains(&current_id) {
                    break;
                }
                if let Some(position) = positions.get(&current_id) {
                    return Err(GraphError::ParentCycle(path[*position..].to_vec()));
                }
                positions.insert(current_id.clone(), path.len());
                path.push(current_id.clone());
                current = nodes.get(&current_id).and_then(|node| node.parent_id.clone());
            }
            completed.extend(path);
        }
        Ok(())
    }
}

fn is_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            matches!(index, 8 | 13 | 18 | 23) && byte == b'-'
                || !matches!(index, 8 | 13 | 18 | 23) && byte.is_ascii_hexdigit()
        })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::SAMPLE_PROJECT;

    const PROJECT_SCHEMA: &str = include_str!("../../../schemas/project-v1.schema.json");
    const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000010";

    #[test]
    fn validates_fixture_and_loads_cross_workspace_dependencies() {
        let graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture should validate");
        let subgraph =
            graph.load_subgraph(&[ObjectId::new(DOCUMENT_ID)]).expect("document should exist");
        let types: BTreeSet<_> = subgraph.iter().map(|node| node.node_type.as_str()).collect();

        assert_eq!(subgraph.len(), 4);
        assert!(types.contains("design.document"));
        assert!(types.contains("design.frame"));
        assert!(types.contains("media.image"));
        assert!(types.contains("brand.style"));
    }

    #[test]
    fn preserves_unknown_node_fields_for_round_trip() {
        let json = r#"{
                "schema_version": 1,
                "project_id": "00000000-0000-4000-8000-000000000001",
                "name": "Future data",
                "nodes": [{
                    "id": "00000000-0000-4000-8000-000000000010",
                    "type": "design.document",
                    "future_property": {"kept": true}
                }],
                "asset_refs": []
            }"#;
        let graph = ProjectGraph::from_json(json).expect("unknown node data is compatible");
        assert_eq!(graph.nodes[0].extensions["future_property"]["kept"], true);
        let encoded = serde_json::to_string(&graph).expect("graph should serialize");
        assert!(encoded.contains("future_property"));
    }

    #[test]
    fn requires_the_asset_refs_envelope_field() {
        let json = format!(
            r#"{{
                "schema_version": 1,
                "project_id": "00000000-0000-4000-8000-000000000001",
                "name": "Missing assets field",
                "nodes": [{{"id": "{DOCUMENT_ID}", "type": "design.document"}}]
            }}"#
        );
        assert!(matches!(ProjectGraph::from_json(&json), Err(GraphError::InvalidJson(_))));
    }

    #[test]
    fn rejects_duplicate_ids_and_missing_references() {
        let duplicate = format!(
            r#"{{
                "schema_version": 1,
                "project_id": "00000000-0000-4000-8000-000000000001",
                "name": "Invalid",
                "nodes": [
                    {{"id": "{DOCUMENT_ID}", "type": "design.document"}},
                    {{"id": "{DOCUMENT_ID}", "type": "design.frame"}}
                ],
                "asset_refs": []
            }}"#
        );
        assert!(matches!(ProjectGraph::from_json(&duplicate), Err(GraphError::DuplicateNodeId(_))));

        let missing = format!(
            r#"{{
                "schema_version": 1,
                "project_id": "00000000-0000-4000-8000-000000000001",
                "name": "Invalid",
                "nodes": [{{
                    "id": "{DOCUMENT_ID}",
                    "type": "design.document",
                    "references": ["00000000-0000-4000-8000-000000000099"]
                }}],
                "asset_refs": []
            }}"#
        );
        assert!(matches!(
            ProjectGraph::from_json(&missing),
            Err(GraphError::MissingNodeReference { .. })
        ));
    }

    #[test]
    fn partial_loading_excludes_unrelated_nodes_in_a_large_graph() {
        let mut graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture should validate");
        let unrelated_count = 2_048;
        for number in 100..(100 + unrelated_count) {
            graph.nodes.push(Node {
                id: ObjectId::new(format!("00000000-0000-4000-8000-{number:012}")),
                node_type: "media.image".to_owned(),
                parent_id: None,
                references: Vec::new(),
                name: None,
                asset_ref: None,
                properties: BTreeMap::new(),
                order: 0,
                deleted: false,
                extensions: BTreeMap::new(),
            });
        }
        graph.validate().expect("large synthetic graph should validate");

        let loaded = graph
            .load_subgraph(&[ObjectId::new(DOCUMENT_ID)])
            .expect("document dependencies should load");

        assert_eq!(graph.nodes().count(), unrelated_count + 4);
        assert_eq!(loaded.len(), 4);
        assert!(
            loaded.iter().all(|node| node.id.as_str() != "00000000-0000-4000-8000-000000000100")
        );
    }

    #[test]
    fn rejects_parent_cycles_with_the_cycle_members() {
        let json = format!(
            r#"{{
                "schema_version": 1,
                "project_id": "00000000-0000-4000-8000-000000000001",
                "name": "Cycle",
                "nodes": [
                    {{"id": "{DOCUMENT_ID}", "type": "design.document", "parent_id": "00000000-0000-4000-8000-000000000011"}},
                    {{"id": "00000000-0000-4000-8000-000000000011", "type": "design.frame", "parent_id": "{DOCUMENT_ID}"}}
                ],
                "asset_refs": []
            }}"#
        );

        assert!(matches!(
            ProjectGraph::from_json(&json),
            Err(GraphError::ParentCycle(path)) if path.len() == 2
        ));
    }

    #[test]
    fn rejects_blank_labels_and_asset_metadata() {
        let mut graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture should validate");
        graph.name = " \t ".to_owned();
        assert_eq!(graph.validate(), Err(GraphError::EmptyProjectName));

        let mut graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture should validate");
        graph.nodes[0].node_type = "\n".to_owned();
        assert!(matches!(graph.validate(), Err(GraphError::EmptyNodeType(_))));

        let mut graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture should validate");
        graph.nodes[0].properties.insert(" \t".to_owned(), Value::Null);
        assert!(matches!(graph.validate(), Err(GraphError::EmptyPropertyName(_))));

        let mut graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture should validate");
        graph.asset_refs[0].kind = " \t ".to_owned();
        assert!(matches!(graph.validate(), Err(GraphError::EmptyAssetKind(_))));

        let mut graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture should validate");
        graph.asset_refs[0].provenance = "\n".to_owned();
        assert!(matches!(graph.validate(), Err(GraphError::EmptyAssetProvenance(_))));
    }

    #[test]
    fn published_schema_tracks_runtime_nonblank_constraints() {
        let schema: Value = serde_json::from_str(PROJECT_SCHEMA).expect("schema JSON should parse");

        assert_eq!(schema["$schema"], "https://json-schema.org/draft/2020-12/schema");
        assert_eq!(schema["properties"]["schema_version"]["const"], 1);
        assert_eq!(schema["properties"]["name"]["pattern"], r"\S");
        assert_eq!(schema["$defs"]["node"]["properties"]["type"]["pattern"], r"\S");
        assert_eq!(
            schema["$defs"]["node"]["properties"]["properties"]["propertyNames"]["pattern"],
            r"\S"
        );
        assert_eq!(schema["$defs"]["asset_ref"]["properties"]["kind"]["pattern"], r"\S");
        assert_eq!(schema["$defs"]["asset_ref"]["properties"]["provenance"]["pattern"], r"\S");
    }
}
