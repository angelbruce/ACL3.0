use chrono::{NaiveDateTime, DateTime, Utc};
use diesel::prelude::*;
// use diesel::sql_types::*;
use serde::{Serialize, Deserialize};
// use uuid::Uuid;



#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

// #[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
// #[diesel(table_name = crate::schema::users)]
// #[diesel(check_for_backend(diesel::pg::Pg))]
// pub struct NewUser {
//     pub username: String,
//     pub email: String,
//     pub password_hash: String,
//     pub role: String,
//     pub department_id: Option<i64>,
// }

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::departments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Department {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::departments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDepartment {
    pub name: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Permission {
    pub id: i64,
    pub menu_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
}

// #[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
// #[diesel(table_name = crate::schema::permissions)]
// #[diesel(check_for_backend(diesel::pg::Pg))]
// pub struct NewPermission {
//     pub name: String,
//     pub description: Option<String>,
//     pub resource: String,
//     pub action: String,
// }

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::role_permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RolePermission {
    pub id: i64,
    pub role_id: i64,
    pub permission_id: i64,
    pub created_at: NaiveDateTime,
}

// #[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
// #[diesel(table_name = crate::schema::role_permissions)]
// #[diesel(check_for_backend(diesel::pg::Pg))]
// pub struct NewRolePermission {
//     pub role: String,
//     pub permission_id: i64,
// }

#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewRole {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub password: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<String>,
    pub department_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: String,
    pub department_id: Option<i64>,
    pub department_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub department_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

// #[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
// #[diesel(table_name = crate::schema::mcp_providers)]
// #[diesel(check_for_backend(diesel::pg::Pg))]
// pub struct McpProvider {
//     pub id: i64,
//     pub name: String,
//     pub description: Option<String>,
//     pub url: String,
//     pub api_key: Option<String>,
//     pub enabled: bool,
//     pub created_at: NaiveDateTime,
//     pub updated_at: NaiveDateTime,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
// #[diesel(table_name = crate::schema::mcp_providers)]
// #[diesel(check_for_backend(diesel::pg::Pg))]
// pub struct NewMcpProvider {
//     pub name: String,
//     pub description: Option<String>,
//     pub url: String,
//     pub api_key: Option<String>,
//     pub enabled: bool,
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMcpProviderRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub api_key: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::llm_models)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LlmModel {
    pub id: i64,
    pub name: String,
    pub access_url:String,
    pub api_key:String,
    pub is_default: bool,
}


#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::llm_models)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewLlmModel {
    pub name: String,
    pub access_url: String,
    pub api_key: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLlmModelRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub description: Option<String>,
    pub agent_id: Option<i64>,
    pub model_id: Option<i64>,
    pub created_at: NaiveDateTime,
}


#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSession {
    pub user_id: i64,
    pub description: Option<String>,
    pub agent_id: Option<i64>,
    pub model_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::session_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSessionItem {
    pub session_id: i64,
    pub description: String,
    pub session_type: String,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRequest {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub response: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::workspace_files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkspaceFile {
    pub id: i64,
    pub user_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub is_directory: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}


#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::kanban_boards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KanbanBoard {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_by: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}



#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::kanban_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KanbanItem {
    pub id: i64,
    pub board_id: i64,
    pub user_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub shared_at: NaiveDateTime,
}


#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::kanban_subscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KanbanSubscription {
    pub id: i64,
    pub board_id: i64,
    pub user_id: i64,
    pub subscribed_at: NaiveDateTime,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKanbanBoardRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateKanbanBoardRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareFileRequest {
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KanbanBoardWithItems {
    pub board: KanbanBoard,
    pub items: Vec<KanbanItem>,
    pub subscriber_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribedBoard {
    pub board: KanbanBoard,
    pub items: KanbanSubscription,
}


#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::projects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Project {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub purpose: String,
    pub description: Option<String>,
    pub model_id: Option<i64>,
    pub agent_id: Option<i64>,
    pub last_accessed_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}


#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::project_files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProjectFile {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub content: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub directory: Option<String>,
    pub state: i32,
}


#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::project_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProjectMessage {
    pub id: i64,
    pub project_id: i64,
    pub role: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable,Default)]
#[diesel(table_name = crate::schema::project_container_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProjectContainerConfig {
    pub id: i64,
    pub project_id: i64,
    pub project_dir: String,
    pub published_ports: String,
    pub volumes: String,
    pub environment: String,
    pub command: String,
    pub working_dir: String,
    pub tags: String,
    pub container_name: String,
    pub cpu_usage: String,
    pub memory_usage: String,
    pub image_name: String,
    pub creator_id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}


#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::project_summaries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProjectSummary {
    pub id: i64,
    pub user_id: i64,
    pub project_id: i64,
    pub file_name: String,
    pub summary: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateProjectRequest {
    pub name: String,
    pub purpose: String,
    pub description: Option<String>,
    pub model_id: Option<i64>,
    pub agent_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub model_id: Option<i64>,
    pub agent_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateProjectFileRequest {
    pub name: String,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateProjectFileRequest {
    pub content: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddProjectMessageRequest {
    pub content: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateOrUpdateProjectSummaryRequest {
    pub file_name: String,
    pub summary: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectWithNames {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub purpose: String,
    pub description: Option<String>,
    pub model_id: Option<i64>,
    pub agent_id: Option<i64>,
    pub model_name: Option<String>,
    pub agent_name: Option<String>,
    pub last_accessed_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SharedCreateProjectRequest {
    pub name: String,
    pub purpose: String,
    pub description: Option<String>,
    pub model_id: Option<i64>,
    pub agent_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SharedUpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub model_id: Option<i64>,
    pub agent_id: Option<i64>,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct SharedFileInfo {
    pub id: i64,
    pub file_name: String,
    pub file_path: String,
    pub shared_at: String,
    pub owner_id: i64,
    pub owner_name: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct FlowDefinition {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub definition: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFlowRequest {
    pub name: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFlowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub definition: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlowInstance {
    pub id: String,
    pub flow_id: String,
    pub status: String,
    pub inputs: serde_json::Value,
    pub outputs: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteFlowRequest {
    pub inputs: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteFlowResponse {
    pub instance_id: String,
    pub status: String,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct PersonnelWithDetails {
//     pub id: i64,
//     pub username: String,
//     pub email: String,
//     pub role: String,
//     pub department_id: Option<i64>,
//     pub department_name: Option<String>,
//     pub created_at: NaiveDateTime,
// }


#[derive(Debug, Serialize, Deserialize)]
pub struct PersonnelWithDetails {
    pub personnel: Personnel,
    pub departments: Vec<Department>,
    pub roles: Vec<Role>,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token:String,
    pub refresh_token:String,
    pub user_id:i64
}


#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::personnel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Personnel {
    pub id :i64,
    pub user_id : Option<i64>,
    pub name :String,
    pub gender :Option<String>,
    pub email :Option<String>,
    pub wechat: Option<String>,
    pub phone :Option<String>,
    pub last_login_date :Option<NaiveDateTime>,
    pub created_at : NaiveDateTime,
    pub updated_at : NaiveDateTime
}


#[derive(Debug, Serialize, Deserialize, Clone, QueryableByName)]
#[diesel(table_name = crate::schema::personnel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersonnelResult {
    pub id: i64,
    pub user_id: Option<i64>,
    pub name: String,
    pub gender: Option<String>,
    pub email: Option<String>,
    pub wechat: Option<String>,
    pub phone: Option<String>,
    pub last_login_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::personnel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPersonnel {
    pub user_id: Option<i64>,
    pub name: String,
    pub gender: Option<String>,
    pub email: Option<String>,
    pub wechat: Option<String>,
    pub phone: Option<String>,
    pub last_login_date: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::personnel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersonnelUpdate {
    pub name: Option<String>,
    pub gender: Option<String>,
    pub email: Option<String>,
    pub wechat: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPTool {
    pub name:String,
    pub description: String,
    #[serde(flatten)]
    pub input_schema: serde_json::Value,
    #[serde(flatten)]
    pub output_schema: serde_json::Value,
    pub server_id: Option<i64>,

}

#[derive(Clone,Default,Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role :String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name : Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls : Option<Vec<ToolCallInfo>>,
}

#[derive(Clone,Default,Debug, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub id: Option<String>,
    pub index : Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub tool_type:Option<String>,
    pub function:Option<ToolCallFunction>,
    pub arguments:Option<serde_json::Value>,
    pub name: Option<String>,
}

#[derive(Clone,Default,Debug, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: Option<String>,
    pub arguments: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::flows)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Flow {
    pub id : i64,
    pub name : String,
    pub config : serde_json::Value,
    pub created_at : NaiveDateTime, 
}


#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::flows)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFlow {
    pub name: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::flow_runtimes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FlowRuntime {
    pub id : i64,
    pub flow_id: i64,
    pub is_over :bool,
    pub created_at : NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::flow_runtimes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFlowRuntime {
    pub flow_id: i64,
    pub is_over: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::flow_runtime_nodes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FlowRuntimeNode {
    //the id of the node in the flow runtime
    pub id : i64,
    // the id of the flow runtime, it's the instance id of the flow blueprint.
    pub flow_runtime_id: i64,
    // the id of the flow
    pub flow_id: i64,
    //the id of the node in the flow config.
    pub flow_node_id: String,
    //the action id is basicly the agent id of the node which is stands for performer.
    pub action_id: i64, 
    // the action value of the node stands for the users blueprint.
    pub action :String,
    // the prompt of the agent only used for current node.
    pub prompt :Option<String>,
    // the status of the runtime-node, it's be Running, RunningOver, Stop.
    pub status :String,
    // the next choice of the node, it's the id of the next node in the flow config.
    pub next_choice :Option<String>,
    // the created time of the node
    pub created_at : NaiveDateTime,
    // need human or not , 0 means no, 1 means yes.
    pub human : i32,
}


#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::flow_runtime_nodes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFlowRuntimeNode {
    pub flow_runtime_id: i64,
    pub flow_id: i64,
    pub action_id: i64,
    pub action: String,
    pub prompt: Option<String>,
    pub status: String,
    pub next_choice: Option<String>,
}


pub enum NodeStatus {
    Running,
    RunningOver,
    Stop,
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NodeStatus::Running => write!(f, "Running"),
            NodeStatus::RunningOver => write!(f, "RunningOver"),
            NodeStatus::Stop => write!(f, "Stop"),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vertex {
    //the id of the vertex in the flow config
    pub id : String,
    //the value of the vertex-node which is the users blueprint.
    pub value: String,
    // the paths which have to be finished if the node want to continued to move to the next node.
    pub paths:Vec<String>,
    pub r#type :String,
    //the agent id of the vertex-node, it's the id of the agent in the flow config.
    pub agent: Option<i64>,
    //node's completion degree,only 100% or 1%.
    //100% means need nodes point to this node all completed if want to continue to move to the next node.
    //1% means need can move to the next node if any node node point to this node completed.
    pub degree: Option<i64>,
    //the prompt of the agent, only for this node.
    pub prompt: Option<String>,
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Edge{
    pub id : String,
    pub value : String,
    pub src : String,
    pub target : String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowConfigModel {
    pub vertices:Vec<Vertex>,
    pub edges:Vec<Edge>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionWithNames {
    pub id: i64,
    pub user_id: i64,
    pub description: Option<String>,
    pub agent_id: Option<i64>,
    pub model_id: Option<i64>,
    pub agent_name: Option<String>,
    pub model_name: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::session_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SessionItem {
    pub id: i64,
    pub session_id: i64,
    pub description : String,
    pub session_type : String,
    pub created_at: NaiveDateTime,
}

pub enum SessionType {
    Assistant,
    User,
    ToolCall
}

impl std::fmt::Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          match self {
            SessionType::Assistant => write!(f, "Assistant"),
            SessionType::User => write!(f, "User"),
            SessionType::ToolCall => write!(f, "ToolCall"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateSessionRequest {
    pub user_id: i64,
    pub description : Option<String>,
    pub agent_id:  Option<i64>,
    pub model_id:  Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateSessionRequest {
    pub agent_id:  Option<i64>,
    pub model_id:  Option<i64>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddMessageRequest {
    pub description : String,
    pub session_type:  String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MCPToolCallResult {
    pub success : bool,
    pub content:  String,
    pub error:  Option<String>,
}



#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::mcp_servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct McpServer {
    pub id : i64,
    pub name : String,
    pub description : Option<String>,
    pub server_type : String,
    pub url : String,
    pub headers : Option<serde_json::Value>, 
    pub enabled : bool,
    pub stateless : bool,
    pub created_at:  NaiveDateTime,
    pub updated_at:  NaiveDateTime
}


#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::mcp_servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMcpServer {
    pub name: String,
    pub description: Option<String>,
    pub server_type: String,
    pub url: String,
    pub headers: Option<serde_json::Value>,
    pub enabled: bool,
    pub stateless: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::mcp_servers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct McpServerUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMcpServerRequest {
    pub name : String,
    pub description : Option<String>,
    pub server_type : String,
    pub url : String,
    pub headers : serde_json::Value,
    pub enabled : Option<bool>,
    pub stateless : Option<bool>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerWithTools {
    pub server : McpServer,
    pub tools : Vec<MCPTool>,
 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmRequest {
    pub model_id: i64,
    pub agent_id: Option<i64>,
    pub messages: Vec<ChatMessage>,
    pub project_id: Option<i64>,
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateLlmModelRequest {
    pub name: String,
    pub access_url: String,
    pub api_key: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmTool {
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: LlmToolFunction,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentDetail {
    pub id: i64,
    pub name: String,
    pub defination: Option<String>,
    pub tools: Vec<AgentTool>,
    pub skills: Vec<AgentSkill>,
    pub content_stores: Vec<ContentStoreConfig>,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::agent_tools)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentTool {
    pub id: i64,
    pub agent_id: i64,
    pub name: String,
    pub description: String,
    pub input_schema: String,
    pub output_schema: String,
    pub server_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::agent_tools)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewAgentTool {
    pub agent_id: i64,
    pub name: String,
    pub description: String,
    pub input_schema: String,
    pub output_schema: String,
    pub server_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewAgent {
    pub name: String,
    pub defination: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::agent_skills)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewAgentSkill {
    pub agent_id: i64,
    pub skill_prompt: String,
}


#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::agent_skills)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentSkill {
    pub id: i64,
    pub agent_id: i64,
    pub skill_prompt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::content_store_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ContentStoreConfig {
    pub id: i64,
    pub agent_id: i64,
    pub store_type: String,
    pub config: String,
}


#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::content_store_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewContentStoreConfig {
    pub agent_id: i64,
    pub store_type: String,
    pub config: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Agent {
    pub id: i64,
    pub name: String,
    pub defination: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAgentRequest {
    pub name: String,
    pub defination: Option<String>,
    pub tools: Option<Vec<CreateAgentTool>>,
    pub skills: Option<Vec<CreateAgentSkill>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAgentTool {
    pub name: String,
    pub description: String,
    pub input_schema: String,
    pub output_schema: String,
    pub server_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAgentSkill {
    pub skill_prompt: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignDepartmentsRequest {
    pub personnel_id: Option<i64>,
    pub department_ids: Vec<i64>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignRolesRequest {
    pub personnel_id: i64,
    pub role_ids: Vec<i64>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateDepartmentRequest {
    pub parent_id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateMenuRequest {
    pub name: String,
    pub path: Option<String>,
    pub parent_id: Option<i64>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePermissionRequest {
    pub name: String,
    pub description: Option<String>,
    pub menu_id: i64,
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePersonnelRequest {
    pub user_id:  Option<i64>,
    pub name: String,
    pub gender: Option<String>,
    pub email: Option<String>,
    pub wechat: Option<String>,
    pub phone: Option<String>,
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_super_admin: Option<bool>,
    pub permission_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::menus)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Menu {
    pub id: i64,
    pub name: String,
    pub path: Option<String>,
    pub parent_id: Option<i64>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = crate::schema::menus)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMenu {
    pub name: String,
    pub path: Option<String>,
    pub parent_id: Option<i64>,
    pub icon: Option<String>,
    pub sort_order: i32,
}


#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub is_super_admin: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateDepartmentRequest {
    pub name:  Option<String>,
    pub parent_id:  Option<i64>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdatePersonnelRequest {
    pub name:  Option<String>,
    pub gender: Option<String>,
    pub email: Option<String>,
    pub wechat: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateMenuRequest {
    pub name:  Option<String>,
    pub path: Option<String>,
    pub parent_id: Option<i64>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateRoleRequest {
    pub name:  Option<String>,
    pub description: Option<String>,
    pub is_super_admin: Option<bool>,
}