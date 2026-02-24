use crate::types::AppState;
use axum::routing::{Router, get};

pub fn get_router() -> Router<AppState> {
    let single_file_router = get(super::views::browse_files).delete(super::views::delete_file);

    Router::new()
        .route("/browse", single_file_router.clone())
        .route("/browse/", single_file_router.clone())
        .route("/browse/{*file_path}", single_file_router)
}
