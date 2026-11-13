# webAxumTowerMicroserviceInterconection
Yeah, a web Axum-Tower-Tokio Microservice Interconection


And it works!

## Environment variables

- `WORKER_SHUTDOWN_TIMEOUT_SECS` — optional, seconds each background worker gets to stop on shutdown before it's abandoned (default `8`, under Docker's 10-second grace period).
- `DEMO_HEARTBEAT_FAIL_AFTER` — optional, demo only: makes the `heartbeat` worker fail after that many beats, so the app exits with code 1.
- `DEMO_STUCK_WORKER` — optional, demo only: `true` starts a worker that ignores the stop signal, to see the shutdown timeout.
- `TELEGRAM_BOT_TOKEN` — **required**, the bot token from @BotFather; the app exits at start without it. Don't put a real token in `.env` (it's tracked in git): set it in your shell or IDE run configuration.
- `TELEGRAM_API_URL` — optional, Telegram Bot API base URL (default `https://api.telegram.org`); tests point it at a fake server.
