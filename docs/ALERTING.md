# Alerting from the monitoring server

The two checks of RUNBOOK 2.7 run on the owner's second server (`brain`) as systemd timers. They decide *when* something is wrong; this page is the contract for the application on that server that turns a decision into an alert (the owner's choice on 13 Oct 2026: no mailer and no external service on `brain`, the owner's own web app there sends the one emergency message). Nothing in Team Companion's scripts needs to change for it.

## What runs, and what it decides

| Timer | Script | Runs | Decides | Output |
|---|---|---|---|---|
| `companion-healthcheck.timer` | `/opt/companion-watch/healthcheck-remote.sh` | every 5 minutes | **DOWN** after 2 failed checks in a row (`FAILS_BEFORE_ALERT`), **BACK** on the first ok after that | exit 0 ok, 1 failed, 2 bad setting; one log line per run; the count of failures in a row in `/var/lib/companion-watch/healthcheck.state` |
| `companion-backup-watch.timer` | `/opt/companion-watch/backup-watch.sh remote` | 07:15 and 19:15 UTC | **STALE** when the newest backup on the Storage Box is 36 h or older (`MAX_AGE_HOURS`), or none is found or readable | exit 0 ok, 1 alert, 2 bad setting; one log line per run |

A failed check is not an alert: the uptime script alerts once when the count reaches 2 (about 10 minutes down) and once more when the site is back, nothing in between; the backup script alerts on every run that finds the backups stale (twice a day until fixed). The settings and log lines are in `scripts/healthcheck-remote.sh` and `scripts/backup-watch.sh` (comment block at the top of each).

## Option A (recommended): be the mailer

Both scripts send their alerts through `mail` or `/usr/sbin/sendmail`, whichever exists; without either they only log "alert not sent". So the simplest integration is a `sendmail` the app provides: an executable at `/usr/sbin/sendmail` (root-owned, `0755`) that accepts the call `sendmail -t` and reads one message from standard input:

```
To: <ALERT_EMAIL>
Subject: <subject>

<body, several lines>
```

The subject tells the app what happened; the body is for the human:

| Subject | Meaning | Body holds |
|---|---|---|
| `Team Companion is DOWN: https://app.team-companion.eu/healthz` | 2 failed checks in a row | the latest reason (HTTP status and the first 200 characters of the answer, or curl's error), the checking host, the time (UTC), the pointer to RUNBOOK 11 |
| `Team Companion is back: https://app.team-companion.eu/healthz` | ok again | how many checks failed in a row |
| `Team Companion backups: <reason>` | backups stale, missing or unreadable | the reason (for example `the newest backup is 40 h old (2026-10-12T02:31:00Z), more than 36 h`), mode, limit, the pointer to RUNBOOK 5 and 7 |

Rules for the shim: exit 0 once the message is accepted (a non-zero exit only logs "sendmail failed"; the script does not retry), take under a few seconds, never print to stdout. The `To:` address is `ALERT_EMAIL` from the systemd units (`info@team-companion.eu` today); the app may ignore it and use its own recipients. Test the path once with the command in RUNBOOK 2.7 (`HEALTH_URL=https://app.team-companion.eu/nope FAILS_BEFORE_ALERT=1 STATE_FILE=/tmp/hc-test ALERT_EMAIL=<you> /opt/companion-watch/healthcheck-remote.sh` must produce one DOWN message), then remove `/tmp/hc-test`.

With this option the de-duplication (one DOWN, one BACK) stays in the scripts and the app only forwards.

## Option B: poll the results

If the app would rather read than be called:

- **Last result of each check:** `systemctl show companion-healthcheck.service -p ExecMainStatus -p ExecMainExitTimestamp` (and the same for `companion-backup-watch.service`): `ExecMainStatus=0` ok, `1` failed, `2` bad setting. A timestamp older than 15 minutes (uptime) or 13 hours (backups) means the timer itself stopped: alert on that too.
- **Failures in a row:** `/var/lib/companion-watch/healthcheck.state` holds one integer. `0` ok; `2` or more means DOWN by the script's own rule. The file is written by user `ops`; read it, never write it.
- **Why:** the journal, `journalctl -u companion-healthcheck -n 1 -o cat` gives lines like `healthcheck: ok: <url>` or `healthcheck: FAILED (3 in a row): <url>: <reason>`; `journalctl -u companion-backup-watch -n 1 -o cat` gives `backup-watch: ok: newest backup <time> (<n> h old)` or `backup-watch: ALERT: <reason>`.
- **When to alert:** DOWN when the state file reads 2 or more (once, then again only after it went back to 0); BACK when it reads 0 after a DOWN; BACKUPS when the backup service's last status is 1 (once a day is enough); and TIMER DEAD when either timestamp is stale.

Option B needs the app to keep its own "already alerted" state; option A gets that for free.

## Either way

- The owner reads the alert and follows RUNBOOK 11 (incident) or RUNBOOK 5 and 7 (backups). The alert is the only thing the app owes; no automatic action on the VM.
- The app's own emergency message when `brain` itself is down is a separate matter (the owner's).
- Record in the launch checklist (section 6) the date the first real alert arrived; until then the line stays open.
