# CI check log, run 38986142851 (24715b5082668f095cbfba6a057d0f5bfc6edb65)

Last 300 lines of the Check, lint, unit tests job.

```
=== Format check ===
=== Check SSR target ===
    Updating crates.io index
    Checking lol_team_companion v0.1.0 (/home/runner/work/lol_team_companion/lol_team_companion)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 40.22s
warning: the following packages contain code that will be rejected by a future version of Rust: proc-macro-error2 v2.0.1
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
=== Check WASM/hydrate target ===
    Checking lol_team_companion v0.1.0 (/home/runner/work/lol_team_companion/lol_team_companion)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 26.56s
warning: the following packages contain code that will be rejected by a future version of Rust: proc-macro-error2 v2.0.1
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
=== Clippy (SSR) ===
    Checking lol_team_companion v0.1.0 (/home/runner/work/lol_team_companion/lol_team_companion)
error: deref which would be done by auto-deref
   --> src/pages/design_marks.rs:155:43
    |
155 | ...                   {mark_svg(*paths, "size-4 shrink-0", "1.5")}
    |                                 ^^^^^^ help: try: `paths`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#explicit_auto_deref
    = note: `-D clippy::explicit-auto-deref` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::explicit_auto_deref)]`

error: deref which would be done by auto-deref
   --> src/pages/design_marks.rs:167:47
    |
167 | ...                   {mark_svg(*paths, "size-6 shrink-0", "1.33333")}
    |                                 ^^^^^^ help: try: `paths`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#explicit_auto_deref

error: deref which would be done by auto-deref
   --> src/pages/design_marks.rs:170:48
    |
170 | ...                   {mark_specimen(*paths)}
    |                                      ^^^^^^ help: try: `paths`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#explicit_auto_deref

error: could not compile `lol_team_companion` (lib) due to 3 previous errors
```
