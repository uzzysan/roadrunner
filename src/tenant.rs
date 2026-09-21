use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Database scope applied to every tenant-aware transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantContext {
    Carrier(Uuid),
    SystemAdmin,
}

impl TenantContext {
    pub fn carrier(carrier_id: Uuid) -> Self {
        Self::Carrier(carrier_id)
    }

    pub fn system_admin() -> Self {
        Self::SystemAdmin
    }

    /// Starts a transaction and installs transaction-local PostgreSQL settings.
    /// The settings disappear on commit or rollback, so pooled connections cannot
    /// leak one request's carrier into the next request.
    pub async fn begin<'a>(
        self,
        pool: &'a PgPool,
    ) -> Result<Transaction<'a, Postgres>, sqlx::Error> {
        let mut tx = pool.begin().await?;
        self.apply(&mut tx).await?;
        Ok(tx)
    }

    /// Applies the context to an existing transaction.
    pub async fn apply(self, tx: &mut Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
        let (carrier_id, system_admin) = match self {
            Self::Carrier(carrier_id) => (carrier_id.to_string(), "false"),
            Self::SystemAdmin => (String::new(), "true"),
        };

        sqlx::query(
            "SELECT set_config('app.carrier_id', $1, true), \
                    set_config('app.system_admin', $2, true)",
        )
        .bind(carrier_id)
        .bind(system_admin)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
