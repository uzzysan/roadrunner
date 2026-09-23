#!/bin/sh
set -eu

: "${DB_USER:?DB_USER must name the PostgreSQL owner role}"
: "${DB_PASSWORD:?DB_PASSWORD must contain the PostgreSQL owner password}"
: "${DB_NAME:?DB_NAME must name the application database}"
: "${DB_APP_USER:?DB_APP_USER must name the runtime role}"
: "${DB_APP_PASSWORD:?DB_APP_PASSWORD must contain the runtime role password}"
DB_HOST="${DB_HOST:-postgres}"

export PGPASSWORD="$DB_PASSWORD"

until pg_isready -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" >/dev/null 2>&1; do
    sleep 1
done

psql \
    --host="$DB_HOST" \
    --username="$DB_USER" \
    --dbname="$DB_NAME" \
    --set=ON_ERROR_STOP=1 \
    --set=app_user="$DB_APP_USER" \
    --set=app_password="$DB_APP_PASSWORD" <<'SQL'
SELECT 'CREATE ROLE roadrunner_app NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOBYPASSRLS'
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'roadrunner_app')
\gexec

SELECT format(
    'CREATE ROLE %I LOGIN PASSWORD %L NOSUPERUSER NOCREATEDB NOCREATEROLE INHERIT NOBYPASSRLS',
    :'app_user',
    :'app_password'
)
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = :'app_user')
\gexec

SELECT format(
    'ALTER ROLE %I LOGIN PASSWORD %L NOSUPERUSER NOCREATEDB NOCREATEROLE INHERIT NOBYPASSRLS',
    :'app_user',
    :'app_password'
)
\gexec

SELECT format('GRANT roadrunner_app TO %I', :'app_user')
\gexec

SELECT format('GRANT CONNECT ON DATABASE %I TO %I', current_database(), :'app_user')
\gexec
SQL
