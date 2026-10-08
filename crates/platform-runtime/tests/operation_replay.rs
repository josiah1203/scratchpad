use jsonschema::validator_for;
use platform_runtime::{
    SAMPLE_PROJECT,
    command::{ActorKind, CommandEnvelope, CommandService},
    graph::ProjectGraph,
    journal::{AppendOutcome, LocalJournal},
    permissions::{
        AuthorizationError, Capability, CapabilityGrant, CommandCapabilityResolver, PolicySnapshot,
        principal,
    },
};
use serde_json::Value;
use tempfile::tempdir;

const COMMAND_SCHEMA: &str = include_str!("../../../schemas/command-envelope-v1.schema.json");
const OPERATIONS: &str = include_str!("../../../tests/fixtures/replay-v1/operations.jsonl");
const EXPECTED_PROJECT: &str =
    include_str!("../../../tests/fixtures/replay-v1/expected-project.json");

struct FixtureCapabilityResolver;

impl CommandCapabilityResolver for FixtureCapabilityResolver {
    fn required_capability(
        &self,
        envelope: &CommandEnvelope,
    ) -> Result<Capability, AuthorizationError> {
        match envelope.target_id.as_str() {
            "00000000-0000-4000-8000-000000000012" => Ok(Capability::ImageWrite),
            "00000000-0000-4000-8000-000000000011" | "00000000-0000-4000-8000-000000000014" => {
                Ok(Capability::DesignWrite)
            }
            _ => Err(AuthorizationError::UnregisteredCommand),
        }
    }
}

#[test]
fn checked_in_operations_validate_and_replay_to_the_expected_graph() {
    let schema: Value = serde_json::from_str(COMMAND_SCHEMA).expect("command schema JSON");
    let validator = validator_for(&schema).expect("command envelope schema should compile");
    let base = ProjectGraph::from_json(SAMPLE_PROJECT).expect("synthetic campaign base graph");
    let expected =
        ProjectGraph::from_json(EXPECTED_PROJECT).expect("expected replay graph should validate");
    let directory = tempdir().expect("temporary journal directory");
    let mut journal =
        LocalJournal::open(directory.path().join("replay.sqlite")).expect("open test journal");
    let project_id = base.project_id.clone();
    let fixture_author = principal(ActorKind::User, "fixture-author");
    let mut policy = PolicySnapshot::new(1);
    policy.grant(CapabilityGrant::project(
        fixture_author.clone(),
        "synthetic-test-tenant",
        project_id.clone(),
        Capability::DesignWrite,
    ));
    policy.grant(CapabilityGrant::project(
        fixture_author,
        "synthetic-test-tenant",
        project_id.clone(),
        Capability::ImageWrite,
    ));
    let resolver = FixtureCapabilityResolver;
    let mut committed_graph = base.clone();
    let mut operation_count = 0_usize;
    let mut previous_operation_id = None;

    for (line_number, line) in OPERATIONS.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line).unwrap_or_else(|error| {
            panic!("operation line {} is not JSON: {error}", line_number + 1)
        });
        assert!(
            validator.is_valid(&value),
            "operation line {} must match command-envelope v1",
            line_number + 1
        );
        let envelope: CommandEnvelope = serde_json::from_value(value).unwrap_or_else(|error| {
            panic!("operation line {} is not a command: {error}", line_number + 1)
        });
        assert_eq!(
            envelope.causal.client_sequence,
            u64::try_from(operation_count + 1).expect("fixture sequence fits u64")
        );
        assert_eq!(envelope.causal.parent_operation_id, previous_operation_id);
        assert_eq!(
            CommandService::commit_local(
                &mut committed_graph,
                &mut journal,
                &envelope,
                "synthetic-test-tenant",
                &resolver,
                &policy,
            )
            .expect("fixture command should pass the authorized local boundary")
            .append,
            AppendOutcome::Committed
        );
        previous_operation_id = Some(envelope.operation_id.clone());
        operation_count += 1;
    }

    assert_eq!(operation_count, 4, "fixture sequence should remain explicit");
    assert_eq!(committed_graph, expected);
    let mut replayed = base.clone();
    assert_eq!(
        CommandService::replay_from(&mut replayed, &journal, &project_id, None)
            .expect("operations should replay"),
        operation_count
    );
    assert_eq!(replayed, expected);

    let mut repeated_replay = base;
    assert_eq!(
        CommandService::replay_from(&mut repeated_replay, &journal, &project_id, None)
            .expect("replay should be deterministic"),
        operation_count
    );
    assert_eq!(repeated_replay, expected);
}
