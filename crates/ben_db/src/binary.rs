use crate::error::DbError;
use ben_wire::rowbinary::RowBinaryEncode;
use reqwest::Client;
use url::Url;

pub struct ChBinaryClient {
    url: Url,
    auth: Option<(String, String)>,
    client: Client,
}

impl ChBinaryClient {
    pub fn new(
        base: &str,
        db: &str,
        user: Option<String>,
        pass: Option<String>,
    ) -> Result<Self, DbError> {
        let mut url = Url::parse(base)?;
        url.query_pairs_mut().append_pair("database", db);

        Ok(Self {
            url,
            auth: user.zip(pass),
            client: Client::new(),
        })
    }

    pub async fn insert_struct<T: RowBinaryEncode>(
        &self,
        table: &str,
        value: &T,
    ) -> Result<(), DbError> {
        let mut buf = Vec::with_capacity(128);
        value.encode_rowbinary(&mut buf)?;
        self.insert_rowbinary(table, buf).await
    }

    pub async fn insert_rowbinary(&self, table: &str, body: Vec<u8>) -> Result<(), DbError> {
        let mut url = self.url.clone();
        url.query_pairs_mut()
            .append_pair("query", &format!("INSERT INTO {} FORMAT RowBinary", table));

        let mut req = self
            .client
            .post(url)
            .header("Content-Type", "application/octet-stream")
            .body(body);

        if let Some((u, p)) = &self.auth {
            req = req.basic_auth(u, Some(p));
        }

        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(DbError::Server(text));
        }

        Ok(())
    }
}
