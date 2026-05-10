use crate::prelude::*;
use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub node_type: NodeType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub position: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Input,
    Output,
    Processor,
    Transform,
    Filter,
    Effect,
    Analysis,
    Conversion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: Uuid,
    pub source_node: Uuid,
    pub target_node: Uuid,
    pub source_port: String,
    pub target_port: String,
    pub data_type: DataType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Audio,
    Image,
    Video,
    Raw,
    Metadata,
    Control,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingGraph {
    pub id: Uuid,
    pub name: String,
    pub nodes: HashMap<Uuid, Node>,
    pub edges: HashMap<Uuid, Edge>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExecution {
    pub graph_id: Uuid,
    pub status: ExecutionStatus,
    pub node_results: HashMap<Uuid, GraphResult>,
    pub execution_order: Vec<Uuid>,
    pub current_node: Option<Uuid>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Validating,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

pub struct GraphManager {
    graphs: Arc<RwLock<HashMap<Uuid, ProcessingGraph>>>,
    executions: Arc<RwLock<HashMap<Uuid, GraphExecution>>>,
}

impl GraphManager {
    pub fn new() -> Self {
        Self {
            graphs: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_graph(&self, graph: ProcessingGraph) -> Result<()> {
        let mut graphs = self.graphs.write();
        graphs.insert(graph.id, graph);
        Ok(())
    }

    pub fn add_node(&self, graph_id: Uuid, node: Node) -> Result<()> {
        let mut graphs = self.graphs.write();
        if let Some(graph) = graphs.get_mut(&graph_id) {
            graph.nodes.insert(node.id, node);
            Ok(())
        } else {
            Err(CoreError::Task(format!("Graph {} not found", graph_id)))
        }
    }

    pub fn remove_node(&self, graph_id: Uuid, node_id: Uuid) -> Result<()> {
        let mut graphs = self.graphs.write();
        if let Some(graph) = graphs.get_mut(&graph_id) {
            graph.nodes.remove(&node_id);
            graph.edges.retain(|_, edge| edge.source_node != node_id && edge.target_node != node_id);
            Ok(())
        } else {
            Err(CoreError::Task(format!("Graph {} not found", graph_id)))
        }
    }

    pub fn add_edge(&self, graph_id: Uuid, edge: Edge) -> Result<()> {
        let mut graphs = self.graphs.write();
        if let Some(graph) = graphs.get_mut(&graph_id) {
            if !graph.nodes.contains_key(&edge.source_node) {
                return Err(CoreError::Task(format!("Source node {} not found", edge.source_node)));
            }
            if !graph.nodes.contains_key(&edge.target_node) {
                return Err(CoreError::Task(format!("Target node {} not found", edge.target_node)));
            }
            graph.edges.insert(edge.id, edge);
            Ok(())
        } else {
            Err(CoreError::Task(format!("Graph {} not found", graph_id)))
        }
    }

    pub fn remove_edge(&self, graph_id: Uuid, edge_id: Uuid) -> Result<()> {
        let mut graphs = self.graphs.write();
        if let Some(graph) = graphs.get_mut(&graph_id) {
            graph.edges.remove(&edge_id);
            Ok(())
        } else {
            Err(CoreError::Task(format!("Graph {} not found", graph_id)))
        }
    }

    pub fn validate_graph(&self, graph_id: Uuid) -> Result<()> {
        let graph = {
            let graphs = self.graphs.read();
            graphs.get(&graph_id).cloned()
                .ok_or_else(|| CoreError::Task(format!("Graph {} not found", graph_id)))?
        };

        self.check_cycles(&graph)?;
        self.check_connections(&graph)?;
        self.check_data_flow(&graph)?;

        Ok(())
    }

    fn check_cycles(&self, graph: &ProcessingGraph) -> Result<()> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for node_id in graph.nodes.keys() {
            if !visited.contains(node_id) {
                if self.has_cycle_dfs(
                    node_id,
                    &graph,
                    &mut visited,
                    &mut recursion_stack,
                )? {
                    return Err(CoreError::Task("Cycle detected in graph".to_string()));
                }
            }
        }

        Ok(())
    }

    fn has_cycle_dfs(
        &self,
        node_id: &Uuid,
        graph: &ProcessingGraph,
        visited: &mut HashSet<Uuid>,
        recursion_stack: &mut HashSet<Uuid>,
    ) -> Result<bool> {
        visited.insert(*node_id);
        recursion_stack.insert(*node_id);

        for edge in graph.edges.values() {
            if edge.source_node == *node_id {
                if !visited.contains(&edge.target_node) {
                    if self.has_cycle_dfs(&edge.target_node, graph, visited, recursion_stack)? {
                        return Ok(true);
                    }
                } else if recursion_stack.contains(&edge.target_node) {
                    return Ok(true);
                }
            }
        }

        recursion_stack.remove(node_id);
        Ok(false)
    }

    fn check_connections(&self, graph: &ProcessingGraph) -> Result<()> {
        for edge in graph.edges.values() {
            if !graph.nodes.contains_key(&edge.source_node) {
                return Err(CoreError::Task(format!(
                    "Edge references non-existent source node {}",
                    edge.source_node
                )));
            }
            if !graph.nodes.contains_key(&edge.target_node) {
                return Err(CoreError::Task(format!(
                    "Edge references non-existent target node {}",
                    edge.target_node
                )));
            }
        }
        Ok(())
    }

    fn check_data_flow(&self, graph: &ProcessingGraph) -> Result<()> {
        let mut input_nodes = HashSet::new();
        let mut output_nodes = HashSet::new();

        for node in graph.nodes.values() {
            match node.node_type {
                NodeType::Input => input_nodes.insert(node.id),
                NodeType::Output => output_nodes.insert(node.id),
                _ => false,
            };
        }

        if input_nodes.is_empty() {
            return Err(CoreError::Task("Graph has no input nodes".to_string()));
        }

        if output_nodes.is_empty() {
            return Err(CoreError::Task("Graph has no output nodes".to_string()));
        }

        Ok(())
    }

    pub async fn execute_graph(&self, graph_id: Uuid) -> Result<Uuid> {
        self.validate_graph(graph_id)?;

        let graph = {
            let graphs = self.graphs.read();
            graphs.get(&graph_id).cloned()
                .ok_or_else(|| CoreError::Task(format!("Graph {} not found", graph_id)))?
        };

        let execution_id = Uuid::new_v4();
        let execution = GraphExecution {
            graph_id,
            status: ExecutionStatus::Running,
            node_results: HashMap::new(),
            execution_order: self.calculate_execution_order(&graph)?,
            current_node: None,
            error: None,
        };

        {
            let mut executions = self.executions.write();
            executions.insert(execution_id, execution);
        }

        let executions = self.executions.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::process_graph(graph, execution_id, executions).await {
                tracing::error!("Graph execution failed: {}", e);
            }
        });

        Ok(execution_id)
    }

    fn calculate_execution_order(&self, graph: &ProcessingGraph) -> Result<Vec<Uuid>> {
        let mut visited = HashSet::new();
        let mut order = Vec::new();
        let mut temp = HashSet::new();

        for node_id in graph.nodes.keys() {
            if !visited.contains(node_id) {
                self.topological_sort(node_id, graph, &mut visited, &mut temp, &mut order)?;
            }
        }

        Ok(order)
    }

    fn topological_sort(
        &self,
        node_id: &Uuid,
        graph: &ProcessingGraph,
        visited: &mut HashSet<Uuid>,
        temp: &mut HashSet<Uuid>,
        order: &mut Vec<Uuid>,
    ) -> Result<()> {
        if temp.contains(node_id) {
            return Err(CoreError::Task("Cycle detected during topological sort".to_string()));
        }

        if visited.contains(node_id) {
            return Ok(());
        }

        temp.insert(*node_id);

        for edge in graph.edges.values() {
            if edge.target_node == *node_id && !visited.contains(&edge.source_node) {
                self.topological_sort(&edge.source_node, graph, visited, temp, order)?;
            }
        }

        temp.remove(node_id);
        visited.insert(*node_id);
        order.push(*node_id);

        Ok(())
    }

    async fn process_graph(
        graph: ProcessingGraph,
        execution_id: Uuid,
        executions: Arc<RwLock<HashMap<Uuid, GraphExecution>>>,
    ) -> Result<()> {
        let execution_order = {
            let exec = executions.read();
            exec.get(&execution_id)
                .map(|e| e.execution_order.clone())
                .unwrap_or_default()
        };

        for node_id in execution_order {
            {
                let mut exec = executions.write();
                if let Some(execution) = exec.get_mut(&execution_id) {
                    execution.current_node = Some(node_id);
                }
            }

            let start_time = std::time::Instant::now();
            let result = Self::execute_node(&graph, &node_id).await;
            let execution_time = start_time.elapsed().as_millis() as u64;

            let graph_result = GraphResult {
                success: result.is_ok(),
                data: result.ok(),
                error: result.err().map(|e| e.to_string()),
                execution_time_ms: execution_time,
            };

            {
                let mut exec = executions.write();
                if let Some(execution) = exec.get_mut(&execution_id) {
                    execution.node_results.insert(node_id, graph_result);
                }
            }

            if let Err(e) = result {
                {
                    let mut exec = executions.write();
                    if let Some(execution) = exec.get_mut(&execution_id) {
                        execution.status = ExecutionStatus::Failed;
                        execution.error = Some(e.to_string());
                    }
                }
                return Err(e);
            }
        }

        {
            let mut exec = executions.write();
            if let Some(execution) = exec.get_mut(&execution_id) {
                execution.status = ExecutionStatus::Completed;
                execution.current_node = None;
            }
        }

        Ok(())
    }

    async fn execute_node(graph: &ProcessingGraph, node_id: &Uuid) -> Result<serde_json::Value> {
        let node = graph.nodes.get(node_id)
            .ok_or_else(|| CoreError::Task(format!("Node {} not found", node_id)))?;

        match node.node_type {
            NodeType::Input => {
                Ok(serde_json::json!({"type": "input", "data": "raw_input"}))
            },
            NodeType::Processor => {
                Ok(serde_json::json!({"type": "processed", "node": node.name}))
            },
            NodeType::Transform => {
                Ok(serde_json::json!({"type": "transformed", "node": node.name}))
            },
            NodeType::Filter => {
                Ok(serde_json::json!({"type": "filtered", "node": node.name}))
            },
            NodeType::Effect => {
                Ok(serde_json::json!({"type": "effect_applied", "node": node.name}))
            },
            NodeType::Analysis => {
                Ok(serde_json::json!({"type": "analyzed", "node": node.name}))
            },
            NodeType::Conversion => {
                Ok(serde_json::json!({"type": "converted", "node": node.name}))
            },
            NodeType::Output => {
                Ok(serde_json::json!({"type": "output", "data": "final_result"}))
            },
        }
    }

    pub fn get_execution(&self, execution_id: Uuid) -> Option<GraphExecution> {
        let executions = self.executions.read();
        executions.get(&execution_id).cloned()
    }

    pub fn get_graph(&self, graph_id: Uuid) -> Option<ProcessingGraph> {
        let graphs = self.graphs.read();
        graphs.get(&graph_id).cloned()
    }

    pub fn list_graphs(&self) -> Vec<ProcessingGraph> {
        let graphs = self.graphs.read();
        graphs.values().cloned().collect()
    }

    pub fn cancel_execution(&self, execution_id: Uuid) -> Result<()> {
        let mut executions = self.executions.write();
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.status = ExecutionStatus::Cancelled;
            Ok(())
        } else {
            Err(CoreError::Task(format!("Execution {} not found", execution_id)))
        }
    }
}
