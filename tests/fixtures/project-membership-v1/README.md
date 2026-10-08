# Project membership v1 fixture

`active-user.json` is a synthetic, test-only current-membership record. It contains no real account or tenant identifiers. The shape is validated against [`project-membership-v1.schema.json`](../../../schemas/project-membership-v1.schema.json).

Membership is a tenant/project relationship, not a capability grant. This fixture does not establish an authoritative membership service, role mapping, invitation flow, or mutation audit log; those remain open contract and service decisions.
