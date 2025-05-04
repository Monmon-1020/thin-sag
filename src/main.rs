use clap::Parser;
use anyhow::{Context, Result};

mod vault;
mod ui_adapter;

/// コマンドライン引数定義
#[derive(Parser)]
#[command(author, version, about)]
struct Opt {
    /// 起動するアプリの Bundle ID
    #[arg(long)]
    app: String,

    /// Keychain のアイテムラベル
    #[arg(long)]
    secret: String,

    /// 入力テキスト ( "{secret}" をプレースホルダとする )
    #[arg(long, default_value = "Hello {secret}!")]
    text: String,
}

fn main() -> Result<()> {
    // 1) 引数パース
    let opt = Opt::parse();

    // 2) Keychain からシークレット取得
    let secret = vault::get_secret(&opt.secret)
        .context("Keychain からシークレットを取得できませんでした")?;

    // 3) テキストに埋め込み
    let input_text = opt.text.replace("{secret}", &secret);

    // 4) アプリ起動
    ui_adapter::launch_app(&opt.app)
        .context(format!("アプリ {} の起動に失敗しました", &opt.app))?;

    // 5) 少し待ってから入力
    std::thread::sleep(std::time::Duration::from_secs(1));
    ui_adapter::type_text(&input_text)
        .context("テキスト入力に失敗しました")?;

    Ok(())
}