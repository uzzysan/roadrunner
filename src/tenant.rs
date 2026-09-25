use std::future::Future;

use async_stream::try_stream;
use axum::{
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::Response,
};
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::TryStreamExt;
use sqlx::{
    postgres::{PgQueryResult, PgRow, PgStatement, PgTypeInfo},
    Describe, Either, Execute, Executor, PgPool, Postgres, Transaction,
};
use uuid::Uuid;

use crate::{auth::jwt, errors::AppError, state::AppState};

pub const CARRIER_HEADER: &str = "x-carrier-id";

tokio::task_local! {
    static REQUEST_TENANT_CONTEXT: Option<TenantContext>;
}

/// Database scope applied to every tenant-aware transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantContext {
    Carrier(Uuid),
    SystemAdmin,
}

#[derive(Debug, Clone)]
pub struct TenantIdentity {
    pub carrier_id: Uuid,
    pub claims: Option<jwt::Claims>,
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
        let (carrier_id, system_admin) = self.settings();

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

    fn settings(self) -> (String, &'static str) {
        match self {
            Self::Carrier(carrier_id) => (carrier_id.to_string(), "false"),
            Self::SystemAdmin => (String::new(), "true"),
        }
    }
}

pub fn current_request_context() -> Option<TenantContext> {
    REQUEST_TENANT_CONTEXT
        .try_with(|context| *context)
        .ok()
        .flatten()
}

/// Pool adapter that refuses unscoped tenant queries and wraps every operation
/// in a transaction with SET LOCAL carrier settings. Dropped/cancelled futures
/// roll the transaction back, and pooled connections never retain tenant state.
#[derive(Clone, Debug)]
pub struct TenantPool {
    inner: PgPool,
}

impl TenantPool {
    pub fn new(inner: PgPool) -> Self {
        Self { inner }
    }

    pub(crate) fn raw_pool(&self) -> &PgPool {
        &self.inner
    }

    pub async fn begin(&self) -> Result<Transaction<'_, Postgres>, sqlx::Error> {
        let context = current_request_context().ok_or_else(|| {
            sqlx::Error::Protocol("tenant database access attempted without context".into())
        })?;
        context.begin(&self.inner).await
    }

    pub async fn begin_as(
        &self,
        context: TenantContext,
    ) -> Result<Transaction<'_, Postgres>, sqlx::Error> {
        context.begin(&self.inner).await
    }
}

impl From<PgPool> for TenantPool {
    fn from(pool: PgPool) -> Self {
        Self::new(pool)
    }
}

impl<'pool> Executor<'pool> for &'pool TenantPool {
    type Database = Postgres;

    fn fetch_many<'executor, 'query: 'executor, Query>(
        self,
        query: Query,
    ) -> BoxStream<'executor, Result<Either<PgQueryResult, PgRow>, sqlx::Error>>
    where
        'pool: 'executor,
        Query: 'query + Execute<'query, Postgres>,
    {
        Box::pin(try_stream! {
            let mut tx = self.begin().await?;
            let mut rows = (&mut *tx).fetch_many(query);
            while let Some(row) = rows.try_next().await? {
                yield row;
            }
            drop(rows);
            tx.commit().await?;
        })
    }

    fn fetch_optional<'executor, 'query: 'executor, Query>(
        self,
        query: Query,
    ) -> BoxFuture<'executor, Result<Option<PgRow>, sqlx::Error>>
    where
        'pool: 'executor,
        Query: 'query + Execute<'query, Postgres>,
    {
        Box::pin(async move {
            let mut tx = self.begin().await?;
            let row = (&mut *tx).fetch_optional(query).await?;
            tx.commit().await?;
            Ok(row)
        })
    }

    fn prepare_with<'executor, 'query: 'executor>(
        self,
        sql: &'query str,
        parameters: &'executor [PgTypeInfo],
    ) -> BoxFuture<'executor, Result<PgStatement<'query>, sqlx::Error>>
    where
        'pool: 'executor,
    {
        Box::pin(async move {
            let mut tx = self.begin().await?;
            let statement = (&mut *tx).prepare_with(sql, parameters).await?;
            tx.commit().await?;
            Ok(statement)
        })
    }

    fn describe<'executor, 'query: 'executor>(
        self,
        sql: &'query str,
    ) -> BoxFuture<'executor, Result<Describe<Postgres>, sqlx::Error>>
    where
        'pool: 'executor,
    {
        Box::pin(async move {
            let mut tx = self.begin().await?;
            let description = (&mut *tx).describe(sql).await?;
            tx.commit().await?;
            Ok(description)
        })
    }
}

pub async fn scope_request_context<F>(context: Option<TenantContext>, future: F) -> F::Output
where
    F: Future,
{
    REQUEST_TENANT_CONTEXT.scope(context, future).await
}

pub async fn can_access_carrier(
    pool: &TenantPool,
    user_id: Uuid,
    carrier_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT app.can_access_carrier($1, $2)")
        .bind(user_id)
        .bind(carrier_id)
        .fetch_one(pool.raw_pool())
        .await
}

async fn is_active_carrier(pool: &TenantPool, carrier_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT app.is_active_carrier($1)")
        .bind(carrier_id)
        .fetch_one(pool.raw_pool())
        .await
}

/// Resolves and verifies the carrier before any handler touches tenant-owned
/// tables, then scopes every pool checkout made by that handler.
pub async fn tenant_middleware(
    State(state): State<AppState>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    if matches!(request.uri().path(), "/" | "/health" | "/webhooks/stripe") {
        return Ok(scope_request_context(None, next.run(request)).await);
    }

    let requested_carrier = request
        .headers()
        .get(CARRIER_HEADER)
        .map(|value| {
            value
                .to_str()
                .map_err(|_| AppError::BadRequest("Invalid X-Carrier-ID header".into()))
                .and_then(|value| {
                    Uuid::parse_str(value)
                        .map_err(|_| AppError::BadRequest("Invalid X-Carrier-ID header".into()))
                })
        })
        .transpose()?;

    let claims = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|token| {
            jwt::decode_token(token, &state.config.jwt_secret)
                .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))
        })
        .transpose()?;

    let carrier_id = claims
        .as_ref()
        .map(|claims| claims.carrier_id)
        .or(requested_carrier)
        .unwrap_or(state.config.default_carrier_id);

    if requested_carrier.is_some_and(|requested| requested != carrier_id) {
        return Err(AppError::Forbidden(
            "The requested carrier does not match the authenticated carrier".into(),
        ));
    }

    let authorized = if let Some(claims) = &claims {
        can_access_carrier(&state.db, claims.sub, carrier_id).await?
    } else {
        is_active_carrier(&state.db, carrier_id).await?
    };
    if !authorized {
        return Err(AppError::Forbidden("Carrier access denied".into()));
    }

    request
        .extensions_mut()
        .insert(TenantIdentity { carrier_id, claims });

    Ok(scope_request_context(Some(TenantContext::carrier(carrier_id)), next.run(request)).await)
}
