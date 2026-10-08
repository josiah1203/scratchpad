use jsonschema::{Validator, validator_for};
use serde_json::{Value, json};

const MEMBERSHIP_SCHEMA: &str = include_str!("../../../schemas/project-membership-v1.schema.json");
const ACTIVE_MEMBERSHIP: &str =
    include_str!("../../../tests/fixtures/project-membership-v1/active-user.json");

fn membership_validator() -> Validator {
    let schema: Value = serde_json::from_str(MEMBERSHIP_SCHEMA).expect("schema JSON");
    validator_for(&schema).expect("membership schema should compile as Draft 2020-12")
}

#[test]
fn schema_accepts_the_synthetic_active_membership_fixture() {
    let validator = membership_validator();
    let membership: Value = serde_json::from_str(ACTIVE_MEMBERSHIP).expect("fixture JSON");

    assert!(validator.is_valid(&membership));
}

#[test]
fn schema_rejects_invalid_membership_and_embedded_capability_grants() {
    let validator = membership_validator();
    let fixture: Value = serde_json::from_str(ACTIVE_MEMBERSHIP).expect("fixture JSON");

    let mut invalid_project_id = fixture.clone();
    invalid_project_id["project_id"] = json!("not-a-uuid");
    assert!(!validator.is_valid(&invalid_project_id));

    let mut blank_principal = fixture.clone();
    blank_principal["principal"]["id"] = json!(" \t ");
    assert!(!validator.is_valid(&blank_principal));

    let mut embedded_capability = fixture.clone();
    embedded_capability["capabilities"] = json!(["project.read"]);
    assert!(!validator.is_valid(&embedded_capability));

    let mut unsupported_status = fixture;
    unsupported_status["status"] = json!("invited");
    assert!(!validator.is_valid(&unsupported_status));
}
