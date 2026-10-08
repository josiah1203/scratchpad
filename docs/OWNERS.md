# Ownership boundaries

These are role-based ownership boundaries for the initial repository. Hosting-specific usernames can be mapped to these roles when the remote repository is established.

| Area | Contract owner | Review boundary |
|---|---|---|
| `AGENTS.md`, `goals/`, repository workflow | Foundation lead | Changes must preserve the goal dependency and evidence rules. |
| `crates/platform-runtime/`, `schemas/` | Runtime/contract lead | Shared graph, command, history, asset, sync, and permission contracts have one owner at a time. |
| `tests/fixtures/`, `scripts/`, CI | Test/reliability lead | Fixtures must be synthetic or licensed; verification must remain reproducible. |
| `docs/adr/`, architecture and API docs | Architecture lead | Consequential decisions require evidence, alternatives, consequences, and revisit triggers. |
| Workspace modules | Workspace lead | Workspace code consumes stable runtime contracts and does not create parallel persistence or undo paths. |

When ownership changes, update this file in the same change as the contract handoff.

