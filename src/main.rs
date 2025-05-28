#![allow(clippy::needless_return)]
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod action;
mod adapter;
mod api;
mod error;
mod job;
mod mac_ax;
mod mask;
mod models;
mod policy;
mod screenshot;
mod tree;
mod vault;
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(long)]
        app: String,
        #[arg(long)]
        secret: String,
        #[arg(long, default_value = "Hello {secret}!")]
        text: String,
    },
    Serve {
        #[arg(long, default_value_t = 8900)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Run { app, secret, text } => {
            let secret_val = vault::get_secret(&secret)?;
            let text = text.replace("{secret}", &secret_val);
            println!("⚠️  CLI モードは廃止されました。API サーバモードを使用してください。");
        }
        Commands::Serve { port } => {
            let router = api::build_router();
            println!("🔌  API サーバ起動 http://127.0.0.1:{port}");

            let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;

            axum::serve(listener, router.into_make_service())
                .await
                .context("サーバ起動中にエラーが発生しました")?;
        }
    }
    Ok(())
}
