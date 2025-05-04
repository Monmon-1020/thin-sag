use security_framework::os::macos::passwords::find_generic_password;

pub fn get_secret(label: &str) -> Result<String, anyhow::Error> {
    // 1) ラベルでアイテム検索
    let (password, _) = find_generic_password(None, label, "")
        .map_err(|_| anyhow::anyhow!("Keychain 内にラベル '{}' が見つかりません", label))?;
    // 2) バイト配列を UTF-8 文字列に変換
    let s = String::from_utf8(password.to_vec())
        .map_err(|_| anyhow::anyhow!("パスワードが UTF-8 として不正です"))?;

    Ok(s)
}