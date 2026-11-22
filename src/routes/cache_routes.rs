// Generic Redis CRUD demo, separate from the get_users cache-aside:
// localhost/cache/some-key with a JSON body {"value": "..."} for create/update.
use crate::handlers::cache_handler::{
    create_cache_entry, delete_cache_entry, get_cache_entry, update_cache_entry,
};
use crate::state::AppState;
use axum::{Router, routing::get};

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/cache/{key}",
        get(get_cache_entry)
            .post(create_cache_entry)
            .put(update_cache_entry)
            .delete(delete_cache_entry),
    )
}
