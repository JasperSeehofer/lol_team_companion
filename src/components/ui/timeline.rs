//! TimelineScrub: the Sand Table's scrubber shell (reserved for M4). A
//! ruler with moments and a playhead that can be moved by keyboard and
//! pointer. There is no playback yet, so there is no play button.

use leptos::prelude::*;

/// A named moment on the ruler ("17:00 Pick-off").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Moment {
    /// Seconds from the start.
    pub at: u32,
    pub label: String,
}

impl Moment {
    pub fn new(at: u32, label: impl Into<String>) -> Self {
        Self {
            at,
            label: label.into(),
        }
    }
}

/// "17:05" for 1025 seconds.
pub fn fmt_clock(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// The moment the playhead is in: the last one at or before `value`.
pub fn current_moment(moments: &[Moment], value: u32) -> Option<usize> {
    moments
        .iter()
        .enumerate()
        .filter(|(_, m)| m.at <= value)
        .max_by_key(|(_, m)| m.at)
        .map(|(i, _)| i)
}

/// Where a key moves the playhead, or `None` for other keys.
/// Arrows step 1 s (10 s with Shift), PageUp/PageDown 60 s, Home/End jump
/// to the ends, J and L jump to the previous and next moment.
pub fn scrub_key(
    key: &str,
    shift: bool,
    value: u32,
    duration: u32,
    moments: &[Moment],
) -> Option<u32> {
    let step = if shift { 10 } else { 1 };
    let v = match key {
        "ArrowRight" | "ArrowUp" => value.saturating_add(step),
        "ArrowLeft" | "ArrowDown" => value.saturating_sub(step),
        "PageUp" => value.saturating_add(60),
        "PageDown" => value.saturating_sub(60),
        "Home" => 0,
        "End" => duration,
        "j" | "J" => moments
            .iter()
            .map(|m| m.at)
            .filter(|&a| a < value)
            .max()
            .unwrap_or(0),
        "l" | "L" => moments
            .iter()
            .map(|m| m.at)
            .filter(|&a| a > value)
            .min()
            .unwrap_or(duration),
        _ => return None,
    };
    Some(v.min(duration))
}

fn pct(at: u32, duration: u32) -> f32 {
    at as f32 / duration.max(1) as f32 * 100.0
}

/// The scrubber: a time readout, a moments track, a minute scale and an
/// accent playhead. Static data for now; M4 drives it from the replay.
///
/// A11y: the ruler is one tab stop with `role="slider"`, a name,
/// `aria-valuemin/max/now` in seconds and `aria-valuetext` in words
/// ("17:05, moment: Pick-off"). Left/Right step 1 s (Shift: 10 s),
/// PageUp/PageDown a minute, Home/End the ends, J/L the previous/next
/// moment (K is reserved for pause once playback exists). A click on the
/// ruler seeks. Moments are also listed as text.
#[component]
pub fn TimelineScrub(
    #[prop(into)] label: String,
    /// Length in seconds.
    duration: u32,
    value: RwSignal<u32>,
    moments: Vec<Moment>,
) -> impl IntoView {
    let moments = StoredValue::new(moments);
    let current = Memo::new(move |_| moments.with_value(|m| current_moment(m, value.get())));
    let valuetext = move || {
        let v = value.get();
        match current
            .get()
            .and_then(|i| moments.with_value(|m| m.get(i).cloned()))
        {
            Some(m) => format!("{}, moment: {}", fmt_clock(v), m.label),
            None => fmt_clock(v),
        }
    };
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let next = moments.with_value(|m| {
            scrub_key(
                &ev.key(),
                ev.shift_key(),
                value.get_untracked(),
                duration,
                m,
            )
        });
        if let Some(v) = next {
            ev.prevent_default();
            value.set(v);
        }
    };
    let on_click = move |ev: leptos::ev::MouseEvent| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            if let Some(el) = ev
                .current_target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            {
                let rect = el.get_bounding_client_rect();
                // The first 6rem are the track labels.
                let label_w = 96.0;
                let w = (rect.width() - label_w).max(1.0);
                let x = (ev.client_x() as f64 - rect.left() - label_w).clamp(0.0, w);
                value.set(((x / w) * duration as f64).round() as u32);
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = ev;
    };
    let minutes = duration / 60;
    let major = if minutes > 40 { 10 } else { 5 };
    let scale_labels = (0..=minutes / major)
        .map(|i| {
            let m = i * major;
            let p = pct(m * 60, duration);
            // Every other label is dropped on phones so they never collide.
            let shift = match (p > 99.9, i % 2 == 1) {
                (true, _) => "-translate-x-full",
                (false, true) => "-translate-x-1/2 max-sm:hidden",
                (false, false) => "-translate-x-1/2",
            };
            view! {
                <span class=format!("absolute top-3 figures text-caption leading-none text-muted {shift}") style=format!("left: {p:.3}%;")>
                    {format!("{m}:00")}
                </span>
                <span class="absolute top-0 h-2.5 w-px bg-secondary" style=format!("left: {p:.3}%;")></span>
            }
        })
        .collect_view();
    let minor = (0..=minutes)
        .map(|m| {
            let p = pct(m * 60, duration);
            view! { <span class="absolute top-0 h-1.5 w-px bg-muted" style=format!("left: {p:.3}%;")></span> }
        })
        .collect_view();
    let flags = move || {
        let cur = current.get();
        moments.with_value(|ms| {
            ms.iter()
                .enumerate()
                .map(|(i, m)| {
                    let p = pct(m.at, duration);
                    let high = i % 2 == 0;
                    let is_cur = cur == Some(i);
                    let pos = if high { "top-1.5 h-11" } else { "top-6 h-6" };
                    let stem = if is_cur { "w-0.5 self-stretch bg-accent" } else { "w-px self-stretch bg-muted" };
                    let text = if is_cur {
                        "rounded-[2px] px-0.5 text-primary outline outline-[1.5px] outline-offset-2 outline-accent"
                    } else {
                        "text-secondary"
                    };
                    view! {
                        <span class=format!("absolute flex items-start gap-1.5 whitespace-nowrap text-caption {pos}") style=format!("left: {p:.3}%;")>
                            <span class=stem></span>
                            <span class=text>
                                <b class="figures mr-1 font-medium text-primary">{fmt_clock(m.at)}</b>
                                // On phones the flag shows only the time; the
                                // current moment is named in the readout above.
                                <span class="max-sm:hidden">{m.label.clone()}</span>
                            </span>
                        </span>
                    }
                })
                .collect_view()
        })
    };
    view! {
        <div class="flex flex-col gap-3" data-timeline="true">
            <div class="flex flex-wrap items-center gap-x-5 gap-y-2">
                <output class="figures text-heading font-semibold text-primary" aria-hidden="true">{move || fmt_clock(value.get())}</output>
                <span class="text-label text-muted">
                    "Moment: "
                    <b class="font-medium text-primary">
                        {move || current.get().and_then(|i| moments.with_value(|m| m.get(i).map(|m| m.label.clone()))).unwrap_or_else(|| "none yet".to_string())}
                    </b>
                </span>
                <p class="flex flex-wrap items-center gap-x-3 gap-y-1 text-caption text-muted sm:ml-auto">
                    <span class="inline-flex items-center gap-1"><kbd class="rounded-ctl border border-outline border-b-2 px-1">"J"</kbd><kbd class="rounded-ctl border border-outline border-b-2 px-1">"L"</kbd>" moments"</span>
                    <span class="inline-flex items-center gap-1"><kbd class="rounded-ctl border border-outline border-b-2 px-1">"\u{2190}"</kbd><kbd class="rounded-ctl border border-outline border-b-2 px-1">"\u{2192}"</kbd>" 1 s, Shift 10 s"</span>
                    <span class="inline-flex items-center gap-1"><kbd class="rounded-ctl border border-outline border-b-2 px-1">"PgUp"</kbd>" 1 min"</span>
                </p>
            </div>
            <div
                role="slider"
                tabindex="0"
                aria-label=label
                aria-valuemin="0"
                aria-valuemax=duration.to_string()
                aria-valuenow=move || value.get().to_string()
                aria-valuetext=valuetext
                class="relative grid cursor-pointer grid-cols-[6rem_minmax(0,1fr)] overflow-x-clip"
                on:keydown=on_keydown
                on:click=on_click
            >
                <span class="eyebrow flex items-center border-t border-rule">"Moments"</span>
                <div class="relative h-14 border-t border-rule">{flags}</div>
                <span class="border-t border-outline"></span>
                <div class="relative h-8 border-t border-outline">{minor}{scale_labels}</div>
                <div class="pointer-events-none absolute bottom-8 left-24 right-0 top-0" aria-hidden="true">
                    <span class="absolute bottom-0 top-0 w-0.5 -translate-x-1/2 bg-accent" style=move || format!("left: {:.3}%;", pct(value.get(), duration))></span>
                </div>
            </div>
            <ol class="sr-only">
                {moments.with_value(|ms| ms.iter().map(|m| view! { <li>{format!("{} {}", fmt_clock(m.at), m.label)}</li> }).collect_view())}
            </ol>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms() -> Vec<Moment> {
        vec![
            Moment::new(360, "First roam"),
            Moment::new(1020, "Pick-off"),
            Moment::new(1450, "Baron fight"),
        ]
    }

    #[test]
    fn clock_and_current_moment() {
        assert_eq!(fmt_clock(1025), "17:05");
        assert_eq!(fmt_clock(5), "0:05");
        assert_eq!(current_moment(&ms(), 1025), Some(1));
        assert_eq!(current_moment(&ms(), 100), None);
    }

    #[test]
    fn keys_step_jump_and_clamp() {
        let m = ms();
        assert_eq!(scrub_key("ArrowRight", false, 1025, 1800, &m), Some(1026));
        assert_eq!(scrub_key("ArrowLeft", true, 1025, 1800, &m), Some(1015));
        assert_eq!(scrub_key("PageUp", false, 1790, 1800, &m), Some(1800));
        assert_eq!(scrub_key("PageDown", false, 30, 1800, &m), Some(0));
        assert_eq!(scrub_key("j", false, 1025, 1800, &m), Some(1020));
        assert_eq!(scrub_key("J", false, 1020, 1800, &m), Some(360));
        assert_eq!(scrub_key("l", false, 1025, 1800, &m), Some(1450));
        assert_eq!(scrub_key("l", false, 1500, 1800, &m), Some(1800));
        assert_eq!(scrub_key("End", false, 0, 1800, &m), Some(1800));
        assert_eq!(scrub_key("x", false, 0, 1800, &m), None);
    }
}
