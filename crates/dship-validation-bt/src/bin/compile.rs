//! Build-time: read BT JSON, validate, emit Rust code.
//!
//! Usage: dship-bt-compile <behavior-tree.json> [--output validation.rs]
//!
//! Output is no_std compatible for MultiversX contract use.

use dship_validation_bt::{BehaviorTree, Node, NodeType};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <behavior-tree.json> [--output validation.rs]", args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];
    let json = fs::read_to_string(input_path).expect("read input");
    let tree: BehaviorTree = serde_json::from_str(&json).expect("parse BT JSON");

    let out_idx = args.iter().position(|a| a == "--output");
    if let Some(i) = out_idx {
        if i + 1 < args.len() {
            let out_path = &args[i + 1];
            let code = emit_rust(&tree);
            fs::write(Path::new(out_path), code).expect("write output");
            println!("Wrote validation to {}", out_path);
        }
    }
}

/// Normalize BT field name to Rust param name (camelCase -> snake_case).
fn field_to_param(field: &str) -> String {
    let mut s = String::new();
    for (i, c) in field.chars().enumerate() {
        if c == '.' {
            s.push('_');
        } else if c.is_uppercase() && i > 0 {
            s.push('_');
            s.push(c.to_lowercase().next().unwrap());
        } else if c != '.' {
            s.push(c.to_lowercase().next().unwrap_or(c));
        }
    }
    s
}

/// Get the canonical param name for a field. E.g. "dangerousGoods" (eq null) and "dangerousGoods.length" -> "dangerous_goods_len"
fn canonical_param(field: &str, is_null_check: bool) -> String {
    if field.contains(".length") || field.contains(".len") {
        let base = field_to_param(field)
            .trim_end_matches("length")
            .trim_end_matches('_')
            .to_string();
        format!("{}_len", base)
    } else if is_null_check {
        format!("{}_len", field_to_param(field))
    } else {
        field_to_param(field)
    }
}

/// Param for RangeCheck/Condition field - same convention as canonical_param for .length
fn param_for_field(field: &str) -> String {
    if field.contains(".length") || field.contains(".len") {
        let base = field_to_param(field)
            .trim_end_matches("length")
            .trim_end_matches('_')
            .to_string();
        format!("{}_len", base)
    } else {
        field_to_param(field)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum FieldKind {
    /// u64 for weight, amount, etc.
    NumericU64,
    /// usize for .length fields
    NumericUsize,
    /// &[u8] for string/enum checks
    Slice,
}

fn collect_fields(node: &Node, fields: &mut std::collections::BTreeMap<String, FieldKind>) {
    if let Some(ref p) = node.params {
        if let Some(obj) = p.as_object() {
            if let Some(field) = obj.get("field").and_then(|v| v.as_str()) {
                let kind = match node.node_type {
                    NodeType::EnumCheck => FieldKind::Slice,
                    NodeType::RangeCheck | NodeType::Condition => {
                        let is_len = field.contains(".length") || field.contains(".len");
                        if is_len {
                            FieldKind::NumericUsize
                        } else if let Some(v) = obj.get("value") {
                            if v.is_null() {
                                FieldKind::NumericUsize // null check -> len
                            } else if v.is_f64() || v.is_i64() || v.is_u64() {
                                FieldKind::NumericU64
                            } else {
                                FieldKind::Slice
                            }
                        } else if obj.get("min").is_some() || obj.get("max").is_some() {
                            FieldKind::NumericU64
                        } else {
                            FieldKind::NumericU64
                        }
                    }
                    _ => FieldKind::NumericU64,
                };
                let param = if node.node_type == NodeType::Condition
                    && obj.get("value").map(|v| v.is_null()).unwrap_or(false)
                {
                    canonical_param(field, true)
                } else {
                    param_for_field(field)
                };
                fields.insert(param, kind);
            }
        }
    }
    if let Some(ref children) = node.children {
        for c in children {
            collect_fields(c, fields);
        }
    }
}

fn emit_rust(tree: &BehaviorTree) -> String {
    let mut fields: std::collections::BTreeMap<String, FieldKind> = std::collections::BTreeMap::new();
    collect_fields(&tree.root, &mut fields);

    let mut s = String::from("//! Auto-generated validation from behavior tree. Do not edit.\n\n");

    // Function signature (params in alphabetical order)
    let params: Vec<String> = fields
        .iter()
        .map(|(name, kind)| match kind {
            FieldKind::NumericU64 => format!("{}: u64", name),
            FieldKind::NumericUsize => format!("{}: usize", name),
            FieldKind::Slice => format!("{}: &[u8]", name),
        })
        .collect();
    s.push_str(&format!("pub fn validate({}) -> bool {{\n", params.join(", ")));

    // Body
    emit_node_body(&mut s, &tree.root, 1, &fields);

    s.push_str("    true\n");
    s.push_str("}\n");
    s
}

fn emit_node_body(
    s: &mut String,
    node: &Node,
    depth: usize,
    fields: &std::collections::BTreeMap<String, FieldKind>,
) {
    let indent = "    ".repeat(depth);
    match node.node_type {
        NodeType::Sequence => {
            if let Some(ref children) = node.children {
                for c in children {
                    emit_node_body(s, c, depth, fields);
                }
            }
        }
        NodeType::Selector => {
            if let Some(ref children) = node.children {
                // Selector: first success = success; all fail = fail. As child of Sequence, success = continue, fail = return false.
                let mut branches = Vec::new();
                for c in children.iter() {
                    let mut expr = String::new();
                    emit_condition_expr(&mut expr, c, fields, false);
                    branches.push(expr);
                }
                if branches.len() == 1 {
                    s.push_str(&format!("{}if !({}) {{ return false; }}\n", indent, branches[0]));
                } else {
                    let combined = branches.join(" || ");
                    s.push_str(&format!("{}if !({}) {{ return false; }}\n", indent, combined));
                }
            }
        }
        NodeType::RangeCheck => {
            if let Some(ref p) = node.params {
                if let Some(obj) = p.as_object() {
                    let field = obj.get("field").and_then(|v| v.as_str()).unwrap_or("?");
                    let op = obj.get("op").and_then(|v| v.as_str()).unwrap_or("eq");
                    let param = param_for_field(field);
                    let (cond, ok) = match op {
                        "lt" => {
                            let val = obj.get("max").or(obj.get("value"))
                                .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64)))
                                .unwrap_or(0.0);
                            (format!("{} < {}", param, val as i64), true)
                        }
                        "le" => {
                            let val = obj.get("max").or(obj.get("value"))
                                .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64)))
                                .unwrap_or(0.0);
                            (format!("{} <= {}", param, val as i64), true)
                        }
                        "eq" => {
                            let val = obj.get("value")
                                .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64)))
                                .unwrap_or(0.0);
                            (format!("{} == {}", param, val as i64), true)
                        }
                        "ne" => {
                            let val = obj.get("value")
                                .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64)))
                                .unwrap_or(0.0);
                            (format!("{} != {}", param, val as i64), true)
                        }
                        "ge" => {
                            let val = obj.get("min").or(obj.get("value"))
                                .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64)))
                                .unwrap_or(0.0);
                            (format!("{} >= {}", param, val as i64), true)
                        }
                        "gt" => {
                            let val = obj.get("min").or(obj.get("value"))
                                .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64)))
                                .unwrap_or(0.0);
                            (format!("{} > {}", param, val as i64), true)
                        }
                        _ => (format!("false"), false),
                    };
                    if ok {
                        s.push_str(&format!("{}if !({}) {{ return false; }}\n", indent, cond));
                    }
                }
            }
        }
        NodeType::EnumCheck => {
            if let Some(ref p) = node.params {
                if let Some(obj) = p.as_object() {
                    let field = obj.get("field").and_then(|v| v.as_str()).unwrap_or("?");
                    let param = field_to_param(field);
                    let empty: Vec<serde_json::Value> = vec![];
                    let allowed = obj.get("allowed").and_then(|a| a.as_array()).unwrap_or(&empty);
                    let mut checks = Vec::new();
                    for a in allowed {
                        if let Some(slit) = a.as_str() {
                            let bytes = format!("b\"{}\"", slit);
                            checks.push(format!("{} == {}", param, bytes));
                        }
                    }
                    if !checks.is_empty() {
                        let expr = checks.join(" || ");
                        s.push_str(&format!("{}if !({}) {{ return false; }}\n", indent, expr));
                    }
                }
            }
        }
        NodeType::Condition => {
            if let Some(ref p) = node.params {
                if let Some(obj) = p.as_object() {
                    let field = obj.get("field").and_then(|v| v.as_str()).unwrap_or("?");
                    let op = obj.get("op").and_then(|v| v.as_str()).unwrap_or("eq");
                    let val = obj.get("value");
                    let param = canonical_param(field, val.map(|v| v.is_null()).unwrap_or(false));
                    if val.map(|v| v.is_null()).unwrap_or(false) {
                        // null check -> empty/len 0
                        s.push_str(&format!("{}if {} != 0 {{ return false; }}\n", indent, param));
                    } else if let Some(n) = val.and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64))) {
                        match op {
                            "eq" => s.push_str(&format!("{}if {} != {} {{ return false; }}\n", indent, param, n as i64)),
                            "ne" => s.push_str(&format!("{}if {} == {} {{ return false; }}\n", indent, param, n as i64)),
                            _ => {}
                        }
                    }
                }
            }
        }
        NodeType::Inverter => {
            if let Some(ref children) = node.children {
                if let Some(c) = children.first() {
                    s.push_str(&format!("{}if ", indent));
                    emit_condition_expr(s, c, fields, true);
                    s.push_str(" { return false; }\n");
                }
            }
        }
        _ => {
            // Fallback: recurse into children for Sequence-like behavior
            if let Some(ref children) = node.children {
                for c in children {
                    emit_node_body(s, c, depth, fields);
                }
            }
        }
    }
}

/// Emit a boolean expression for a node (used in Selector). Invert for Inverter.
fn emit_condition_expr(
    s: &mut String,
    node: &Node,
    _fields: &std::collections::BTreeMap<String, FieldKind>,
    invert: bool,
) {
    match node.node_type {
        NodeType::RangeCheck => {
            if let Some(ref p) = node.params {
                if let Some(obj) = p.as_object() {
                    let field = obj.get("field").and_then(|v| v.as_str()).unwrap_or("?");
                    let op = obj.get("op").and_then(|v| v.as_str()).unwrap_or("eq");
                    let param = param_for_field(field);
                    let val = obj.get("max").or(obj.get("value"))
                        .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64)))
                        .unwrap_or(0.0);
                    let (cond, sign) = match op {
                        "eq" => (format!("{} == {}", param, val as i64), true),
                        "le" => (format!("{} <= {}", param, val as i64), true),
                        _ => (format!("true"), true),
                    };
                    if invert && sign {
                        s.push_str(&format!("!({})", cond));
                    } else {
                        s.push_str(&cond);
                    }
                }
            }
        }
        NodeType::Condition => {
            if let Some(ref p) = node.params {
                if let Some(obj) = p.as_object() {
                    let field = obj.get("field").and_then(|v| v.as_str()).unwrap_or("?");
                    let param = canonical_param(field, obj.get("value").map(|v| v.is_null()).unwrap_or(false));
                    let val = obj.get("value");
                    if val.map(|v| v.is_null()).unwrap_or(false) {
                        if invert {
                            s.push_str(&format!("{} != 0", param));
                        } else {
                            s.push_str(&format!("{} == 0", param));
                        }
                    } else {
                        s.push_str("true");
                    }
                }
            }
        }
        _ => s.push_str("true"),
    }
}
