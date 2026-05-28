pub mod route_cache;
pub mod routing;
#[cfg(feature = "server")]
pub mod server;
pub mod sitemap;

#[macro_export]
macro_rules! dioxus_site_main {
    (
        app = $app:path,
        cleanup_generated_route_cache = $cleanup:path,
        mark_generated_route_cache = $mark:path $(,)?
    ) => {
        #[cfg(feature = "server")]
        fn main() {
            $crate::server::serve_dioxus_site(
                $app,
                |public_dir| $cleanup(public_dir),
                |public_dir| $mark(public_dir),
            );
        }

        #[cfg(all(feature = "web", not(feature = "server")))]
        fn main() {
            dioxus::launch($app);
        }

        #[cfg(not(any(feature = "web", feature = "server")))]
        fn main() {}
    };
}
