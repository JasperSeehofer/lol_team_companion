# Mock of the Companion HTTP surface the restore drill touches (NOT the real
# app; the real responses are checked by scripts/smoke_signin.sh in CI): /healthz, /auth/login form, login server fn
# with redirect + session cookie, get_current_user.
import http.server, sys, urllib.parse
EMAIL, PASSWORD = "drill@example.org", "drill-password-123"
class H(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a): pass
    def do_GET(self):
        if self.path == "/healthz":
            self._send(200, b'{"status":"ok","db":"ok"}', "application/json")
        elif self.path == "/auth/login":
            self._send(200, b'<html><form method="post" action="/api/login_action1234567890"><input name="email"></form></html>', "text/html")
        else:
            self._send(404, b"nf", "text/plain")
    def do_POST(self):
        n = int(self.headers.get("Content-Length", 0)); body = self.rfile.read(n).decode()
        if self.path == "/api/login_action1234567890":
            q = urllib.parse.parse_qs(body)
            if q.get("email") == [EMAIL] and q.get("password") == [PASSWORD]:
                self.send_response(302); self.send_header("Location", "/")
                self.send_header("Set-Cookie", "id=abc; HttpOnly; SameSite=Strict; Path=/")
                self.end_headers()
            else:
                self.send_response(302); self.send_header("Location", "/auth/login?__err=Invalid"); self.end_headers()
        elif self.path == "/api/get_current_user987" and "id=abc" in self.headers.get("Cookie", ""):
            self._send(200, ('{"email":"%s"}' % EMAIL).encode(), "application/json")
        else:
            self._send(401, b"no", "text/plain")
    def _send(self, code, b, ct):
        self.send_response(code); self.send_header("Content-Type", ct); self.send_header("Content-Length", str(len(b))); self.end_headers(); self.wfile.write(b)
http.server.HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
