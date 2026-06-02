use rmcp::handler::server::router::tool::ToolRoute;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ListToolsResult, PaginatedRequestParams,
    ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};

use crate::tools::{configure, inspect, onboard};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct AacServer {
    tool_router: ToolRouter<AacServer>,
}

impl AacServer {
    fn empty() -> Self {
        Self {
            tool_router: ToolRouter::default(),
        }
    }

    pub fn with_route(mut self, route: ToolRoute<Self>) -> Self {
        self.tool_router.add_route(route);
        self
    }

    pub fn new() -> Self {
        Self::empty()
            .with_route(inspect::inspect_route())
            .with_route(onboard::onboard_route())
            .with_route(configure::configure_route())
    }
}

impl ServerHandler for AacServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info.name = "3t-scaffold-mcp".to_string();
        info.server_info.version = VERSION.to_string();
        info.instructions = Some("3T scaffold server — use scaffold_inspect to inspect a project, scaffold_onboard to onboard it, scaffold_configure to add skills.".to_string());
        info
    }

    async fn list_tools(
        &self,
        _params: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            ..Default::default()
        })
    }

    async fn call_tool(
        &self,
        params: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let tool_ctx = rmcp::handler::server::tool::ToolCallContext::new(self, params, ctx);
        self.tool_router.call(tool_ctx).await
    }
}

pub(crate) fn text_result(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![Content::text(msg.into())])
}

pub(crate) fn error_result(msg: impl std::fmt::Display) -> CallToolResult {
    CallToolResult::error(vec![Content::text(format!("Error: {msg}"))])
}
