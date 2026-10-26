# CI check log, run 40391465887 (30e1ab2a10ef991c77b4e13640d842ae80a3bd73)

Last 300 lines of the Check, lint, unit tests job.

```
=== Format check ===
error: unexpected closing delimiter: `}`
    --> /home/runner/work/lol_team_companion/lol_team_companion/src/components/nav.rs:1003:5
     |
 726 |     view! {
     |           - this delimiter might not be properly closed...
...
 772 |                             }}
     |                             - ...as it matches this but it has different indentation
...
1003 |     }
     |     ^ unexpected closing delimiter

Error writing files: failed to resolve mod `nav`: cannot parse /home/runner/work/lol_team_companion/lol_team_companion/src/components/nav.rs
```
