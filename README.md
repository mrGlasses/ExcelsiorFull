# webAxumTowerMicroserviceInterconection
Yeah, a web Axum-Tower-Tokio Microservice Interconection


And it works!

## Environment variables

- `WORKER_SHUTDOWN_TIMEOUT_SECS` — optional, seconds each background worker gets to stop on shutdown before it's abandoned (default `8`, under Docker's 10-second grace period).
- `DEMO_HEARTBEAT_FAIL_AFTER` — optional, demo only: makes the `heartbeat` worker fail after that many beats, so the app exits with code 1.
- `DEMO_STUCK_WORKER` — optional, demo only: `true` starts a worker that ignores the stop signal, to see the shutdown timeout.
- `TELEGRAM_BOT_TOKEN` — **required**, the bot token from @BotFather; the app exits at start without it. Don't put a real token in `.env` (it's tracked in git): set it in your shell or IDE run configuration.
- `TELEGRAM_API_URL` — optional, Telegram Bot API base URL (default `https://api.telegram.org`); tests point it at a fake server.
- `OUTBOX_MAX_ATTEMPTS` — optional, how many times the outbox worker tries a failing event before setting it aside (default `10`).

## Database migrations

The SQL files in `src/database/migrations` (`0001_...sql`, `0002_...sql`, ...) are compiled
into the app and applied automatically at startup by sqlx, recorded in `_sqlx_migrations`.
Never edit a file that has already been applied (the app refuses to start if you do); add a
new numbered file instead. SQL files are kept LF by `.gitattributes` (sqlx checksums them).

## Outbox (MariaDB)

Events are written to the `outbox` table in the same transaction as the change they describe
(`insert_outbox_event(&mut tx, ...)`, then `wake_outbox_worker()` after the commit), and
handled by the `outbox` worker (`handlers/outbox_handler.rs`). MariaDB has no LISTEN/NOTIFY:
the app wakes its own worker, and the worker re-checks the table every second for events
inserted from outside. Failing events are retried up to `OUTBOX_MAX_ATTEMPTS`, and
`FOR UPDATE SKIP LOCKED` lets several app copies share the work without doubles.
Demo: `curl -X POST localhost:3000/demo-events -H 'Content-Type: application/json' -d '{"msg":"hi"}'`.
