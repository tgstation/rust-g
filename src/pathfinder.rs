use num::integer::sqrt;
use pathfinding::prelude::astar;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::hash::Hash;
use std::num::ParseIntError;
use std::sync::{Arc, RwLock};
use std::usize::MAX;

// static mut are not thread safe, and manipulating one is only possible behind unsafe tag
// However, most of the unsafe call should be thread safe because nodes inside this vec are behind a RwLock
static mut NODES: Vec<NodeContainer> = Vec::new();

// Container for a node. Exist mainly to be able to implement Hash, which is not implemented for RefCell
#[derive(Clone)]
struct NodeContainer {
    node: Arc<RwLock<Option<Node>>>,
}

impl Eq for NodeContainer {}

impl PartialEq for NodeContainer {
    fn eq(&self, other: &Self) -> bool {
        self.node
            .read()
            .unwrap()
            .eq(other.node.read().unwrap().borrow())
    }
}

impl Hash for NodeContainer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node.read().unwrap().hash(state)
    }
}

impl NodeContainer {
    fn new(node: Node) -> Self {
        let node = Arc::new(RwLock::new(Some(node)));
        NodeContainer { node }
    }

    fn new_empty() -> Self {
        let node = Arc::new(RwLock::new(None));
        NodeContainer { node }
    }
}

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
    fn successors(&self) -> Vec<(NodeContainer, usize)> {
        self.connected_nodes_id
            .iter()
            .map(|index| unsafe { NODES.get(*index) })
            .flatten()
            .map(|node_container| {
                (
                    // We create a new Arc to be able to share ownership without having to clone the node itself
                    node_container.clone(),
                    self.distance(node_container.node.read().unwrap().as_ref().unwrap()),
                )
            })
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

enum RegisteringNodesError {
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

byond_fn!(fn register_nodes_astar(json) {
    match register_nodes(json) {
        Ok(s) => Some(s),
        Err(e) => Some(format!("{e}"))
    }
});

// Builds a list of nodes from a json file.
// Errors if the input list of nodes is not correctly indexed. Each node should have for unique id its position in the list, with the first unique-id being 0.
// Memory safety not guaranteed in multithread environment
fn register_nodes(json: &str) -> Result<String, RegisteringNodesError> {
    let nodes: Vec<Node> = serde_json::from_str(json)?;
    if nodes
        .iter()
        .enumerate()
        .filter(|(i, node)| i != &node.unique_id)
        .count()
        != 0
    {
        return Err(RegisteringNodesError::NodesNotCorrectlyIndexed);
    }

    nodes
        .into_iter()
        .for_each(|node| unsafe { NODES.push(NodeContainer::new(node)) });

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
    if new_node.unique_id != unsafe { NODES.len() } {
        return Err(RegisteringNodesError::NodesNotCorrectlyIndexed);
    }

    // Make sure every connection we have we other nodes is 2 ways
    new_node.connected_nodes_id.iter().for_each(|index| {
        if let Some(node_container) = unsafe { NODES.get_mut(*index) } {
            if let Some(node) = node_container.node.write().unwrap().as_mut() {
                node.connected_nodes_id.push(new_node.unique_id)
            }
        };
    });

    unsafe {
        NODES.push(NodeContainer::new(new_node));
    }

    Ok("1".to_string())
}

enum DeleteNodeError {
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
    let unique_id = unique_id.parse::<usize>()?;

    {
        let node_to_delete_container = unsafe { NODES.get(unique_id) };

        let node_to_delete_ref = match node_to_delete_container {
            None => return Err(DeleteNodeError::NodeNotFound),
            Some(node_container) => node_container.node.write().unwrap(),
        };

        let node_to_delete = match node_to_delete_ref.as_ref() {
            None => return Err(DeleteNodeError::NodeNotFound),
            Some(node) => node,
        };

        // Erase all links to the removed node
        node_to_delete.connected_nodes_id.iter().for_each(|i| {
            if let Some(node_container) = unsafe { NODES.get_mut(*i) } {
                if let Some(node) = node_container.node.write().unwrap().as_mut() {
                    node.connected_nodes_id
                        .retain(|index| index != &node_to_delete.unique_id);
                }
            }
        });
    } // We need to drop everything before set the removed node to None. This is to ensure memory safety

    unsafe { NODES[unique_id] = NodeContainer::new_empty() }

    Ok("1".to_string())
}

enum AstarError {
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
        match generate_path(start_node_id, goal_node_id) {
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

// Compute the shortest path between start node and goal node using A*
fn generate_path(start_node_id: usize, goal_node_id: usize) -> Result<Vec<usize>, AstarError> {
    // Get the container of the start node. Errors if the start node cannot be found or is none
    let start_node_container = match unsafe { NODES.get(start_node_id) } {
        None => return Err(AstarError::StartNodeNotFound),
        Some(node_container) => match node_container.node.read().unwrap().as_ref() {
            None => return Err(AstarError::StartNodeNotFound),
            Some(_) => node_container,
        },
    };

    // Get a reference to the goal node. Errors if the goal node cannot be found or is none
    let goal_node_container = match unsafe { NODES.get(goal_node_id) } {
        None => return Err(AstarError::GoalNodeNotFound),
        Some(node_container) => node_container.node.read().unwrap(),
    };

    let goal_node = match goal_node_container.as_ref() {
        None => return Err(AstarError::GoalNodeNotFound),
        Some(node) => node,
    };

    if goal_node.z
        != start_node_container
            .node
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .z
    {
        return Err(AstarError::NoPathFound);
    }

    // Compute the shortest path between start node and goal node using A*
    let path = astar(
        start_node_container,
        |node_container| {
            if let Some(node) = node_container.node.read().unwrap().as_ref() {
                node.successors()
            } else {
                Vec::new()
            }
        },
        |node_container| {
            if let Some(node) = node_container.node.read().unwrap().as_ref() {
                node.distance(goal_node)
            } else {
                MAX
            }
        },
        |node_container| {
            if let Some(node) = node_container.node.read().unwrap().as_ref() {
                node.distance(goal_node) == 0
            } else {
                false
            }
        },
    );

    // Extract a vector of node container from the path variable. Errors if no path was found
    let path = match path {
        None => return Err(AstarError::NoPathFound),
        Some(path) => path.0,
    };

    // Map every nodecontainer to the unique id of its node, so it can be sent to byond
    Ok(path
        .into_iter()
        .map(|node| node.node.read().unwrap().as_ref().unwrap().unique_id)
        .collect())
}
