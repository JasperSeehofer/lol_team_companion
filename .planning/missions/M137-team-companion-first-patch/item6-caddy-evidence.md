# Item 6 evidence: Caddy access-log filter

Caddy v2.11.7 linux_amd64 release binary (compose uses `caddy:2.11`). `filter multiple` is NOT registered in this version
(`module not registered: caddy.logging.encoders.filter.multiple`), so one `regexp` covers both the share path and the invite query:

    request>uri regexp (^/s/)[^/?#]+|([?&]invite=)[^&#]* ${1}${2}REDACTED
    request>headers>Referer delete

`caddy validate --adapter caddyfile` (env COMPANION_DOMAIN/ACME_EMAIL set, log path redirected to a tmp dir because /var/log/caddy is not writable here): `Valid configuration`.

Live run (log block copied from the Caddyfile, `respond "ok" 200`), requests:
1. `GET /s/abcdefabcdefabcdefabcdefabcdef12?x=1` with `Referer: https://x.example/s/SECRETREFERER`
2. `GET /auth/register?invite=SECRETCODE&a=b`
3. `GET /auth/register?a=b&invite=SECRET2`

Logged `uri` values:

    /s/REDACTED?x=1
    /auth/register?invite=REDACTED&a=b
    /auth/register?a=b&invite=REDACTED

Request headers logged: only User-Agent and Accept (Referer absent). `grep -ciE 'abcdefabc|SECRET|referer' access.log` = 0.
