# CI check log, run 39616282863 (05ddead09af961461fc89f68814f2023652c0178)

Last 300 lines of the Check, lint, unit tests job.

```
=== Format check ===
=== Check SSR target ===
    Updating crates.io index
    Checking lol_team_companion v0.1.0 (/home/runner/work/lol_team_companion/lol_team_companion)
error[E0525]: expected a closure that implements the `Fn` trait, but this closure only implements `FnOnce`
   --> src/components/nav.rs:548:5
    |
548 | /     view! {
549 | |         <Dialog id="phone-menu" open=open title="Menu" size=DialogSize::Sheet>
550 | |             <nav aria-label="Phone menu" class="-mx-5 -my-4" data-phone-menu>
    | |              --- required by a bound introduced by this call
551 | |                 <div class="flex items-center justify-between gap-3 border-b border-rule px-5 py-4">
...   |
560 | |                 {model.groups.into_iter().map(|(heading, pages)| {
    | |                  ------------ closure is `FnOnce` because it moves the variable `model.groups` out of its environment
...   |
635 | |         </Dialog>
636 | |     }
    | |     ^
    | |     |
    | |_____this closure implements `FnOnce`, not `Fn`
    |       the requirement to implement `Fn` derives from here
    |
    = note: required for `Arc<dyn Fn() -> leptos::prelude::AnyView + Send + std::marker::Sync>` to implement `leptos::prelude::ToChildren<{closure@src/components/nav.rs:548:5: 636:6}>`
    = note: this error originates in the macro `view` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0525`.
error: could not compile `lol_team_companion` (lib) due to 1 previous error
```
