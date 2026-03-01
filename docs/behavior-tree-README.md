# Validation Behavior Tree – Configuration & UI Guide

## Summary

This document describes (1) the behavior tree configuration needed when compiling d-ship dApps, and (2) what is required to implement a UI for defining and saving behavior trees as JSON.

---

## 1. Behavior Tree Configuration for dApp Compilation

### Purpose

The behavior tree defines how input data is validated before creating on-chain entities. It is embedded in the d-app at build time and can express:

- **Direct validations** – Hard-coded checks (e.g. weight ≤ 30 kg, `weightUnit` in allowed set).
- **External validations** – Oracle calls and proof checks (MultiverseX oracles, ZK proofs).

### Format

- **Schema**: `schemas/behavior-tree.schema.json`
- **Structure**: Behavior3-style AST; one root node; Success/Failure only.
- **Node shape**: `{ id, type, children?, params? }`

### Compilation Flow

1. **Source**: A JSON file conforming to `behavior-tree.schema.json` (e.g. `schemas/behavior-tree.example.json`).
2. **Config embedding**: The tree can be included in d-app config via `validationTree` (see `schemas/config/parcel-config.schema.json`).
3. **Build step**: `dship-bt-compile <behavior-tree.json> [--output validation.rs]` reads the JSON and emits Rust validation code.
4. **Integration**: The generated code is compiled into the contract; `ValidationConfig::from_config` or similar logic consumes the tree at runtime (or the tree is fully inlined at compile time).

### Node Types

| Type | Children | Params | Use |
|------|----------|--------|-----|
| **Sequence** | 1–32 | — | Run in order; fail on first Failure. |
| **Selector** | 1–32 | — | Run until one succeeds. |
| **Inverter** | 1 | — | Invert child result. |
| **Condition** | — | `field`, `op`, `value` | Generic comparison. |
| **RangeCheck** | — | `field`, `op`, `min`/`max`/`value` | Numeric/range checks. |
| **EnumCheck** | — | `field`, `allowed` | Value must be in allowed set. |
| **RegexCheck** | — | `field`, `pattern` | String matches regex. |
| **OracleCall** | — | `oracle`, `endpoint`, `args?` | MultiverseX oracle call. |
| **ProofRequired** | — | `proofType`, `publicInputs` | ZK proof requirement. |
| **GasLimit** | 1 | `limit` | Decorate child with gas limit. |

### Example Document

```json
{
  "version": "1.0",
  "root": {
    "id": "parcel-validation",
    "type": "Sequence",
    "children": [
      {
        "id": "weight-range",
        "type": "RangeCheck",
        "params": { "field": "weight", "op": "le", "max": 30000 }
      },
      {
        "id": "weight-unit-enum",
        "type": "EnumCheck",
        "params": { "field": "weightUnit", "allowed": ["G", "KG", "LB", "OZ"] }
      }
    ]
  }
}
```

### Optional Blackboard

A `blackboard` object (key-value store) can hold intermediate values for cross-node state during validation.

---

## 2. Implementing a UI to Define and Save the Behavior Tree

### Goals

- Provide a diagram-based UI for building behavior trees.
- Allow saving/loading JSON that matches `behavior-tree.schema.json`.
- Validate the tree against the schema before saving.

### Required Capabilities

| Capability | Description |
|------------|-------------|
| **Node creation** | Add nodes with `id` and `type`; support all node types above. |
| **Hierarchy** | Attach children to composites (Sequence, Selector, Inverter, GasLimit). |
| **Params editor** | Per-type forms for `params` (e.g. field name, op, min/max/value for RangeCheck). |
| **Tree structure** | Single root; enforce `children` rules (e.g. Inverter/GasLimit: exactly 1 child). |
| **Serialization** | Export to JSON in the exact structure expected by the schema. |
| **Deserialization** | Load JSON and reconstruct the tree for editing. |
| **Schema validation** | Validate against `behavior-tree.schema.json` before save (e.g. via AJV or similar). |

### Suggested Tech Stack

- **Canvas/graph library**: React Flow, mxGraph, or a lightweight D3-based tree view for the diagram.
- **Form library**: For `params` editing (React Hook Form, Formik, or plain controlled inputs).
- **Validation**: `ajv` or `ajv-formats` with `behavior-tree.schema.json` to ensure valid output.
- **Storage**: File save via browser download, or backend store if integrated into a build pipeline.

### Serialization / Deserialization

- Use the schema as the single source of truth.
- `dship-validation-bt` provides serde-compatible Rust structs (`BehaviorTree`, `Node`, `NodeType`, etc.) for compile-time processing; the UI should produce JSON that deserializes correctly into those structs.
- Round-trip: Load JSON → build in-memory tree → edit in UI → serialize back to JSON → validate against schema → save.

### UI Workflow (Suggested)

1. **Load**: User opens a JSON file or creates a new tree.
2. **Edit**: Drag-and-drop or menu to add nodes; forms to edit `id`, `type`, `params`; connect children for composites.
3. **Validate**: Run schema validation before export.
4. **Save**: Export valid JSON (e.g. `validation-tree.json`) for use in `dship-bt-compile` or d-app config.

### References

- `schemas/behavior-tree.schema.json` – full JSON Schema
- `schemas/behavior-tree.example.json` – example tree
- `schemas/config/parcel-config.schema.json` – config embedding via `validationTree`
- `crates/dship-validation-bt` – Rust AST models and `dship-bt-compile` tool
