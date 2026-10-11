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
}

/// A role as a typographic mark (TOP, JGL, MID, BOT, SUP). Replaces role
/// icons: no third-party images.
///
/// A11y: the three letters are hidden from screen readers and the full
/// word ("Jungle") is read instead. Text, so it scales and never breaks.
/// A `guessed` role (inferred, not set by a person) is drawn as uncertain:
/// a dashed underline and a "?" after the letters, and it is read as
/// "Jungle, role guessed".
#[component]
pub fn RoleLabel(
    role: Role,
    #[prop(optional)] style: RoleLabelStyle,
    /// The role was inferred, not confirmed.
    #[prop(optional)]
    guessed: bool,
) -> impl IntoView {
    let class = format!(
        "inline-flex shrink-0 items-center font-mono text-caption font-medium uppercase leading-none tracking-[0.06em] {}",
        style.class()
    );
    view! {
        <span class=class data-role=role.label() data-role-guessed=guessed.then_some("true")>
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

    #[test]
    fn results_have_letters_and_words() {
        assert_eq!(GameResult::from_win(true).letter(), "W");
        assert_eq!(GameResult::from_win(false).word(), "Loss");
    }
}
