use std::collections::HashMap;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use data_model::{ComputeGraphCode, GraphVersion};
use indexify_utils::get_epoch_time_in_ms;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, ToSchema)]
pub struct IndexifyAPIError {
    status_code: StatusCode,
    message: String,
}

impl IndexifyAPIError {
    pub fn new(status_code: StatusCode, message: &str) -> Self {
        Self {
            status_code,
            message: message.to_string(),
        }
    }

    pub fn _bad_request(e: &str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, e)
    }

    pub fn internal_error(e: anyhow::Error) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string().as_str())
    }

    pub fn internal_error_str(e: &str) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, e)
    }

    pub fn not_found(message: &str) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn bad_request(message: &str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }
}

impl IntoResponse for IndexifyAPIError {
    fn into_response(self) -> Response {
        tracing::error!("API Error: {} - {}", self.status_code, self.message);
        (self.status_code, self.message).into_response()
    }
}

impl From<serde_json::Error> for IndexifyAPIError {
    fn from(e: serde_json::Error) -> Self {
        Self::bad_request(&e.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListParams {
    pub limit: Option<usize>,
    pub cursor: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Namespace {
    name: String,
    created_at: u64,
}

impl From<data_model::Namespace> for Namespace {
    fn from(namespace: data_model::Namespace) -> Self {
        Self {
            name: namespace.name,
            created_at: namespace.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NamespaceList {
    pub namespaces: Vec<Namespace>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ComputeFn {
    pub name: String,
    pub fn_name: String,
    pub description: String,
    pub reducer: bool,
}

impl From<&ComputeFn> for data_model::ComputeFn {
    fn from(val: &ComputeFn) -> Self {
        data_model::ComputeFn {
            name: val.name.clone(),
            fn_name: val.fn_name.clone(),
            description: val.description.clone(),
            placement_constraints: Default::default(),
            reducer: val.reducer,
        }
    }
}

impl From<ComputeFn> for data_model::ComputeFn {
    fn from(val: ComputeFn) -> Self {
        data_model::ComputeFn {
            name: val.name.clone(),
            fn_name: val.fn_name.clone(),
            description: val.description.clone(),
            placement_constraints: Default::default(),
            reducer: val.reducer,
        }
    }
}

impl From<data_model::ComputeFn> for ComputeFn {
    fn from(c: data_model::ComputeFn) -> Self {
        Self {
            name: c.name,
            fn_name: c.fn_name,
            description: c.description,
            reducer: c.reducer,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DynamicRouter {
    pub name: String,
    pub source_fn: String,
    pub description: String,
    pub target_fns: Vec<String>,
}

impl From<DynamicRouter> for data_model::DynamicEdgeRouter {
    fn from(val: DynamicRouter) -> Self {
        data_model::DynamicEdgeRouter {
            name: val.name.clone(),
            source_fn: val.source_fn.clone(),
            description: val.description.clone(),
            target_functions: val.target_fns.clone(),
        }
    }
}

impl From<data_model::DynamicEdgeRouter> for DynamicRouter {
    fn from(d: data_model::DynamicEdgeRouter) -> Self {
        Self {
            name: d.name,
            source_fn: d.source_fn,
            description: d.description,
            target_fns: d.target_functions,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub enum Node {
    #[serde(rename = "dynamic_router")]
    DynamicRouter(DynamicRouter),
    #[serde(rename = "compute_fn")]
    ComputeFn(ComputeFn),
}

impl Node {
    pub fn name(&self) -> String {
        match self {
            Node::DynamicRouter(d) => d.name.clone(),
            Node::ComputeFn(c) => c.name.clone(),
        }
    }
}

impl From<Node> for data_model::Node {
    fn from(val: Node) -> Self {
        match val {
            Node::DynamicRouter(d) => data_model::Node::Router(d.into()),
            Node::ComputeFn(c) => data_model::Node::Compute(c.into()),
        }
    }
}

impl From<data_model::Node> for Node {
    fn from(node: data_model::Node) -> Self {
        match node {
            data_model::Node::Router(d) => Node::DynamicRouter(d.into()),
            data_model::Node::Compute(c) => Node::ComputeFn(c.into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ComputeGraph {
    pub name: String,
    pub namespace: String,
    pub description: String,
    pub start_node: Node,
    pub nodes: HashMap<String, Node>,
    pub edges: HashMap<String, Vec<String>>,
    #[serde(default = "get_epoch_time_in_ms")]
    pub created_at: u64,
}

impl ComputeGraph {
    pub fn into_data_model(
        self,
        code_path: &str,
        sha256_hash: &str,
        size: u64,
    ) -> Result<data_model::ComputeGraph, IndexifyAPIError> {
        let mut nodes = HashMap::new();
        for (name, node) in self.nodes {
            nodes.insert(name, node.into());
        }
        let start_fn: data_model::Node = self.start_node.into();
        let compute_graph = data_model::ComputeGraph {
            name: self.name,
            namespace: self.namespace,
            description: self.description,
            start_fn,
            version: Default::default(),
            code: ComputeGraphCode {
                sha256_hash: sha256_hash.to_string(),
                size,
                path: code_path.to_string(),
            },
            nodes,
            edges: self.edges.clone(),
            created_at: 0,
        };
        Ok(compute_graph)
    }
}

impl From<data_model::ComputeGraph> for ComputeGraph {
    fn from(compute_graph: data_model::ComputeGraph) -> Self {
        let start_fn = match compute_graph.start_fn {
            data_model::Node::Router(d) => Node::DynamicRouter(d.into()),
            data_model::Node::Compute(c) => Node::ComputeFn(c.into()),
        };
        let mut nodes = HashMap::new();
        for (k, v) in compute_graph.nodes.into_iter() {
            nodes.insert(k, v.into());
        }
        Self {
            name: compute_graph.name,
            namespace: compute_graph.namespace,
            description: compute_graph.description,
            start_node: start_fn,
            nodes,
            edges: compute_graph.edges,
            created_at: compute_graph.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateNamespace {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ComputeGraphsList {
    pub compute_graphs: Vec<ComputeGraph>,
    pub cursor: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DataObject {
    pub id: String,
    pub payload_size: u64,
    pub payload_sha_256: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryParams {
    pub input_id: Option<String>,
    pub on_graph_end: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphOutputNotification {
    pub output_id: String,
    pub compute_graph: String,
    pub fn_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateNamespaceResponse {
    pub name: Namespace,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GraphInvocations {
    pub invocations: Vec<DataObject>,
    pub cursor: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GraphInputJson {
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GraphInputFile {
    // file:///s3://bucket/key
    // file:///data/path/to/file
    pub url: String,
    pub metadata: serde_json::Value,
    pub sha_256: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InvocationResult {
    pub outputs: HashMap<String, Vec<DataObject>>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum TaskOutcome {
    Unknown,
    Success,
    Failure,
}

impl From<data_model::TaskOutcome> for TaskOutcome {
    fn from(outcome: data_model::TaskOutcome) -> Self {
        match outcome {
            data_model::TaskOutcome::Unknown => TaskOutcome::Unknown,
            data_model::TaskOutcome::Success => TaskOutcome::Success,
            data_model::TaskOutcome::Failure => TaskOutcome::Failure,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Task {
    pub id: String,
    pub namespace: String,
    pub compute_fn: String,
    pub compute_graph: String,
    pub invocation_id: String,
    pub input_key: String,
    pub outcome: TaskOutcome,
    pub reducer_output_id: Option<String>,
    pub graph_version: GraphVersion,
}

impl From<data_model::Task> for Task {
    fn from(task: data_model::Task) -> Self {
        Self {
            id: task.id.to_string(),
            namespace: task.namespace,
            compute_fn: task.compute_fn_name,
            compute_graph: task.compute_graph_name,
            invocation_id: task.invocation_id,
            input_key: task.input_node_output_key,
            outcome: task.outcome.into(),
            reducer_output_id: task.reducer_output_id,
            graph_version: task.graph_version,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Tasks {
    pub tasks: Vec<Task>,
    pub cursor: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FnOutput {
    pub compute_fn: String,
    pub id: String,
}

impl From<data_model::NodeOutput> for FnOutput {
    fn from(output: data_model::NodeOutput) -> Self {
        Self {
            compute_fn: output.compute_fn_name,
            id: output.id.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FnOutputs {
    pub outputs: Vec<FnOutput>,
    pub cursor: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InvocationId {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExecutorMetadata {
    pub address: String,
    pub runner_name: String,
    pub labels: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InvocationQueryParams {
    pub block_until_finish: Option<bool>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_compute_graph_deserialization() {
        // Don't delete this. It makes it easier
        // to test the deserialization of the ComputeGraph struct
        // from the python side
        let json = r#"{"name":"test","description":"test","start_node":{"compute_fn":{"name":"extractor_a","fn_name":"extractor_a","description":"Random description of extractor_a", "reducer": false}},"nodes":{"extractor_a":{"compute_fn":{"name":"extractor_a","fn_name":"extractor_a","description":"Random description of extractor_a", "reducer": false}},"extractor_b":{"compute_fn":{"name":"extractor_b","fn_name":"extractor_b","description":"", "reducer": false}},"extractor_c":{"compute_fn":{"name":"extractor_c","fn_name":"extractor_c","description":"", "reducer": false}}},"edges":{"extractor_a":["extractor_b"],"extractor_b":["extractor_c"]}}"#;
        let mut json_value: serde_json::Value = serde_json::from_str(json).unwrap();
        json_value["namespace"] = serde_json::Value::String("test".to_string());
        let _: super::ComputeGraph = serde_json::from_value(json_value).unwrap();
    }

    #[test]
    fn test_compute_graph_with_router_deserialization() {
        let json = r#"{"name":"graph_a_router","description":"description of graph_a","start_node":{"compute_fn":{"name":"extractor_a","fn_name":"extractor_a","description":"Random description of extractor_a", "reducer": false}},"nodes":{"extractor_a":{"compute_fn":{"name":"extractor_a","fn_name":"extractor_a","description":"Random description of extractor_a", "reducer": false}},"router_x":{"dynamic_router":{"name":"router_x","description":"","source_fn":"router_x","target_fns":["extractor_y","extractor_z"], "reducer": false}},"extractor_y":{"compute_fn":{"name":"extractor_y","fn_name":"extractor_y","description":"", "reducer": false}},"extractor_z":{"compute_fn":{"name":"extractor_z","fn_name":"extractor_z","description":"", "reducer": false}},"extractor_c":{"compute_fn":{"name":"extractor_c","fn_name":"extractor_c","description":"", "reducer": false}}},"edges":{"extractor_a":["router_x"],"extractor_y":["extractor_c"],"extractor_z":["extractor_c"]}}"#;
        let mut json_value: serde_json::Value = serde_json::from_str(json).unwrap();
        json_value["namespace"] = serde_json::Value::String("test".to_string());
        let _: super::ComputeGraph = serde_json::from_value(json_value).unwrap();
    }
}
