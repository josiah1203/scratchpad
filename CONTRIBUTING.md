# Contributing

This repository follows the execution and quality rules in [`AGENTS.md`](AGENTS.md).

## Before changing code

1. Read the root guide, [`goals/README.md`](goals/README.md), the selected goal, and its dependencies.
2. Inspect the current repository and working-tree status.
3. Write down the concrete behavior, affected contracts, acceptance criteria, and verification commands.
4. Check whether the change requires an ADR, migration note, fixture, permission review, or compatibility test.

## During implementation

- Keep the smallest cohesive change that advances the selected goal.
- Do not bypass the shared project graph, command, asset, history, or permission boundaries.
- Do not put creative source bytes in command payloads or log private creative content.
- Preserve existing user work and avoid unrelated formatting or generated artifacts.
- Use role-based ownership in [`docs/OWNERS.md`](docs/OWNERS.md) when coordinating shared contracts.

## Verification and handoff

Run the narrowest meaningful checks first, then the broader checks required by the touched goal. A handoff must include:

- behavior and contracts changed;
- files or areas changed;
- exact commands and outcomes;
- known limitations and unresolved ADRs;
- migration, rollout, or follow-up decisions.

