use rmcp::tool;
use rmcp::tool_handler;
use rmcp::tool_router;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::ServerHandler;
use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::schemars;
use rmcp::model::*;
use rmcp::service::*;

use schemars::JsonSchema;
use schemars::Schema;
use schemars::json_schema;
use schemars::SchemaGenerator;
use serde::{Deserialize, Serialize};
use Result;
use futures::Future;

#[derive(Debug, Clone, Deserialize, Serialize,JsonSchema)]
struct AddParams {
    a: i64,
    b: i64,
}


impl AddParams {
    pub fn sum(&self) -> i64 {
        self.a + self.b
    }
}


#[derive(Debug, Clone, Deserialize, Serialize,JsonSchema)]
struct MultiplyParams {
    a: i64,
    b: i64,
}


impl MultiplyParams {
    pub fn product(&self) -> i64 {
        self.a * self.b
    }
}

#[derive(Clone,Default)]
pub struct McpToolsHandler {
    pub tool_router: ToolRouter<McpToolsHandler>,
}


#[tool_router]
impl McpToolsHandler {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router().clone(),
        }
    }

    #[tool(name = "add", description = "Add two numbers", annotations(title = "我的工具", read_only_hint = true))]
    async fn add(&self, params: Parameters<AddParams>) -> Result<CallToolResult, ErrorData> {
        let data = params.0.sum();
        let mut contents = vec! [];
        let data = RawContent::Text(RawTextContent{text:format!("{}",data),meta:None});
        let content = Content::new(data,None);
        contents.push(content);
        Result::Ok(CallToolResult::success(contents))
    }

    #[tool(name = "multiply", description = "Multiply two numbers")]
    async fn multiply(&self, params: Parameters<MultiplyParams>) -> Result<CallToolResult, ErrorData> {
        let data = params.0.product();
        let mut contents = vec! [];
        let data = RawContent::Text(RawTextContent{text:format!("{}",data),meta:None});
        let content = Content::new(data,None);
        contents.push(content);
        Result::Ok(CallToolResult::success(contents))
    }
}

type McpError = ErrorData;
#[tool_handler]
impl ServerHandler for McpToolsHandler { 

   fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + MaybeSendFuture + '_ {
        println!("request: {:?}",request);
        println!("context: {:?}",context);
        println!("list_tools");
        let tools = self.tool_router.list_all();
        println!("tools: {:?}",tools);
        std::future::ready(Ok(ListToolsResult{tools:tools,next_cursor:None,meta:None}))
    }


    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_logging()
                .build(),
        )
        // .with_server_info(Implementation::new(
        //     "DOCKER MCP Server",
        //     env!("CARGO_PKG_VERSION"),
        // ))
        // .with_instructions(
        //     "EXECUTE COIMMAND ON DOCKER, AND GET THE IO RESULT FROM DOCKER STD",
        // )
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        Ok(ServerHandler::get_info(self))
    }

}