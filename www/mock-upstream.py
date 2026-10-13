"""The upstream every docs capture asks: GitHub, GitLab, Gitea, Redmine, Jira,
Repology and deps.dev, through canibump's `CANIBUMP_<PROVIDER>_API` overrides.

Usage: python3 -I www/mock-upstream.py <routes.json> <port-file> <misses-file>

It binds a free port on 127.0.0.1 and writes the port to <port-file>. Each
request re-reads <routes.json>, so a capture session can swap what upstream
says between two runs. The routes map a request path to the JSON body that
answers it with a 200:

    {"/repos/serde-rs/serde/pulls/2590": {"state": "closed", ...}}

A key with a query string matches only that query, and a bare path matches
any. A body of {"$status": 404, "$body": {...}} answers with that status. A
request no key matches gets a 404 and a line in <misses-file>, which fails the
capture, so a page never shows output from a route nobody wrote.
"""

import http.server
import json
import sys


def answer(routes, target):
    path = target.split("?", 1)[0]
    for key in (target, path):
        if key in routes:
            body = routes[key]
            if isinstance(body, dict) and "$status" in body:
                return body["$status"], body.get("$body", {})
            return 200, body
    return None


def main():
    routes_file, port_file, misses_file = sys.argv[1:4]

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            with open(routes_file, encoding="utf-8") as f:
                routes = json.load(f)
            found = answer(routes, self.path)
            if found is None:
                with open(misses_file, "a", encoding="utf-8") as f:
                    f.write(f"GET {self.path}\n")
                found = (404, {"message": "Not Found"})
            status, body = found
            data = json.dumps(body).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def log_message(self, *args):
            pass

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    with open(port_file, "w", encoding="utf-8") as f:
        f.write(str(server.server_address[1]))
    server.serve_forever()


main()
