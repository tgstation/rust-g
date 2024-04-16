use num_integer::sqrt;
use pathfinding::prelude::astar;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::hash::Hash;
use std::num::ParseIntError;
use std::rc::Rc;
use thiserror::Error;

thread_local! {
static NODES: RefCell<Vec<Option<Rc<Node>>>> = const { RefCell::new(Vec::new()) };
}

fn get_nodes_len() -> usize {
    NODES.with(|nodes_ref| nodes_ref.borrow().len())
}

fn get_node(id: usize) -> Option<Option<Rc<Node>>> {
    NODES.with(|nodes_ref| nodes_ref.borrow().get(id).cloned())
}

fn push_node(node: Node) {
    NODES.with(|nodes_ref| nodes_ref.borrow_mut().push(Some(Rc::new(node))));
}

fn null_out_node(id: usize) {
    NODES.with(|nodes_ref| nodes_ref.borrow_mut()[id] = None);
}

// Container for a node. Exist mainly to be able to implement Hash, which is not implemented for RefCell
#[derive(Serialize, Deserialize, Default, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Node {
    // A unique id that acts as its index in NODES
    unique_id: usize,
    // Position of the node in byond
    x: usize,
    y: usize,
    z: usize,
    // Indexes of nodes connected to this one
    connected_nodes_id: Vec<usize>,
}

impl Node {
    // Return a vector of all connected nodes, encapsulated in a NodeContainer.
    fn successors(&self) -> Vec<(Rc<Node>, usize)> {
        self.connected_nodes_id
            .iter()
            .filter_map(|index| get_node(*index))
            .flatten()
            .map(|node| (node.clone(), self.distance(node.as_ref())))
            .collect()
    }

    // Return the geometric distance between this node and another one.
    fn distance(&self, other: &Self) -> usize {
        sqrt(
            ((self.x as isize - other.x as isize).pow(2)
                + (self.y as isize - other.y as isize).pow(2)) as usize,
        )
    }
}

#[derive(Error, Debug)]
enum RegisteringNodesError {
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error("Nodes were not correctly indexed")]
    NodesNotCorrectlyIndexed,
}

byond_fn!(fn register_nodes_astar(json) {
    match register_nodes(json) { Ok(s) => Some(s),
        Err(e) => Some(format!("{e}"))
    }
});

// Builds a list of nodes from a json file.
// Errors if the input list of nodes is not correctly indexed. Each node should have for unique id its position in the list, with the first unique-id being 0.
fn register_nodes(json: &str) -> Result<String, RegisteringNodesError> {
    let deserialized_nodes: Vec<Node> = serde_json::from_str(json)?;
    if deserialized_nodes
        .iter()
        .enumerate()
        .filter(|(i, node)| i != &node.unique_id)
        .count()
        != 0
    {
        return Err(RegisteringNodesError::NodesNotCorrectlyIndexed);
    }

    deserialized_nodes.into_iter().for_each(push_node);

    Ok("1".to_string())
}

byond_fn!(fn add_node_astar(json) {
    match add_node(json) {
        Ok(s) => Some(s),
        Err(e) => Some(format!("{e}"))
    }
});

// Add a node to the static list of node.
// If it is connected to other existing nodes, it will update their connected_nodes_id list.
fn add_node(json: &str) -> Result<String, RegisteringNodesError> {
    let new_node: Node = serde_json::from_str(json)?;

    // As always, a node unique id should correspond to its index in NODES
    if new_node.unique_id != get_nodes_len() {
        return Err(RegisteringNodesError::NodesNotCorrectlyIndexed);
    }

    // Make sure every connection we have we other nodes is 2 ways
    for index in new_node.connected_nodes_id.iter() {
        NODES.with(|nodes_ref| {
            if let Some(Some(node)) = nodes_ref.borrow_mut().get_mut(*index) {
                Rc::get_mut(node)
                    .unwrap()
                    .connected_nodes_id
                    .push(new_node.unique_id)
            }
        })
    }

    push_node(new_node);

    Ok("1".to_string())
}

#[derive(Error, Debug)]
enum DeleteNodeError {
    #[error(transparent)]
    ParsingError(ParseIntError),
    #[error("Node was not found")]
    NodeNotFound,
}

byond_fn!(fn remove_node_astar(unique_id) {
    match remove_node(unique_id) {
        Ok(s) => Some(s),
        Err(e) => Some(format!("{e}"))
    }
});

// Replace the node with unique_id by None
// Update connected nodes as well so nothing target the removed node anymore
// Errors if no node can be found with unique_id
fn remove_node(unique_id: &str) -> Result<String, DeleteNodeError> {
    let unique_id = match unique_id.parse::<usize>() {
        Ok(id) => id,
        Err(e) => return Err(DeleteNodeError::ParsingError(e)),
    };

    let node_to_delete = match get_node(unique_id) {
        Some(Some(node)) => node,
        _ => return Err(DeleteNodeError::NodeNotFound),
    };

    for index in node_to_delete.connected_nodes_id.iter() {
        NODES.with(|nodes_ref| {
            if let Some(Some(node)) = nodes_ref.borrow_mut().get_mut(*index) {
                Rc::get_mut(node)
                    .unwrap()
                    .connected_nodes_id
                    .retain(|index| index != &node_to_delete.unique_id);
            }
        })
    }

    null_out_node(unique_id);

    Ok("1".to_string())
}

#[derive(Error, Debug)]
enum AstarError {
    #[error("Starting node not found")]
    StartNodeNotFound,
    #[error("Goal node not found")]
    GoalNodeNotFound,
    #[error("No path found")]
    NoPath,
}

byond_fn!(fn generate_path_astar(start_node_id, goal_node_id) {
    if let (Ok(start_node_id), Ok(goal_node_id)) = (start_node_id.parse::<usize>(), goal_node_id.parse::<usize>()) {
        match generate_path(start_node_id, goal_node_id) {
            Ok(vector) => Some(match serde_json::to_string(&vector) {
                Ok(s) => s,
                Err(_) => "Cannot serialize path".to_string(),
            }),
            Err(e) => Some(format!("{e}"))
        }
    }
    else {
        Some("Invalid arguments".to_string())
    }
});

// Compute the shortest path between start node and goal node using A*
fn generate_path(start_node_id: usize, goal_node_id: usize) -> Result<Vec<usize>, AstarError> {
    let start_node = match get_node(start_node_id) {
        Some(Some(node)) => node,
        _ => return Err(AstarError::StartNodeNotFound),
    };

    let goal_node = match get_node(goal_node_id) {
        Some(Some(node)) => node,
        _ => return Err(AstarError::GoalNodeNotFound),
    };

    if goal_node.z != start_node.z {
        return Err(AstarError::NoPath);
    }

    // Compute the shortest path between start node and goal node using A*
    let path = astar(
        &start_node,
        |node| node.successors(),
        |node| node.distance(&goal_node),
        |node| node.distance(&goal_node) == 0,
    );

    // Extract a vector of node container from the path variable. Errors if no path was found
    let path = match path {
        None => return Err(AstarError::NoPath),
        Some(path) => path.0,
    };

    // Map every nodecontainer to the unique id of its node, so it can be sent to byond
    Ok(path
        .into_iter()
        .map(|node| node.unique_id)
        .rev() // Reverse iterator so it is easy to pop the list in byond
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register() {
        let json = std::fs::read_to_string("tests/rsc/ai_nodes_info.json").unwrap();
        assert!(register_nodes(&json).is_ok());
        assert!(NODES.with(|nodes_ref| nodes_ref.borrow().len() != 0))
    }

    #[test]
    fn test_add_node() {
        let json = std::fs::read_to_string("tests/rsc/ai_nodes_info.json").unwrap();
        assert!(register_nodes(&json).is_ok());
        let mut node_to_add = NODES
            .with(|nodes_ref| nodes_ref.borrow().get(18).cloned())
            .unwrap()
            .unwrap()
            .as_ref()
            .clone();
        let initial_len = NODES.with(|nodes_ref| nodes_ref.borrow().len());

        node_to_add.unique_id = initial_len;
        assert!(add_node(&serde_json::to_string(&node_to_add).unwrap()).is_ok());
        assert!(initial_len == NODES.with(|nodes_ref| nodes_ref.borrow().len() - 1));
    }

    #[test]
    fn test_remove_node() {
        let json = std::fs::read_to_string("tests/rsc/ai_nodes_info.json").unwrap();
        assert!(register_nodes(&json).is_ok());

        assert!(remove_node("11").is_ok());
        assert!(NODES.with(|nodes_ref| nodes_ref.borrow().get(11).unwrap().is_none()))
    }

    #[test]
    fn test_pathfinding() {
        let json = std::fs::read_to_string("tests/rsc/ai_nodes_info.json").unwrap();
        assert!(register_nodes(&json).is_ok());
        assert!(generate_path(10, 25).is_ok());
    }
}
