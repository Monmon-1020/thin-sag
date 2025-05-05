use anyhow::Result;

pub trait UiAdapter: Send + Sync + 'static {
    fn launch(&self, target: &str) -> Result<()>;
    fn click(&self, selector: Option<&str>, x: Option<i32>, y: Option<i32>) -> Result<()>;
    fn type_text(&self, text: &str) -> Result<()>;
    fn scroll(&self, dy: i32) -> Result<()>;
    fn keypress(&self, key: &str) -> Result<()>;
    fn wait_ms(&self, ms: u64);
}