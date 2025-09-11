use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

pub async fn handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "ozan": "kenan",
            "asli":"sahin",
            "123":"456",
            "anne seni cok seviyorum":"anne seni cok seviyorum",
            "KAŞIK":"ÇATAL"
        })),
    )
}
