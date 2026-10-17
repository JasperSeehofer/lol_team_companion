#!/usr/bin/env python3
"""Load check for server-rendered pages: every page must finish streaming.

Registers one account with the e2e invite code, then requests a mix of
signed-in and public pages with N requests over C parallel connections and
fails if any response does not complete within the timeout. This is the
check that found the M11 "page.goto stall" (leptos 0.8.17's <Suspense> now
and then never finished streaming: about one page in 8,000 at 16 parallel
requests; 0 in 60,000 with leptos 0.8.22). CI runs it after the e2e suite.

  scripts/ssr_stress.py [base_url] [requests] [parallel] [timeout_s]

Defaults: http://127.0.0.1:3020, 12000, 16, 20. Environment: INVITE_CODE
(default E2E-INVITE-CODE), STRESS_DUMP_DIR (write the partial body of each
stalled response there, for diagnosis). Standard library only.
"""
import os
import queue
import random
import re
import sys
import threading
import time
import urllib.parse
import urllib.request
import http.cookiejar

BASE = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:3020"
N = int(sys.argv[2]) if len(sys.argv) > 2 else 12000
PARALLEL = int(sys.argv[3]) if len(sys.argv) > 3 else 16
TIMEOUT = float(sys.argv[4]) if len(sys.argv) > 4 else 20.0
INVITE = os.environ.get("INVITE_CODE", "E2E-INVITE-CODE")
DUMP = os.environ.get("STRESS_DUMP_DIR")

SIGNED_IN = ["/", "/profile", "/stats", "/me", "/team/roster", "/opponents",
             "/draft", "/changelog", "/bugs", "/what-next"]
PUBLIC = [f"/auth/register?invite={INVITE}", "/auth/login", "/legal/datenschutz"]


def register():
    jar = http.cookiejar.CookieJar()
    opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
    html = opener.open(f"{BASE}/auth/register?invite={INVITE}", timeout=30).read().decode()
    action = re.search(r'action="(/api/register_action[A-Za-z0-9_]*)"', html).group(1)
    tag = os.urandom(4).hex()
    form = urllib.parse.urlencode({
        "username": f"stress-{tag}", "email": f"stress-{tag}@test.invalid",
        "password": "stress-password-123", "invite_code": INVITE,
        "accept_terms": "on",
    }).encode()
    opener.open(urllib.request.Request(BASE + action, data=form, headers={
        "Content-Type": "application/x-www-form-urlencoded",
        "Accept": "text/html", "Referer": BASE + "/",
    }), timeout=30).read()
    cookie = "; ".join(f"{c.name}={c.value}" for c in jar)
    if "id=" not in cookie:
        sys.exit("ssr_stress: registration gave no session cookie")
    return cookie


def main():
    cookie = register()
    work = queue.Queue()
    for i in range(N):
        work.put(i)
    lock = threading.Lock()
    stalled, failed, done = [], [], [0]
    slowest = [0.0]

    def worker():
        rng = random.Random()
        while True:
            try:
                work.get_nowait()
            except queue.Empty:
                return
            signed_in = rng.random() < 0.75
            path = rng.choice(SIGNED_IN if signed_in else PUBLIC)
            headers = {"Accept": "text/html"}
            if signed_in:
                headers["Cookie"] = cookie
            body, started = [], time.time()
            try:
                req = urllib.request.Request(BASE + path, headers=headers)
                with urllib.request.urlopen(req, timeout=TIMEOUT) as r:
                    while True:
                        chunk = r.read1(65536)
                        if not chunk:
                            break
                        body.append(chunk)
                took = time.time() - started
                with lock:
                    slowest[0] = max(slowest[0], took)
            except TimeoutError:
                with lock:
                    stalled.append((path, round(time.time() - started, 1)))
                    if DUMP:
                        os.makedirs(DUMP, exist_ok=True)
                        name = re.sub(r"[^a-z0-9]+", "_", path) or "home"
                        with open(os.path.join(DUMP, f"stalled-{len(stalled)}{name}.html"), "wb") as f:
                            f.write(b"".join(body))
            except Exception as e:  # HTTP errors, resets: reported, not stalls
                with lock:
                    failed.append((path, repr(e)[:160]))
            with lock:
                done[0] += 1

    threads = [threading.Thread(target=worker) for _ in range(PARALLEL)]
    t0 = time.time()
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    print(f"ssr_stress: {done[0]} requests in {time.time() - t0:.0f}s over {PARALLEL} connections; "
          f"slowest complete {slowest[0]:.1f}s; stalled {len(stalled)}; failed {len(failed)}")
    for s in stalled[:20]:
        print("ssr_stress: STALLED (no complete response within the timeout):", *s)
    for f in failed[:20]:
        print("ssr_stress: FAILED:", *f)
    sys.exit(1 if stalled or failed else 0)


if __name__ == "__main__":
    main()
