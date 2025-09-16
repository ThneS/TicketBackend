use eyre::Result;

use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Debug, Clone)]
pub struct Db(pub PgPool);

impl Db {
    pub async fn connect(
        database_url: &str,
        max_connections: u32,
    ) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Set any session level configs here if needed
                    sqlx::query("SET application_name = 'ticket-backend'")
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(database_url)
            .await?;
        Ok(Self(pool))
    }

    pub fn pool(&self) -> &PgPool {
        &self.0
    }
}

// Migration helper (expects sqlx-cli or external migration runner). For now a stub.
#[allow(unused)]
pub async fn run_migrations(_pool: &PgPool) -> Result<()> {
    // You can later integrate: sqlx::migrate!("migrations").run(pool).await?;
    Ok(())
}
