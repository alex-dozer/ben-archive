use crate::{ChHttpClient, DbError};
use ben_contracts::schema::SchemaManifest;

/// Ensures the database exists.
pub async fn ensure_database(client: &ChHttpClient, db: &str) -> Result<(), DbError> {
    client.ensure_db(db).await?;
    Ok(())
}

pub async fn apply_manifest(
    client: &ChHttpClient,
    manifest: &SchemaManifest,
) -> Result<(), DbError> {
    for table in &manifest.tables {
        client.exec(&table.ddl).await?;
    }
    Ok(())
}
