use serde::Deserialize;
use mut_static::MutStatic;
use lazy_static::lazy_static;
use pathfinding::prelude::astar;
use num::integer::sqrt;

#[derive(Deserialize, Default, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Node {
    unique_id: usize,
    x: usize,
    y: usize,
    connected_nodes_id: Vec<usize>
}

impl Node {
    fn successors(&self) -> Vec<(Self, usize)> {
        self.connected_nodes_id.iter().map(|id| {
            if let Some(connected_node) = NODES.read().unwrap().get(*id) {
                Some(connected_node.clone())
            }
            else {
                None
            }
        }).filter_map(|node| node).map(|node| (self.distance(&node), node)).map(|(distance, node)| (node.clone(), distance)).collect()
    }

    fn distance(&self, other:&Self) -> usize {
        sqrt((self.x - other.x).pow(2) + (self.y - other.y).pow(2))
    }
}

lazy_static! {
    static ref NODES:MutStatic<Vec<Node>> = MutStatic::new();
}

pub enum RegisteringNodesError {
    SerdeError(serde_json::Error),
    MutexError(mut_static::Error),
}

impl From<serde_json::Error> for RegisteringNodesError {
    fn from(error:serde_json::Error) -> Self {
        RegisteringNodesError::SerdeError(error)
    }
}
impl From<mut_static::Error> for RegisteringNodesError {
    fn from(error:mut_static::Error) -> Self {
        RegisteringNodesError::MutexError(error)
    }
}

byond_fn!(fn register_nodes(json) {
    match register_nodes_(json) {
        Ok(s) => Some(s),
        Err(e) => Some(match e {
            RegisteringNodesError::MutexError(_) => "Mutex error".to_string(),
            RegisteringNodesError::SerdeError(_) => "Parsing error".to_string()
        })
    }
});

fn register_nodes_(json: &str) -> Result<String, RegisteringNodesError>{
    let nodes:Vec<Node> = serde_json::from_str(json)?;
    if let Err(e) = NODES.set(nodes) {
        return Err(RegisteringNodesError::MutexError(e));
    }
    Ok("1".to_string())
}

byond_fn!(fn astar_generate_path(start_id, goal_id) {
    if let (Ok(start_id), Ok(goal_id)) = (start_id.parse::<usize>(), goal_id.parse::<usize>()) {
        match astar_generate_path_(start_id, goal_id) {
            Ok(vector) => Some(match serde_json::to_string(&vector) {
                Ok(s) => s,
                Err(_) => "Cannot serialize path".to_string(),
            }),
            Err(e) => Some(match e {
                AstarError::MutexError(_) => "Mutex error".to_string(),
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

pub enum AstarError {
    MutexError(mut_static::Error),
    StartNodeNotFound,
    GoalNodeNotFound,
    NoPathFound,
}

impl From<mut_static::Error> for AstarError {
    fn from(error:mut_static::Error) -> Self {
        AstarError::MutexError(error)
    }
}

fn astar_generate_path_(start_id: usize, goal_id: usize) -> Result<Vec<usize>, AstarError> {
    let nodes = NODES.read()?;

    let start_node = nodes.get(start_id);
    if start_node.is_none() {
        return Err(AstarError::StartNodeNotFound);
    }

    let goal_node = nodes.get(goal_id);
    if goal_node.is_none() {
        return Err(AstarError::GoalNodeNotFound);
    }

    let path = astar(start_node.unwrap(), |node| node.successors(), |node| node.distance(goal_node.unwrap()), |node| node == goal_node.unwrap());
    if path.is_none() {
        return Err(AstarError::NoPathFound);
    }

    let (path, _) = path.unwrap();
    Ok(path.into_iter().map(|node| node.unique_id).collect())
}
