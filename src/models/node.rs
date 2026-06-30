use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Map,
    Agent,
    Site,
    TacticType,
    AgentCombo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: i64,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub name: String,
    pub parent_id: Option<i64>,
}
