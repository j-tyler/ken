# Validation Fixtures

Deterministic bind-validation fixtures for follow-up implementation.

Each fixture contains:
- `contract.json` — `ken_bind_schema_v1` contract model (each field includes `type`, `required`, `description`, `purpose`, `used_by_frames`, `source_guidance`)
- `bind.json` — caller payload
- `expected.json` — expected deterministic output

These fixtures are documentation-first and should become executable tests when runtime code is implemented.

Suggested scenario coverage includes: missing bind object, payload type mismatch, stable ordering, missing field metadata, and invalid type token (e.g., `type: Elephant`).
