use jsonschema::{Validator, validator_for};
use platform_runtime::graph::ProjectGraph;
use serde_json::{Value, json};

const PROJECT_SCHEMA: &str = include_str!("../../../schemas/project-v1.schema.json");
const SAMPLE_PROJECT: &str =
    include_str!("../../../tests/fixtures/synthetic-campaign/project.json");

fn project_validator() -> Validator {
    let schema: Value = serde_json::from_str(PROJECT_SCHEMA).expect("schema JSON should parse");
    validator_for(&schema).expect("project schema should be a valid JSON Schema")
}

#[test]
fn schema_accepts_the_checked_in_synthetic_campaign_fixture() {
    let validator = project_validator();
    let project: Value = serde_json::from_str(SAMPLE_PROJECT).expect("fixture JSON should parse");

    assert!(validator.is_valid(&project));
    ProjectGraph::from_json(SAMPLE_PROJECT).expect("runtime graph validator should accept fixture");
}

#[test]
fn schema_rejects_invalid_graph_contracts_and_unrecognized_project_fields() {
    let validator = project_validator();
    let fixture: Value = serde_json::from_str(SAMPLE_PROJECT).expect("fixture JSON should parse");

    let mut invalid_identity = fixture.clone();
    invalid_identity["project_id"] = json!("not-a-uuid");
    assert!(!validator.is_valid(&invalid_identity));

    let mut blank_name = fixture.clone();
    blank_name["name"] = json!(" \t ");
    assert!(!validator.is_valid(&blank_name));

    let mut unknown_project_field = fixture;
    unknown_project_field["future_top_level"] = json!(true);
    assert!(!validator.is_valid(&unknown_project_field));
    assert!(
        ProjectGraph::from_json(
            &serde_json::to_string(&unknown_project_field).expect("project JSON should encode")
        )
        .is_err()
    );
}

#[test]
fn schema_allows_unknown_node_fields_for_forward_compatible_round_tripping() {
    let validator = project_validator();
    let mut project: Value =
        serde_json::from_str(SAMPLE_PROJECT).expect("fixture JSON should parse");
    project["nodes"][0]["future_property"] = json!({"retained": true});
    let encoded = serde_json::to_string(&project).expect("project JSON should encode");

    assert!(validator.is_valid(&project));
    let graph =
        ProjectGraph::from_json(&encoded).expect("runtime should preserve unknown node data");
    let round_tripped = serde_json::to_value(graph).expect("graph should serialize");
    assert_eq!(round_tripped["nodes"][0]["future_property"]["retained"], true);
}

#[test]
fn schema_accepts_a_large_generated_graph() {
    let validator = project_validator();
    let mut project: Value =
        serde_json::from_str(SAMPLE_PROJECT).expect("fixture JSON should parse");
    let nodes = project["nodes"].as_array_mut().expect("fixture nodes should be an array");

    nodes.extend((0..2_048).map(|number| {
        json!({
            "id": format!("00000000-0000-4000-8000-{number:012x}"),
            "type": "media.image",
            "name": format!("Generated image {number}")
        })
    }));

    assert!(validator.is_valid(&project));
}
