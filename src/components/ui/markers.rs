//! Small domain marks: RoleLabel (TOP JGL MID BOT SUP), SideMarker (blue
//! circle, red square) and ResultGlyph (filled W, hollow L). Each carries
//! its meaning in text and shape, never in colour alone.

use leptos::prelude::*;

use super::draft::DraftSide;

/// A lane role. Serialised in lowercase ("top", "jungle", "mid", "bot",
/// "support"): the Draft Graph's trait table stores it that way.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Top,
    Jungle,
    Mid,
    Bot,
    Support,
}

impl Role {
    pub const ALL: [Role; 5] = [Role::Top, Role::Jungle, Role::Mid, Role::Bot, Role::Support];

    /// Parse any spelling the app stores or the Riot API returns.
    pub fn parse(s: &str) -> Option<Role> {
        match s.trim().to_ascii_lowercase().as_str() {
            "top" => Some(Role::Top),
            "jungle" | "jgl" | "jg" | "jungler" => Some(Role::Jungle),
            "mid" | "middle" => Some(Role::Mid),
            "bot" | "bottom" | "adc" | "carry" | "marksman" => Some(Role::Bot),
            "support" | "sup" | "supp" | "utility" => Some(Role::Support),
            _ => None,
        }
    }

    /// The stored spelling: top, jungle, mid, bot, support.
    pub fn key(self) -> &'static str {
        match self {
            Role::Top => "top",
            Role::Jungle => "jungle",
            Role::Mid => "mid",
            Role::Bot => "bot",
            Role::Support => "support",
        }
    }

    /// The three-letter mark: TOP, JGL, MID, BOT, SUP.
    pub fn short(self) -> &'static str {
        match self {
            Role::Top => "TOP",
            Role::Jungle => "JGL",
            Role::Mid => "MID",
            Role::Bot => "BOT",
            Role::Support => "SUP",
        }
    }

    /// The position icon's name on `/img/ui/position/{name}`.
    pub fn icon_name(self) -> &'static str {
        match self {
            Role::Top => "top",
            Role::Jungle => "jungle",
            Role::Mid => "middle",
            Role::Bot => "bottom",
            Role::Support => "utility",
        }
    }

    /// The word: Top, Jungle, Mid, Bot, Support.
    pub fn label(self) -> &'static str {
        match self {
            Role::Top => "Top",
            Role::Jungle => "Jungle",
            Role::Mid => "Mid",
            Role::Bot => "Bot",
            Role::Support => "Support",
        }
    }
}

/// How a [`RoleLabel`] is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RoleLabelStyle {
    /// Mono caps in muted ink: inside rows and slots.
    #[default]
    Plain,
    /// In a 3px outline box: roster columns, filters.
    Boxed,
    /// Accent ink: the role being selected right now.
    Active,
}

impl RoleLabelStyle {
    fn class(self) -> &'static str {
        match self {
            RoleLabelStyle::Plain => "text-muted",
            RoleLabelStyle::Boxed => "h-5 rounded-ctl border border-outline px-1.5 text-secondary",
            RoleLabelStyle::Active => "text-accent",
        }
    }

    /// The position icon: the label's own ink, except on `Active`, where it
    /// stays `secondary` (chalk is never for icons).
    fn icon_class(self) -> &'static str {
        match self {
            RoleLabelStyle::Plain | RoleLabelStyle::Boxed => {
                "inline-block size-3.5 shrink-0 bg-current"
            }
            RoleLabelStyle::Active => "inline-block size-3.5 shrink-0 bg-current text-secondary",
        }
    }
}

/// The inline mask for a role's icon: the same-origin sanitised SVG.
fn role_icon_mask(role: Role) -> String {
    let url = format!("/img/ui/position/{}", role.icon_name());
    format!(
        "-webkit-mask:url({url}) center/contain no-repeat;mask:url({url}) center/contain no-repeat"
    )
}

/// A role as its position icon beside a typographic mark (TOP, JGL, MID,
/// BOT, SUP). The icon is the app's own `/img/ui/position/{name}` (a
/// sanitised copy, served same-origin; never a third-party image), drawn as
/// a CSS mask over `bg-current`: it takes the label's ink (`muted` plain,
/// `secondary` boxed) and stays `secondary` on an `Active` label, never the
/// accent. 14 px square left of the letters, 4 px gap (recipe card C, M152).
///
/// A11y: the icon and the three letters are hidden from screen readers and
/// the full word ("Jungle") is read instead. The letters stay text, so the
/// label scales and still reads when the icon does not load. A `guessed`
/// role (inferred, not set by a person) is drawn as uncertain: a dashed
/// underline and a "?" after the letters, and it is read as
/// "Jungle, role guessed". The dashes and "?" sit on the letters only. If
/// the icon does not load, its box stays empty and the letters remain.
#[component]
pub fn RoleLabel(
    role: Role,
    #[prop(optional)] style: RoleLabelStyle,
    /// The role was inferred, not confirmed.
    #[prop(optional)]
    guessed: bool,
) -> impl IntoView {
    let class = format!(
        "inline-flex shrink-0 items-center gap-1 font-mono text-caption font-medium uppercase leading-none tracking-[0.06em] {}",
        style.class()
    );
    view! {
        <span class=class data-role=role.label() data-role-guessed=guessed.then_some("true")>
            <span
                aria-hidden="true"
                class=style.icon_class()
                data-role-icon=role.icon_name()
                style=role_icon_mask(role)
            ></span>
            {if guessed {
                view! {
                    <span aria-hidden="true" class="border-b border-dashed border-current pb-px">{role.short()}</span>
                    <span aria-hidden="true">"?"</span>
                    <span class="sr-only">{format!("{}, role guessed", role.label())}</span>
                }
                .into_any()
            } else {
                view! {
                    <span aria-hidden="true">{role.short()}</span>
                    <span class="sr-only">{role.label()}</span>
                }
                .into_any()
            }}
        </span>
    }
}

/// Whether a [`SideMarker`] writes its side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SideText {
    /// Shape only; the accessible name is still "Blue side".
    #[default]
    None,
    /// "Blue" / "Red".
    Short,
    /// "Blue side" / "Red side".
    Long,
}

/// The side convention from the Survey legend: blue is a circle, red a
/// square, in matched colours.
///
/// A11y: the shape is decorative; the side is always in text, visible
/// (`text`) or visually hidden ("Blue side").
#[component]
pub fn SideMarker(side: DraftSide, #[prop(optional)] text: SideText) -> impl IntoView {
    let shape = match side {
        DraftSide::Blue => "size-2.5 shrink-0 rounded-full bg-side-blue",
        DraftSide::Red => "size-2.5 shrink-0 rounded-[1px] bg-side-red",
    };
    let words = match text {
        SideText::None | SideText::Long => format!("{} side", side.label()),
        SideText::Short => side.label().to_string(),
    };
    let visible = text != SideText::None;
    view! {
        <span class="inline-flex items-center gap-1.5 text-label text-secondary" data-side=side.as_str()>
            <span class=shape aria-hidden="true"></span>
            {if visible {
                view! { <span>{words}</span> }.into_any()
            } else {
                view! { <span class="sr-only">{words}</span> }.into_any()
            }}
        </span>
    }
}

/// A game result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    Win,
    Loss,
}

impl GameResult {
    pub fn from_win(win: bool) -> Self {
        if win {
            GameResult::Win
        } else {
            GameResult::Loss
        }
    }

    pub fn letter(self) -> &'static str {
        match self {
            GameResult::Win => "W",
            GameResult::Loss => "L",
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            GameResult::Win => "Win",
            GameResult::Loss => "Loss",
        }
    }
}

/// Size of a [`ResultGlyph`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlyphSize {
    /// 20px.
    Sm,
    /// 24px.
    #[default]
    Md,
}

/// Win or loss as a glyph: a filled teal W or a hollow neutral L. Never the
/// side colours, so "red side" and "loss" cannot be confused.
///
/// A11y: `role="img"` named "Win" or "Loss"; the letter is the visible
/// shorthand. With `word` the word is written out instead of the letter.
#[component]
pub fn ResultGlyph(
    result: GameResult,
    #[prop(optional)] size: GlyphSize,
    /// Write "Win"/"Loss" instead of the letter.
    #[prop(optional)]
    word: bool,
) -> impl IntoView {
    let tone = match result {
        GameResult::Win => "bg-win border-win text-win-ink",
        GameResult::Loss => "border-loss text-secondary",
    };
    let dims = match (size, word) {
        (GlyphSize::Sm, false) => "size-5",
        (GlyphSize::Md, false) => "size-6",
        (GlyphSize::Sm, true) => "h-5 px-1.5",
        (GlyphSize::Md, true) => "h-6 px-2",
    };
    let class = format!(
        "inline-grid shrink-0 place-items-center rounded-ctl border-[1.5px] font-mono text-caption font-semibold leading-none {tone} {dims}"
    );
    let text = if word { result.word() } else { result.letter() };
    view! {
        <span class=class role="img" aria-label=result.word() data-result=result.word()>
            <span aria-hidden="true">{text}</span>
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_parse_every_stored_spelling() {
        assert_eq!(Role::parse("top"), Some(Role::Top));
        assert_eq!(Role::parse("Jungle"), Some(Role::Jungle));
        assert_eq!(Role::parse("middle"), Some(Role::Mid));
        assert_eq!(Role::parse("ADC"), Some(Role::Bot));
        assert_eq!(Role::parse("UTILITY"), Some(Role::Support));
        assert_eq!(Role::parse("coach"), None);
        let shorts: Vec<&str> = Role::ALL.iter().map(|r| r.short()).collect();
        assert_eq!(shorts, ["TOP", "JGL", "MID", "BOT", "SUP"]);
    }

    /// Every role's icon is on the server's allow-list, and the mask only
    /// ever points at the app's own route.
    #[cfg(feature = "ssr")]
    #[test]
    fn role_icons_are_same_origin_and_allowed() {
        let names: Vec<&str> = Role::ALL.iter().map(|r| r.icon_name()).collect();
        assert_eq!(names, ["top", "jungle", "middle", "bottom", "utility"]);
        for r in Role::ALL {
            assert!(crate::server::ui_icons::lookup("position", r.icon_name()).is_some());
            let m = role_icon_mask(r);
            assert!(m.contains(&format!("url(/img/ui/position/{})", r.icon_name())));
            assert!(!m.contains("//"));
        }
    }

    #[test]
    fn results_have_letters_and_words() {
        assert_eq!(GameResult::from_win(true).letter(), "W");
        assert_eq!(GameResult::from_win(false).word(), "Loss");
    }
}
