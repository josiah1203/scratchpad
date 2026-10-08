# Replay fixture v1

This fixture is generated test data based on the synthetic campaign graph. It contains no user or
third-party content; provenance and usage are synthetic/test-only.

`operations.jsonl` is a four-operation, single-client sequence: set frame opacity, add image alt
text, create a nested shape, then set that shape's x position. `expected-project.json` is the complete
graph expected after replaying those operations against
[`../synthetic-campaign/project.json`](../synthetic-campaign/project.json). The integration test
validates each envelope against command schema v1, commits the sequence through the authorized local
command boundary using test-only project grants, and checks deterministic reconstruction from the
base graph.
