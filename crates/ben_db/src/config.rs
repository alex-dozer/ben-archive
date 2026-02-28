#[derive(Clone)]
pub struct ChConfig {
    pub url: String,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub db: String,
}
impl Default for ChConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("CH_URL").unwrap_or("http://localhost:8123".into()),
            user: Some("default".to_string()),
            pass: Some("".to_string()),
            db: "ben_telemetry".to_string(),
        }
    }
}
