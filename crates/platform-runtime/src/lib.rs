#![forbid(unsafe_code)]

pub mod asset_store;
pub mod command;
pub mod graph;
pub mod history;
pub mod journal;
pub mod permissions;
pub mod portable;

/// Synthetic campaign fixture used by the baseline smoke test and downstream goals.
pub const SAMPLE_PROJECT: &str =
    include_str!("../../../tests/fixtures/synthetic-campaign/project.json");

/// Returns the checked-in synthetic project fixture.
#[must_use]
pub const fn sample_project() -> &'static str {
    SAMPLE_PROJECT
}

#[cfg(test)]
mod tests {
    use super::{SAMPLE_PROJECT, graph::ProjectGraph};

    #[test]
    fn sample_fixture_contains_shared_project_primitives() {
        for required_field in ["schema_version", "project_id", "nodes", "asset_refs"] {
            assert!(SAMPLE_PROJECT.contains(required_field), "missing {required_field}");
        }
    }

    #[test]
    fn sample_fixture_is_a_valid_graph() {
        let graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture should validate");
        assert_eq!(graph.nodes().count(), 4);
    }
}
