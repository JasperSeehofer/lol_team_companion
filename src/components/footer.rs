use leptos::prelude::*;

/// Riot Games' required legal disclaimer, verbatim, with the product name
/// the nav shows. Change only when the product is renamed.
pub const RIOT_DISCLAIMER: &str = "Companion isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.";

/// Site-wide footer rendered on every page: legal links plus Riot's
/// disclaimer.
#[component]
pub fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="border-t border-divider bg-surface/60 mt-12">
            <div class="max-w-7xl mx-auto px-4 sm:px-8 py-6 flex flex-col gap-3 text-xs text-muted">
                <nav class="flex flex-wrap gap-4" aria-label="Legal">
                    <a href="/legal/impressum" class="hover:text-secondary underline-offset-2 hover:underline focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded">
                        "Impressum"
                    </a>
                    <a href="/legal/datenschutz" class="hover:text-secondary underline-offset-2 hover:underline focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded">
                        "Datenschutz"
                    </a>
                </nav>
                <p class="max-w-3xl leading-relaxed" data-testid="riot-disclaimer">{RIOT_DISCLAIMER}</p>
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
