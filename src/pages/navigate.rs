//! Client-side navigation that can run later (from an event handler or
//! after an await), captured while a component runs. Browser only: on the
//! server there is nothing to navigate, and a local stored value must not
//! cross threads, so the server build carries an empty `Nav`.

#[cfg(feature = "hydrate")]
use leptos::prelude::*;

/// The router's navigate function, captured for later use.
#[derive(Clone, Copy)]
pub struct Nav {
    #[cfg(feature = "hydrate")]
    f: StoredValue<NavFn, LocalStorage>,
}

/// Navigate to a path; `true` replaces the history entry without scrolling.
#[cfg(feature = "hydrate")]
type NavFn = std::rc::Rc<dyn Fn(&str, bool)>;

/// Capture the router's navigate function (call while the component runs).
pub fn navigator() -> Nav {
    #[cfg(feature = "hydrate")]
    {
        let navigate = leptos_router::hooks::use_navigate();
        let f: NavFn = std::rc::Rc::new(move |p: &str, replace: bool| {
            let options = leptos_router::NavigateOptions {
                replace,
                scroll: !replace,
                ..Default::default()
            };
            navigate(p, options)
        });
        Nav {
            f: StoredValue::new_local(f),
        }
    }
    #[cfg(not(feature = "hydrate"))]
    Nav {}
}

/// Go to `path` (a new history entry).
pub fn go(nav: Nav, path: &str) {
    #[cfg(feature = "hydrate")]
    {
        let _ = nav.f.try_with_value(|f| f(path, false));
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (nav, path);
}

/// Put `path` in the address bar in place of the current entry, without
/// scrolling: after a save, so a reload or a copied link reopens the record.
pub fn replace_url(nav: Nav, path: &str) {
    #[cfg(feature = "hydrate")]
    {
        let _ = nav.f.try_with_value(|f| f(path, true));
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (nav, path);
}

/// The sign-in link for the page the browser is on now:
/// `/auth/login?next=<this page>` so sign-in returns here (M148 item 8).
/// On the server (and for Home or the auth pages) plain `/auth/login`.
pub fn login_href() -> String {
    #[cfg(feature = "hydrate")]
    {
        if let Some(loc) = web_sys::window().map(|w| w.location()) {
            let path = loc.pathname().unwrap_or_default();
            let search = loc.search().unwrap_or_default();
            return crate::models::login_next::login_href_for(&format!("{path}{search}"));
        }
    }
    crate::models::login_next::LOGIN_PATH.to_string()
}

/// Send a signed-out visitor to sign in, remembering this page. Every
/// protected page calls it when `get_current_user()` answers `None` (a
/// hard navigation, leptos rule 8). Does nothing on the server.
pub fn redirect_to_login() {
    #[cfg(feature = "hydrate")]
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(&login_href());
    }
}
