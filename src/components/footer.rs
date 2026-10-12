use leptos::prelude::*;

/// Riot Games' required legal disclaimer, verbatim, with the product name
/// the nav shows. Change only when the product is renamed.
pub const RIOT_DISCLAIMER: &str = "Companion isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.";

/// Community links for everyone: (label, path).
pub const PUBLIC_COMMUNITY: &[(&str, &str)] = &[
    ("Changelog", "/changelog"),
    ("Transparency", "/transparency"),
];
/// Community links for signed-in users only.
pub const MEMBER_COMMUNITY: &[(&str, &str)] =
    &[("What next", "/what-next"), ("Bug board", "/bugs")];

/// Site-wide footer rendered on every page: community and legal links plus
/// Riot's disclaimer. Quiet chrome: muted caption text on the page ground.
#[component]
pub fn SiteFooter() -> impl IntoView {
    let link = "text-muted underline decoration-rule underline-offset-4 transition-colors hover:text-primary hover:decoration-accent";
    let user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    view! {
        <footer class="mt-6 border-t border-rule">
            <div class="mx-auto flex max-w-7xl flex-col gap-3 px-4 py-6 sm:flex-row sm:items-start sm:justify-between sm:gap-10 sm:px-8">
                <p class="max-w-3xl text-caption leading-relaxed text-muted" data-testid="riot-disclaimer">{RIOT_DISCLAIMER}</p>
                <nav class="flex shrink-0 flex-wrap gap-x-5 gap-y-2 text-caption" aria-label="Community">
                    {PUBLIC_COMMUNITY.iter().map(|(l, h)| view! { <a href=*h class=link>{*l}</a> }).collect_view()}
                    // Same tree on the server and in the browser (Suspend).
                    <Suspense fallback=|| ()>
                        {move || Suspend::new(async move {
                            let signed_in = matches!(user.await, Ok(Some(_)));
                            signed_in.then(|| MEMBER_COMMUNITY.iter().map(|(l, h)| view! { <a href=*h class=link>{*l}</a> }).collect_view())
                        })}
                    </Suspense>
                </nav>
                <nav class="flex shrink-0 gap-5 text-caption" aria-label="Legal">
                    <a href="/legal/impressum" class=link>"Impressum"</a>
                    <a href="/legal/datenschutz" class=link>"Datenschutz"</a>
                </nav>
            </div>
        </footer>
    }
}

#[cfg(test)]
mod tests {
    use super::{MEMBER_COMMUNITY, PUBLIC_COMMUNITY, RIOT_DISCLAIMER};

    #[test]
    fn community_links_never_include_support() {
        let all: Vec<&str> = PUBLIC_COMMUNITY
            .iter()
            .chain(MEMBER_COMMUNITY)
            .map(|l| l.1)
            .collect();
        assert_eq!(
            all,
            vec!["/changelog", "/transparency", "/what-next", "/bugs"]
        );
    }

    #[test]
    fn disclaimer_is_riots_exact_wording() {
        assert!(RIOT_DISCLAIMER.starts_with("Companion isn't endorsed by Riot Games"));
        assert!(RIOT_DISCLAIMER.ends_with(
            "Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc."
        ));
    }
}
