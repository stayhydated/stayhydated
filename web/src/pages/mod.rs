mod home;

use crate::site::routing::{PageKind, SiteRoute};
use dioxus::prelude::*;

pub(crate) fn route_content(route: SiteRoute) -> Element {
    match route.page {
        PageKind::Home => rsx!(home::HomePage {
            locale: route.locale
        }),
    }
}
