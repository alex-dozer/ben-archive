use crate::error::DbError;
use reqwest::Client;
use url::Url;

pub struct ChHttpClient {
    base: Url,
    auth: Option<(String, String)>,
    client: Client,
}

impl ChHttpClient {
    pub fn new(base: &str, user: Option<String>, pass: Option<String>) -> Result<Self, DbError> {
        let mut url = Url::parse(base)?;
        if !url.as_str().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }

        Ok(Self {
            base: url,
            auth: user.zip(pass),
            client: Client::new(),
        })
    }

    pub async fn exec(&self, sql: &str) -> Result<String, DbError> {
        let mut req = self.client.post(self.base.clone()).body(sql.to_string());

        if let Some((u, p)) = &self.auth {
            req = req.basic_auth(u, Some(p));
        }

        let resp = req.send().await?;
        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(DbError::Server(body));
        }

        Ok(body)
    }

    pub async fn ensure_db(&self, name: &str) -> Result<(), DbError> {
        let sql = format!("CREATE DATABASE IF NOT EXISTS `{}`", name);
        let _ = self.exec(&sql).await?;
        Ok(())
    }
}
