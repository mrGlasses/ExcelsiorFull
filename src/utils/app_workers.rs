// Every background worker the app runs, started in one place. To add a worker: write a
// `start(workers, state) -> bool` next to it (like the ones below) and add ONE line here.
use crate::state::AppState;
use crate::utils::workers::Workers;

// Each start() reads its own settings and spawns its worker(s). It returns false when it
// can't start (a missing token, for example): the app then exits with code 1.
// Every start() runs before we decide, so ALL the missing settings are logged at once,
// not one per restart.
pub fn start_app_workers(workers: &mut Workers, state: &AppState) -> bool {
    let started: Vec<bool> = vec![
        crate::handlers::worker_handler::start(workers, state),
        crate::handlers::outbox_handler::start(workers, state),
        crate::handlers::telegram_handler::start(workers, state),
        crate::handlers::discord_handler::start(workers, state),
    ];
    started.into_iter().all(|ok| ok)
}
