//! ChampionAvatar (a champion's square icon from the app's own image
//! endpoint, with a monogram while it loads or when it is missing) and
//! UserAvatar (a person's initial).

use leptos::prelude::*;

use super::draft::DraftSide;
use crate::models::ddragon::champion_icon_url;

/// Size of an avatar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarSize {
    /// 20px: inline in text and dense rows.
    Xs,
    /// 24px: table rows, lists.
    Sm,
    /// 32px: the default (slots, cards).
    #[default]
    Md,
    /// 40px: draft slots, roster.
    Lg,
    /// 56px: a featured champion.
    Xl,
}

impl AvatarSize {
    pub fn class(self) -> &'static str {
        match self {
            AvatarSize::Xs => "size-5 text-caption tracking-[-0.06em]",
            AvatarSize::Sm => "size-6 text-caption",
            AvatarSize::Md => "size-8 text-caption",
            AvatarSize::Lg => "size-10 text-label",
            AvatarSize::Xl => "size-14 text-body",
        }
    }
}

/// Shape of a champion avatar (radius policy: square with the control
/// radius, or round like a map token).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarShape {
    /// 3px corners: lists, pools, tables.
    #[default]
    Square,
    /// Round: tokens. A side overrides the shape (blue round, red square).
    Round,
}

/// What the champion is doing in context.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarState {
    #[default]
    Normal,
    /// Banned: desaturated, struck through, and "banned" in the name.
    Banned,
    /// Out of focus (not in the current filter, not available).
    Dimmed,
}

/// Two-letter monogram for a champion name: initials of the first two
/// words ("Lee Sin" "LS", "Kai'Sa" "KS"), otherwise the first two letters
/// ("Azir" "Az").
pub fn monogram(name: &str) -> String {
    let words: Vec<&str> = name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    match words.as_slice() {
        [] => "?".to_string(),
        [one] => {
            let mut chars = one.chars();
            let first = chars.next().map(|c| c.to_uppercase().to_string());
            let second = chars.next().map(|c| c.to_lowercase().to_string());
            format!(
                "{}{}",
                first.unwrap_or_default(),
                second.unwrap_or_default()
            )
        }
        [a, b, ..] => {
            let fa = a.chars().next().map(|c| c.to_uppercase().to_string());
            let fb = b.chars().next().map(|c| c.to_uppercase().to_string());
            format!("{}{}", fa.unwrap_or_default(), fb.unwrap_or_default())
        }
    }
}

fn frame_class(shape: AvatarShape, side: Option<DraftSide>) -> &'static str {
    match (side, shape) {
        (Some(DraftSide::Blue), _) => "rounded-full border-2 border-side-blue bg-side-blue-deep",
        (Some(DraftSide::Red), _) => "rounded-ctl border-2 border-side-red bg-side-red-deep",
        (None, AvatarShape::Round) => "rounded-full border border-outline/60 bg-ink-2",
        (None, AvatarShape::Square) => "rounded-ctl border border-outline/60 bg-ink-2",
    }
}

fn state_class(state: AvatarState) -> &'static str {
    match state {
        AvatarState::Normal => "",
        AvatarState::Banned => "grayscale opacity-60",
        AvatarState::Dimmed => "opacity-40",
    }
}

/// A champion's square icon from `/img/champion/{name}/icon`.
///
/// A11y: an `<img>` whose alt text is the champion's name ("Azir", or
/// "Azir, banned"); pass `decorative` when the name is written next to it,
/// which empties the alt. The monogram underneath shows while the image
/// loads and stays if it is missing (404); it is hidden from screen
/// readers. Sides add the shape (blue round, red square) as well as the
/// colour; the banned state adds a strike and the word, not only grey.
#[component]
pub fn ChampionAvatar(
    /// Display name or Data Dragon id; the server resolves either.
    #[prop(into)]
    name: String,
    #[prop(optional)] size: AvatarSize,
    #[prop(optional)] shape: AvatarShape,
    #[prop(optional)] side: Option<DraftSide>,
    #[prop(optional)] state: AvatarState,
    /// The name is visible next to the avatar: empty alt text.
    #[prop(optional)]
    decorative: bool,
) -> impl IntoView {
    let failed = RwSignal::new(false);
    let img_ref = NodeRef::<leptos::html::Img>::new();
    let src = champion_icon_url(&name);
    let alt = if decorative {
        String::new()
    } else if state == AvatarState::Banned {
        format!("{name}, banned")
    } else {
        name.clone()
    };
    let mono = monogram(&name);
    let alt_sr = alt.clone();

    // An image that failed before hydration attached `on:error`.
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        if let Some(img) = img_ref.get() {
            if img.complete() && img.natural_width() == 0 {
                failed.set(true);
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = img_ref;
    });

    let class = format!(
        "relative inline-grid shrink-0 place-items-center overflow-hidden {} {} {}",
        size.class(),
        frame_class(shape, side),
        state_class(state)
    );
    let banned = state == AvatarState::Banned;
    view! {
        <span class=class data-champion=name>
            <span class="figures font-semibold leading-none tracking-[-0.02em] text-secondary" aria-hidden="true">{mono}</span>
            {move || (!failed.get()).then(|| view! {
                <img
                    node_ref=img_ref
                    src=src.clone()
                    alt=alt.clone()
                    loading="lazy"
                    decoding="async"
                    width="64"
                    height="64"
                    class="absolute inset-0 size-full object-cover text-transparent"
                    on:error=move |_| failed.set(true)
                />
            })}
            {move || failed.get().then(|| {
                // The alt text went with the image: keep the name.
                let a = alt_sr.clone();
                (!a.is_empty()).then(|| view! { <span class="sr-only">{a}</span> })
            })}
            {banned.then(|| view! {
                <span class="pointer-events-none absolute left-[-10%] top-1/2 h-0.5 w-[120%] -rotate-45 bg-primary" aria-hidden="true"></span>
            })}
        </span>
    }
}

/// A person's initial in a round token (nav, roster, comments).
///
/// A11y: decorative by default (the name is written next to it); pass
/// `label` when it stands alone, which makes it `role="img"` with that
/// name.
#[component]
pub fn UserAvatar(
    #[prop(into)] name: String,
    #[prop(optional)] size: AvatarSize,
    #[prop(optional, into)] label: Option<String>,
) -> impl IntoView {
    let initial = name
        .chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string());
    let class = format!(
        "inline-grid shrink-0 place-items-center rounded-full bg-ink-3 font-semibold text-secondary {}",
        size.class()
    );
    match label {
        Some(l) => view! { <span class=class role="img" aria-label=l>{initial}</span> }.into_any(),
        None => view! { <span class=class aria-hidden="true">{initial}</span> }.into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monograms() {
        assert_eq!(monogram("Azir"), "Az");
        assert_eq!(monogram("Lee Sin"), "LS");
        assert_eq!(monogram("Kai'Sa"), "KS");
        assert_eq!(monogram("Dr. Mundo"), "DM");
        assert_eq!(monogram("Vi"), "Vi");
        assert_eq!(monogram(""), "?");
    }
}
