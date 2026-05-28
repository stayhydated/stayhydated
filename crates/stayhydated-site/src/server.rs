use std::path::{Path, PathBuf};

use dioxus::prelude::Element;
use dioxus_server::axum::Router;
use dioxus_server::{DioxusRouterExt as _, FullstackState, ServerFunction};

pub fn serve_dioxus_site(
    app: fn() -> Element,
    cleanup_generated_route_cache: impl Fn(&Path) -> std::io::Result<()> + Copy + 'static,
    mark_generated_route_cache: impl Fn(&Path) -> std::io::Result<()> + Copy + 'static,
) -> ! {
    dioxus_server::serve(move || async move {
        let cfg = serve_config();
        let public_dir = public_dir();
        cleanup_generated_route_cache(&public_dir).expect("failed to clear generated route cache");
        if std::env::var_os("DIOXUS_PUBLIC_PATH").is_none() {
            mark_generated_route_cache(&public_dir).expect("failed to mark generated route cache");
        }

        let app_router = Router::new().serve_dioxus_application(cfg.clone(), app);
        let app_router = with_base_path(app_router, cfg, app);

        if dioxus::cli_config::base_path().is_some() {
            Ok(static_routes_router()
                .with_state(FullstackState::headless())
                .merge(app_router))
        } else {
            Ok(app_router)
        }
    })
}

pub fn public_dir() -> PathBuf {
    if let Ok(path) = std::env::var("DIOXUS_PUBLIC_PATH") {
        return path.into();
    }

    std::env::current_exe()
        .expect("server binary path should be available")
        .parent()
        .expect("server binary should have a parent directory")
        .join("public")
}

fn serve_config() -> dioxus_server::ServeConfig {
    dioxus_server::ServeConfig::builder()
        .incremental(
            dioxus_server::IncrementalRendererConfig::new()
                .static_dir(public_dir())
                .clear_cache(false),
        )
        .enable_out_of_order_streaming()
}

fn static_routes_router() -> Router<FullstackState> {
    let mut static_routes_router = Router::new();
    for func in ServerFunction::collect() {
        if func.path() == "/api/static_routes" {
            static_routes_router = static_routes_router.route(func.path(), func.method_router());
        }
    }

    static_routes_router
}

fn with_base_path(
    app_router: Router<()>,
    cfg: dioxus_server::ServeConfig,
    app: fn() -> Element,
) -> Router<()> {
    use dioxus_server::axum::body::Body;
    use dioxus_server::axum::extract::{Request, State};

    let Some(base_path) = dioxus::cli_config::base_path() else {
        return app_router;
    };

    let base_path = base_path.trim_matches('/');

    Router::new()
        .nest(&format!("/{base_path}/"), app_router)
        .route(
            &format!("/{base_path}"),
            dioxus_server::axum::routing::get(
                |State(state): State<FullstackState>, mut request: Request<Body>| async move {
                    *request.uri_mut() = "/".parse().expect("root route should parse");
                    FullstackState::render_handler(State(state), request).await
                },
            )
            .with_state(FullstackState::new(cfg, app)),
        )
}
