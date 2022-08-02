use lazy_static::lazy_static;
use mut_static::MutStatic;
use num::integer::sqrt;
use pathfinding::prelude::astar;
use serde::{Deserialize, Serialize};
use std::num::ParseIntError;

#[derive(Serialize, Deserialize, Default, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Node {
    unique_id: usize,
    x: usize,
    y: usize,
    z: usize,
    connected_nodes_id: Vec<usize>,
}

impl Node {
    fn successors(&self) -> Vec<(Self, usize)> {
        self.connected_nodes_id
            .iter()
            .filter_map(|id| NODES.read().unwrap().get(*id).cloned())
            .flatten()
            .map(|node| (node.clone(), self.distance(&node)))
            .collect()
    }

    fn distance(&self, other: &Self) -> usize {
        sqrt(((self.x as isize - other.x as isize).pow(2) + (self.y as isize - other.y as isize).pow(2)) as usize)
    }
}

lazy_static! {
    static ref NODES: MutStatic<Vec<Option<Node>>> = MutStatic::new();
}

pub enum RegisteringNodesError {
    SerdeError(serde_json::Error),
    MutexError(mut_static::Error),
    NodesNotCorrectlyIndexed,
}

impl From<serde_json::Error> for RegisteringNodesError {
    fn from(error: serde_json::Error) -> Self {
        RegisteringNodesError::SerdeError(error)
    }
}
impl From<mut_static::Error> for RegisteringNodesError {
    fn from(error: mut_static::Error) -> Self {
        RegisteringNodesError::MutexError(error)
    }
}

impl std::fmt::Display for RegisteringNodesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisteringNodesError::MutexError(e) => write!(f, "Mutex error : {e}"),
            RegisteringNodesError::SerdeError(e) => write!(f, "Parsing error : {e}"),
            RegisteringNodesError::NodesNotCorrectlyIndexed => {
                write!(f, "Node not indexed properly")
            }
        }
    }
}

byond_fn!(fn register_nodes(json) {
    match register_nodes_(json) {
        Ok(s) => Some(s),
        Err(e) => Some(format!("{e}"))
    }
});

fn register_nodes_(json: &str) -> Result<String, RegisteringNodesError> {
    let nodes: Vec<Option<Node>> = serde_json::from_str(json)?;
    if nodes
        .iter()
        .flatten()
        .enumerate()
        .filter(|(i, node)| i != &node.unique_id)
        .count()
        != 0
    {
        return Err(RegisteringNodesError::NodesNotCorrectlyIndexed);
    }
    if let Err(e) = NODES.set(nodes) {
        return Err(RegisteringNodesError::MutexError(e));
    }
    Ok("1".to_string())
}

byond_fn!(fn add_node(json) {
    match add_node_(json) {
        Ok(s) => Some(s),
        Err(e) => Some(format!("{e}"))
    }
});

fn add_node_(json: &str) -> Result<String, RegisteringNodesError> {
    let new_node: Node = serde_json::from_str(json)?;
    if new_node.unique_id != NODES.read()?.len() {
        return Err(RegisteringNodesError::NodesNotCorrectlyIndexed);
    }
    new_node.connected_nodes_id.iter().for_each(|i| {
        if let Some(Some(node)) = NODES.write().unwrap().get_mut(*i) {
            node.connected_nodes_id.push(new_node.unique_id);
        }
    });
    NODES.write()?.push(Some(new_node));
    Ok("1".to_string())
}

pub enum DeleteNodeError {
    ParsingError(ParseIntError),
    MutexError(mut_static::Error),
    NodeNotFound,
}

impl From<ParseIntError> for DeleteNodeError {
    fn from(error: ParseIntError) -> Self {
        DeleteNodeError::ParsingError(error)
    }
}

impl From<mut_static::Error> for DeleteNodeError {
    fn from(error: mut_static::Error) -> Self {
        DeleteNodeError::MutexError(error)
    }
}

impl std::fmt::Display for DeleteNodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteNodeError::MutexError(e) => write!(f, "Mutex error : {e}"),
            DeleteNodeError::ParsingError(e) => write!(f, "Parsing error : {e}"),
            DeleteNodeError::NodeNotFound => write!(f, "Node not found"),
        }
    }
}

byond_fn!(fn remove_node(unique_id) {
    match remove_node_(unique_id) {
        Ok(s) => Some(s),
        Err(e) => Some(format!("{e}"))
    }
});

fn remove_node_(unique_id: &str) -> Result<String, DeleteNodeError> {
    let unique_id = unique_id.parse::<usize>()?;
    let mut nodes = NODES.write()?;

    let node_to_delete = nodes.get(unique_id);
    if node_to_delete.is_none() || node_to_delete.unwrap().is_none() {
        return Err(DeleteNodeError::NodeNotFound);
    }
    let node_to_delete = node_to_delete.unwrap().clone().unwrap();
    node_to_delete.connected_nodes_id.iter().for_each(|i| {
        if let Some(Some(node)) = nodes.get_mut(*i) {
            node.connected_nodes_id.retain(|id| id != i)
        }
    });

    let node = nodes.get_mut(unique_id).unwrap();
    *node = None;
    Ok("1".to_string())
}

pub enum AstarError {
    MutexError(mut_static::Error),
    StartNodeNotFound,
    GoalNodeNotFound,
    NoPathFound,
}

impl From<mut_static::Error> for AstarError {
    fn from(error: mut_static::Error) -> Self {
        AstarError::MutexError(error)
    }
}

byond_fn!(fn astar_generate_path(start_node_id, goal_node_id) {
    if let (Ok(start_node_id), Ok(goal_node_id)) = (start_node_id.parse::<usize>(), goal_node_id.parse::<usize>()) {
        match astar_generate_path_(start_node_id, goal_node_id) {
            Ok(vector) => Some(match serde_json::to_string(&vector) {
                Ok(s) => s,
                Err(_) => "Cannot serialize path".to_string(),
            }),
            Err(e) => Some(match e {
                AstarError::MutexError(e) => format!("Mutex error : {}", e),
                AstarError::StartNodeNotFound => "Start node not found".to_string(),
                AstarError::GoalNodeNotFound => "Goal node not found".to_string(),
                AstarError::NoPathFound => "No path found".to_string(),
            })
        }
    }
    else {
        Some("Invalid arguments".to_string())
    }
});

fn astar_generate_path_(
    start_node_id: usize,
    goal_node_id: usize,
) -> Result<Vec<usize>, AstarError> {
    let nodes = NODES.read()?;

    let start_node = nodes.get(start_node_id);
    if start_node.is_none() || start_node.unwrap().is_none() {
        return Err(AstarError::StartNodeNotFound);
    }

    let goal_node = nodes.get(goal_node_id);
    if goal_node.is_none() || goal_node.unwrap().is_none() {
        return Err(AstarError::GoalNodeNotFound);
    }

    if start_node.unwrap().as_ref().unwrap().z != goal_node.unwrap().as_ref().unwrap().z {
        return Err(AstarError::NoPathFound)
    }

    let path = astar(
        &start_node.unwrap().clone().unwrap(),
        |node| node.successors(),
        |node| node.distance(&goal_node.unwrap().clone().unwrap()),
        |node| node == &goal_node.unwrap().clone().unwrap(),
    );
    if path.is_none() {
        return Err(AstarError::NoPathFound);
    }

    let (path, _) = path.unwrap();
    Ok(path.into_iter().map(|node| node.unique_id).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_node() {
        let json = std::fs::read_to_string("tests/rsc/ai_nodes_info.json").unwrap();
        assert!(register_nodes_(&json).is_ok())
    }

    #[test]
    fn test_remove_node() {
        let json = std::fs::read_to_string("tests/rsc/ai_nodes_info.json").unwrap();
        register_nodes_(&json);
        assert!(remove_node_("15").is_ok())
    }

    #[test]
    fn test_add_node() {
        let json = std::fs::read_to_string("tests/rsc/ai_nodes_info.json").unwrap();
        register_nodes_(&json);
        let mut node_to_add = NODES.read().unwrap().get(18).unwrap().clone().unwrap();
        node_to_add.unique_id = NODES.read().unwrap().len();
        let node_to_add = serde_json::to_string(&node_to_add).unwrap();
        assert!(add_node_(&node_to_add).is_ok())
    }
}
