# webAxumTowerMicroserviceInterconection
Yeah, a web Axum-Tower-Tokio Microservice Interconection


And it works!

## Environment variables

- `DATABASE_MAX_CONNECTIONS` — optional, Postgres connection pool size (default `5`).

## Database migrations

The SQL files in `src/database/migrations` (`0001_...sql`, `0002_...sql`, ...) are applied
automatically at startup by sqlx, and recorded in the `_sqlx_migrations` table. Never edit a
file that has already been applied (the app refuses to start if you do); add a new numbered file instead.
