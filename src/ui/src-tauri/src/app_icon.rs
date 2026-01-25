use data::entities::{App, Ref};
use tauri::{Manager, UriSchemeContext, UriSchemeResponder, http};
use util::error::{Context, Result};
use util::tracing;

use crate::state::AppState;

pub fn protocol_handler<R: tauri::Runtime>(
    ctx: UriSchemeContext<'_, R>,
    req: http::Request<Vec<u8>>,
    res: UriSchemeResponder,
) {
    // Clone the app handle to move into async closure
    let app_handle = ctx.app_handle().clone();
    // Spawn async task to handle the request
    tauri::async_runtime::spawn(async move {
        let result = handle_request(app_handle, req).await;
        match result {
            Ok(response) => {
                res.respond(response);
            }
            Err(e) => {
                tracing::error!(?e, "failed to handle app icon request");
                let error_response = http::Response::builder()
                    .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .header(http::header::CONTENT_TYPE, "text/plain")
                    .body(format!("Error: {e}").into_bytes())
                    .unwrap();
                res.respond(error_response);
            }
        }
    });
}

async fn handle_request<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    req: http::Request<Vec<u8>>,
) -> Result<http::Response<Vec<u8>>> {
    // Parse app_id from the request URI
    let uri = req.uri();
    let path = uri.path();

    // Extract app_id from path (e.g., "/123" -> 123)
    let app_id_str = path.trim_start_matches('/');
    let app_id: i64 = app_id_str
        .parse()
        .context("failed to parse app_id from URI path")?;

    let app_id_ref = Ref::<App>::new(app_id);

    // Get AppState from the app handle
    let state = app_handle.state::<AppState>();

    // Get the repo and fetch the icon
    let icon_data = {
        let state_guard = state.read().await;
        let state_inner = state_guard.assume_init();
        let mut repo = state_inner.get_repo().await?;
        repo.get_app_icon(app_id_ref).await?
    };

    match icon_data {
        Some(icon_bytes) => {
            // Return the icon as image/png (assuming PNG format)
            let response = http::Response::builder()
                .status(http::StatusCode::OK)
                // Don't manually set the content type, let the browser infer it
                // .header(http::header::CONTENT_TYPE, "image/png")
                .header(http::header::CACHE_CONTROL, "public, max-age=31536000")
                .body(icon_bytes)
                .unwrap();
            Ok(response)
        }
        None => {
            // Return 404 if icon not found
            let response = http::Response::builder()
                .status(http::StatusCode::NOT_FOUND)
                .header(http::header::CONTENT_TYPE, "text/plain")
                .header(http::header::CACHE_CONTROL, "public, max-age=31536000")
                .body(b"App icon not found".to_vec())
                .unwrap();
            Ok(response)
        }
    }
}
