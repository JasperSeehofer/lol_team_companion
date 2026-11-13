use leptos::prelude::*;

/// Riot Games' required legal disclaimer, verbatim, with the product name
/// the nav shows. Change only when the product is renamed.
pub const RIOT_DISCLAIMER: &str = "Team Companion isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.";

/// The notice Riot's "Legal Jibber Jabber" policy (section 6) asks a
/// project to show, word for word from https://www.riotgames.com/en/legal
/// (read 2026-10-17) with the project's title filled in. Shown beside
/// [`RIOT_DISCLAIMER`], never instead of it.
pub const RIOT_LJJ_NOTICE: &str = "Team Companion was created under Riot Games' \"Legal Jibber Jabber\" policy using assets owned by Riot Games. Riot Games does not endorse or sponsor this project.";

/// Community links for everyone: (label, path).
pub const PUBLIC_COMMUNITY: &[(&str, &str)] = &[
    ("Changelog", "/changelog"),
    ("Transparency", "/transparency"),
];
/// The public source code (owner's decision, 12 October 2026: the
/// repository is public). A link the visitor follows; the page loads
/// nothing from it, and it carries no money wording.
pub const SOURCE_URL: &str = "https://github.com/JasperSeehofer/lol_team_companion"; // lint:allow(external-url)

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
                <div class="flex max-w-3xl flex-col gap-2">
                    <p class="text-caption leading-relaxed text-muted" data-testid="riot-disclaimer">{RIOT_DISCLAIMER}</p>
                    <p class="text-caption leading-relaxed text-muted" data-testid="riot-notice">{RIOT_LJJ_NOTICE}</p>
                </div>
                <nav class="flex shrink-0 flex-wrap gap-x-5 gap-y-2 text-caption" aria-label="Community">
                    {PUBLIC_COMMUNITY.iter().map(|(l, h)| view! { <a href=*h class=link>{*l}</a> }).collect_view()}
                    <a href=SOURCE_URL class=link rel="noopener noreferrer" data-testid="source-link">"Source"</a>
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
                    <a href="/legal/terms" class=link>"Terms"</a>
                </nav>
            </div>
        </footer>
    }
}

#[cfg(test)]
mod tests {
    use super::{MEMBER_COMMUNITY, PUBLIC_COMMUNITY, RIOT_DISCLAIMER, RIOT_LJJ_NOTICE, SOURCE_URL};

    /// The source link goes to the public repository itself (not a funding
    /// page; the community test checks the footer's words).
    #[test]
    fn source_link_is_the_public_repository() {
        assert!(SOURCE_URL.starts_with("https://"));
        assert!(SOURCE_URL.ends_with("//github.com/JasperSeehofer/lol_team_companion"));
    }

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
        assert!(RIOT_DISCLAIMER.starts_with("Team Companion isn't endorsed by Riot Games"));
        assert!(RIOT_DISCLAIMER.ends_with(
            "Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc."
        ));
    }

    /// Section 6 of Riot's "Legal Jibber Jabber", word for word with the
    /// title filled in; the existing disclaimer stays beside it.
    #[test]
    fn notice_is_the_legal_jibber_jabber_wording() {
        assert_eq!(
            RIOT_LJJ_NOTICE,
            "Team Companion was created under Riot Games' \"Legal Jibber Jabber\" policy using assets owned by Riot Games. Riot Games does not endorse or sponsor this project."
        );
        assert!(RIOT_LJJ_NOTICE.starts_with("Team Companion was created under"));
        assert!(RIOT_LJJ_NOTICE.ends_with("Riot Games does not endorse or sponsor this project."));
        assert_ne!(RIOT_LJJ_NOTICE, RIOT_DISCLAIMER);
    }
}
