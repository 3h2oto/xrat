#!/usr/bin/env python3
"""Launch the checkout's WebUI with isolated data and verify authentication."""

import argparse
import http.cookiejar
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=18204)
    args = parser.parse_args()
    if not 1 <= args.port <= 65535:
        parser.error("port must be between 1 and 65535")
    with socket.socket() as probe:
        probe.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            probe.bind(("127.0.0.1", args.port))
        except OSError:
            parser.error(f"port {args.port} is unavailable; use just web-smoke <port>")

    binary = Path(__file__).resolve().parent.parent / "target/debug/xrat"
    workspace = Path(tempfile.mkdtemp(prefix="xrat-web-smoke-"))
    env = os.environ.copy()
    env["XRAT_PATH"] = str(workspace)
    password = "admin"
    # Known admin hash for this isolated loopback test; normal password rules stay intact.
    hashed = '$argon2id$v=19$m=19456,t=2,p=1$eHJhdC1zbW9rZS1vbmx5IQ$7pdUon3k7OqnMyG07dAymRqjDr3ONzEMUtdbKzZjc5I'
    env["XRAT_ADMIN_PASSWORD_HASH"] = hashed
    credentials = workspace / "login.txt"
    credentials.write_text(f"Username: admin\nPassword: {password}\n")
    credentials.chmod(0o600)
    config = workspace / "config.toml"
    config.write_text(
        f'[server]\nhost = "127.0.0.1"\nport = {args.port}\n'
        '[server.web]\nenabled = true\nadmin_username = "admin"\n'
        'admin_password_hash = { env = "XRAT_ADMIN_PASSWORD_HASH" }\n'
        'transport = "local_http"\n'
    )
    url = f"http://127.0.0.1:{args.port}"
    log_path = workspace / "server.log"
    with log_path.open("w") as log:
        process = subprocess.Popen(
            [str(binary), "--config", str(config), "--database",
             str(workspace / "db.sqlite"), "webui", "--no-open"],
            env=env,
            stdout=log,
            stderr=log,
            start_new_session=True,
        )

        def stop(_signum: int, _frame: object) -> None:
            raise KeyboardInterrupt

        previous = signal.signal(signal.SIGTERM, stop)
        try:
            print(f"Workspace: {workspace}\nServer log: {log_path}", flush=True)
            client = urllib.request.build_opener(
                urllib.request.ProxyHandler({}),
                urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()),
            )
            deadline = time.monotonic() + 30
            while True:
                if process.poll() is not None:
                    raise RuntimeError(f"WebUI exited; inspect {log_path}")
                try:
                    with client.open(url, timeout=1) as response:
                        if b"xrat" not in response.read().lower():
                            raise RuntimeError("WebUI did not return XRAT HTML")
                    break
                except urllib.error.URLError:
                    if time.monotonic() >= deadline:
                        raise RuntimeError(f"WebUI startup timed out; inspect {log_path}")
                    time.sleep(0.2)
            for route in ("/ui/configs", "/ui/logs/xrat", "/ui/host"):
                with client.open(url + route, timeout=5) as response:
                    if "text/html" not in response.headers.get("Content-Type", ""):
                        raise RuntimeError(f"Deep link {route} did not return the WebUI")
            request = urllib.request.Request(
                url + "/api/v1/login",
                data=json.dumps({"username": "admin", "password": password}).encode(),
                headers={"Content-Type": "application/json", "Origin": url},
            )
            with client.open(request, timeout=10) as response:
                response.read()
            with client.open(url + "/api/v1/me", timeout=10) as response:
                me = json.load(response)
            for endpoint in ("version", "logs/engine"):
                with client.open(url + "/api/v1/" + endpoint, timeout=10) as response:
                    json.load(response)
            for endpoint, payload in (("host/unlock", {"password": password}), ("host/lock", {})):
                request = urllib.request.Request(
                    url + "/api/v1/" + endpoint,
                    data=json.dumps(payload).encode(),
                    headers={"Content-Type": "application/json", "Origin": url,
                             "x-csrf-token": me["csrf_token"]},
                )
                with client.open(request, timeout=10) as response:
                    json.load(response)
            print(
                f"Smoke checks passed: page, login, session, version, engine logs, host unlock/relock.\n"
                f"Open: {url}\nSmoke login: admin / admin\nCredentials: {credentials}\n"
                "Press Ctrl+C to stop. Temporary data remains for inspection.",
                flush=True,
            )
            code = process.wait()
            if code:
                raise RuntimeError(f"WebUI exited with status {code}; inspect {log_path}")
        except KeyboardInterrupt:
            print("\nStopping WebUI.", flush=True)
        finally:
            signal.signal(signal.SIGTERM, previous)
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()


if __name__ == "__main__":
    main()
