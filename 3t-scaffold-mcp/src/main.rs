use anyhow::Result;
use rmcp::transport::io::stdio;
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

mod bundle;
mod server;
mod tools;

use server::AacServer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let service = AacServer::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
