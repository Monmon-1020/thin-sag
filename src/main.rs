#![allow(clippy::needless_return)]
use anyhow::{Result, Context};
use clap::{Parser, Subcommand};


mod vault;
mod ui_adapter;
mod models;
mod api;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// これまでの CLI 方式 
    Run {
        #[arg(long)] app: String,
        #[arg(long)] secret: String,
        #[arg(long, default_value = "Hello {secret}!")] text: String,
    },
    /// 新モード：API サーバ
    Serve {
        #[arg(long, default_value_t = 8900)] port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Run { app, secret, text } => {
            let secret_val = vault::get_secret(&secret)?;
            let text = text.replace("{secret}", &secret_val);
            ui_adapter::launch_app(&app)?;
            std::thread::sleep(std::time::Duration::from_secs(1));
            ui_adapter::type_text(&text)?;
        }
        Commands::Serve { port } => {
            let router = api::build_router();
            println!("🔌  API サーバ起動 http://127.0.0.1:{port}");

            // ① TCP リスナーを作成
            let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;

            // ② axum::serve を使って HTTP サーバを起動
            axum::serve(listener, router.into_make_service())
                .await
                .context("サーバ起動中にエラーが発生しました")?;
        }
    }
    Ok(())
}