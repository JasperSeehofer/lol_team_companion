use leptos::prelude::*;

/// Riot Games' required legal disclaimer, verbatim, with the product name
/// the nav shows. Change only when the product is renamed.
pub const RIOT_DISCLAIMER: &str = "Companion isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.";

/// Site-wide footer rendered on every page: legal links plus Riot's
/// disclaimer. Quiet chrome: muted caption text on the page ground.
#[component]
pub fn SiteFooter() -> impl IntoView {
    let link = "text-muted underline decoration-rule underline-offset-4 transition-colors hover:text-primary hover:decoration-accent";
    view! {
        <footer class="mt-16 border-t border-rule">
            <div class="mx-auto flex max-w-7xl flex-col gap-3 px-4 py-6 sm:flex-row sm:items-start sm:justify-between sm:gap-10 sm:px-8">
                <p class="max-w-3xl text-caption leading-relaxed text-muted" data-testid="riot-disclaimer">{RIOT_DISCLAIMER}</p>
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
    use super::RIOT_DISCLAIMER;

    #[test]
    fn disclaimer_is_riots_exact_wording() {
        assert!(RIOT_DISCLAIMER.starts_with("Companion isn't endorsed by Riot Games"));
        assert!(RIOT_DISCLAIMER.ends_with(
            "Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc."
        ));
    }
}
