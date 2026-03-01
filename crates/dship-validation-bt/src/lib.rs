//! Behavior tree AST for d-ship input validation.
//!
//! Mirrors `schemas/behavior-tree.schema.json` (Behavior3-style).
//! Use at build time to embed BT JSON and compile into Rust validation code.
//!
//! Status: Success / Failure only (no Running).

use serde::{Deserialize, Serialize};

/// Root document: tree + optional blackboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorTree {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub root: Node,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blackboard: Option<Blackboard>,
}

/// Blackboard: key-value store for intermediate validation state.
pub type Blackboard = std::collections::HashMap<String, String>;

/// Node: id, type, optional children, optional params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Node>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum NodeType {
    /// Run children in order; fail on first Failure.
    Sequence,
    /// Try children; succeed on first Success.
    Selector,
    /// Invert child result.
    Inverter,
    /// Generic condition (field, op, value).
    Condition,
    /// Numeric/range check: field, op, min/max/value.
    RangeCheck,
    /// Value must be in allowed set.
    EnumCheck,
    /// String matches regex.
    RegexCheck,
    /// External oracle call (MultiverseX).
    OracleCall,
    /// Require ZK proof (MultiverseX).
    ProofRequired,
    /// Decorate child with gas limit.
    GasLimit,
}

/// Params for RangeCheck.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeCheckParams {
    pub field: String,
    pub op: RangeOp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RangeOp {
    Lt,
    Le,
    Eq,
    Ne,
    Ge,
    Gt,
}

/// Params for EnumCheck.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumCheckParams {
    pub field: String,
    pub allowed: Vec<serde_json::Value>,
}

/// Params for RegexCheck.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexCheckParams {
    pub field: String,
    pub pattern: String,
}

/// Params for OracleCall (MultiverseX).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleCallParams {
    pub oracle: String,
    pub endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

/// Params for ProofRequired (MultiverseX).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofRequiredParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_inputs: Option<Vec<serde_json::Value>>,
}

/// Params for GasLimit decorator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasLimitParams {
    pub limit: u64,
}

impl BehaviorTree {
    /// Parse BT from JSON string.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
