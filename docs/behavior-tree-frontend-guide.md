# Frontend Diagram Guide: Behavior Tree Editor

This document describes what to take into account when building a frontend diagram or editor to represent and maintain validation behavior trees. The trees are JSON documents that compile into Rust validation code via `dship-bt-compile`.

---

## 1. Document Structure

The tree is a single root node with optional metadata:

| Property | Required | Type | Notes |
|----------|----------|------|-------|
| `version` | No | string | Schema version (e.g. `"1.0"`) for migration |
| `root` | Yes | Node | Single root; tree is defined recursively |
| `blackboard` | No | object | Key-value store; not yet used by compiler |

---

## 2. Node Types and Hierarchy

### Composite Nodes (have children)

| Node Type | Children | Semantics |
|-----------|----------|-----------|
| **Sequence** | 1–32 | Run in order; fail on first failure |
| **Selector** | 1–32 | Try in order; succeed on first success |
| **Inverter** | exactly 1 | Negate child result |

### Leaf Nodes (params only, no children)

| Node Type | Purpose |
|-----------|---------|
| **RangeCheck** | Numeric comparison: `field op value` or `field` vs `min`/`max` |
| **EnumCheck** | Field value must be in `allowed` list |
| **Condition** | Generic `field op value` (incl. null checks) |
| **RegexCheck** | String matches pattern (schema-defined; compiler may not emit yet) |

### Deferred / Future

| Node Type | Status |
|-----------|--------|
| **OracleCall** | Schema only; not compiled |
| **ProofRequired** | Schema only; not compiled |
| **GasLimit** | Schema only; decorator with child; not compiled |

**Diagram implication**: Visually distinguish composite vs leaf. Composite nodes are branch points; leaf nodes are validation checks. Show child slots (1–32 for Sequence/Selector, 1 for Inverter).

---

## 3. Node Requirements by Type

Every node has:
- `id` (string, min length 1) – unique within the tree
- `type` (one of the enum values)

Conditional requirements:

| Type | Required | Params |
|------|----------|--------|
| Sequence, Selector | `children` (1–32 items) | — |
| Inverter | `children` (exactly 1) | — |
| RangeCheck | — | `field`, `op`; plus `min`/`max`/`value` as needed |
| EnumCheck | — | `field`, `allowed` (array, min 1 item) |
| RegexCheck | — | `field`, `pattern` |
| Condition | — | `field`, `op`, `value` (optional) |
| GasLimit | `children` (1), `params.limit` | — |

**Diagram implication**: When adding or editing a node, enforce required props. Show type-specific param forms (e.g. RangeCheck needs `op` + one of `min`/`max`/`value`).

---

## 4. RangeCheck Operations

For `RangeCheck`, `op` must be one of: `lt`, `le`, `eq`, `ne`, `ge`, `gt`.

Value source by operation:
- `lt`, `le` → typically `max` or `value`
- `ge`, `gt` → typically `min` or `value`
- `eq`, `ne` → `value` (numeric)

**Diagram implication**: Dropdown for `op`; conditional inputs for `min`, `max`, or `value` based on `op`. Validate that at least one numeric value is present.

---

## 5. Field Naming and Parameter Mapping

The compiler turns `params.field` into a Rust function parameter. The contract passes concrete values when calling the generated `validate(...)`.

### Naming Convention

| BT `field` | Rust param | Type |
|------------|------------|------|
| `weight` | `weight` | `u64` |
| `weightUnit` | `weight_unit` | `&[u8]` |
| `dangerousGoods` with `value: null` | `dangerous_goods_len` | `usize` |
| `dangerousGoods.length` | `dangerous_goods_len` | `usize` |
| `batch_size` | `batch_size` | `u64` |
| `tracking_number_len` | `tracking_number_len` | `usize` |

Rules:
- CamelCase → snake_case (`weightUnit` → `weight_unit`)
- `.` → `_` (`dangerousGoods.length` → `dangerous_goods_len` after base extraction)
- `field` with `value: null` (Condition) → `{base}_len` (empty/zero check)
- `.length` / `.len` suffix → `usize`; EnumCheck → `&[u8]`; other numeric → `u64`

### Parameter Order

All parameters are collected and sorted **alphabetically** to form the function signature. The contract must pass arguments in that order.

**Diagram implication**: Show the resulting param list (e.g. sidebar or tooltip). Warn if field names duplicate after normalization (e.g. `dangerousGoods` null + `dangerousGoods.length` both map to `dangerous_goods_len` – this is intentional and shared).

---

## 6. Composite Semantics for the Diagram

- **Sequence**: All children must succeed. Show as vertical stack or left-to-right flow; label as “AND” semantics.
- **Selector**: First success wins. Show as branching; label as “OR” semantics.
- **Inverter**: Flips success/failure. Show as wrapper around single child.

Use clear visual cues (icons, colors, or line style) so Sequence vs Selector is obvious.

---

## 7. Empty and Minimal Trees

- A `Sequence` with `children: []` compiles to `validate() -> bool { true }` (always pass).
- Contracts can use this as a placeholder until real validation is added.

**Diagram implication**: Allow empty Sequence. Do not force at least one child for Sequence (schema allows 1–32, but compiler handles 0). If your validator enforces minItems, consider relaxing for root Sequence.

---

## 8. Id Uniqueness

Each node `id` must be unique within the tree. Used for debugging and potential future features.

**Diagram implication**: Auto-generate ids when creating nodes (e.g. `weight-range`, `dg-max-one`). Validate uniqueness on save. Support manual override with uniqueness check.

---

## 9. Round-Trip and Serialization

- JSON is the canonical format.
- Preserve `version` and `blackboard` if present.
- Child order matters (Sequence and Selector run children in array order).
- Whitespace/formatting is not semantically significant; pretty-print on save if desired.

**Diagram implication**: Load from JSON, edit in-memory, serialize back to JSON without dropping or reordering data. Avoid normalizing field names in the stored JSON; the compiler handles mapping.

---

## 10. Validation Before Save / Compile

Before saving or exporting:

1. **Structural**: All required fields present; children count correct per type.
2. **Params**: RangeCheck has `field` + `op` + (min/max/value); EnumCheck has `field` + non-empty `allowed`.
3. **Id uniqueness**: No duplicate ids.
4. **Depth**: Consider a max depth (e.g. 10) to avoid stack overflow in compiler or UI.

Optionally run `dship-bt-compile` in the background and surface compile errors (e.g. unknown node type, missing params).

---

## 11. Contract Context and Field Hints

Different contracts validate different concepts. A helpful UX is to show which fields are typical per contract:

| Contract | Example fields |
|----------|----------------|
| parcel | `weight`, `weightUnit`, `dangerousGoods`, `dangerousGoods.length` |
| tracker | `tracking_number_len`, `event_type_len` |
| pickup | `batch_size` |
| agreement | `amount` |
| forwarder-agreement | `payment` |

**Diagram implication**: When a contract is selected (or inferred from file path), offer field autocomplete or suggestions. Document that the contract must pass values in the generated param order.

---

## 12. Visual Layout Suggestions

- **Top-down**: Root at top; children below. Works well for Sequence-heavy trees.
- **Left-right**: Root on left; flow right. Good for linear flows.
- **Collapsible**: Composite nodes expand to show children; collapse to save space.
- **Color coding**: e.g. green = Sequence, blue = Selector, orange = leaf checks.
- **Connectors**: Show parent-child edges. For Sequence/Selector, ordering along the edge indicates execution order.

---

## 13. Common Pitfalls

| Pitfall | Mitigation |
|---------|------------|
| Condition `value: null` vs `value: 0` | Null means “empty/zero length”; 0 is numeric. Use correct type in UI. |
| `dangerousGoods` vs `dangerousGoods.length` | Same logical param `dangerous_goods_len`; Condition uses it for “empty”, RangeCheck for “exactly 1”. |
| EnumCheck `allowed` with non-strings | Compiler expects string literals for `&[u8]` comparison. Restrict to string items in UI. |
| Parameter order mismatch | Always show generated param list. Contract integration doc should specify order. |
| RegexCheck/OracleCall/ProofRequired | In schema but may not compile. Mark as “planned” or “unsupported” in UI. |

---

## 14. References

- Schema: `schemas/behavior-tree.schema.json`
- Example: `schemas/behavior-tree.example.json`, `contracts/parcel/validation-tree.json`
- Compiler: `crates/dship-validation-bt/src/bin/compile.rs`
- Contract usage: `contracts/parcel/src/lib.rs` (see `generated_validation::validate`)
