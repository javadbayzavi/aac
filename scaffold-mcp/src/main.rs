use anyhow::Result;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use tracing_subscriber::EnvFilter;

mod assets;
mod server;
mod sync;
mod tools;

use server::AacServer;

fn main() -> std::process::ExitCode {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to start async runtime: {e}");
            return std::process::ExitCode::from(2);
        }
    };
    match rt.block_on(run()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::ExitCode::from(2)
        }
    }
}

async fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let service = AacServer::new().serve(stdio()).await?;

    tokio::spawn(async {
        if let Err(e) = tokio::task::spawn_blocking(sync::sync)
            .await
            .unwrap_or_else(|e| Err(e.to_string()))
        {
            tracing::warn!("sync failed (using cache if available): {e}");
        }
    });

    service.waiting().await?;
    Ok(())
}
