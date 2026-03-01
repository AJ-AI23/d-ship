//! Build-time: read BT JSON, validate, emit Rust code.
//!
//! Usage: dship-bt-compile <behavior-tree.json> [--output validation.rs]

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
    let _ = tree; // TODO: emit Rust validation code from tree

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

fn emit_rust(tree: &BehaviorTree) -> String {
    let mut s = String::from("//! Auto-generated validation from behavior tree. Do not edit.\n\n");
    emit_node_fn(&mut s, &tree.root, 0);
    s.push_str("\npub fn validate(_input: &[u8]) -> bool { true }\n");
    s
}

fn emit_node_fn(s: &mut String, node: &Node, depth: usize) {
    let indent = "    ".repeat(depth);
    match node.node_type {
        NodeType::Sequence => {
            s.push_str(&format!("{}// Sequence: {}\n", indent, node.id));
            if let Some(ref children) = node.children {
                for c in children {
                    emit_node_fn(s, c, depth + 1);
                }
            }
        }
        NodeType::RangeCheck => {
            if let Some(ref p) = node.params {
                if let Some(obj) = p.as_object() {
                    let field = obj.get("field").and_then(|v| v.as_str()).unwrap_or("?");
                    let op = obj.get("op").and_then(|v| v.as_str()).unwrap_or("?");
                    let value = obj.get("value").map(|v| v.to_string()).unwrap_or_else(|| "?".into());
                    s.push_str(&format!(
                        "{}// RangeCheck: {} {} {}\n",
                        indent, field, op, value
                    ));
                }
            }
        }
        _ => {
            s.push_str(&format!("{}// {}: {}\n", indent, format!("{:?}", node.node_type), node.id));
        }
    }
}
