#!/usr/bin/env python3
"""Loopback PostgreSQL COMMIT fault relay for synthetic local tests only.

One-shot boundary selected with --cut before-send or after-commit-response.
Only a dedicated test owner's connection may use this port; migration and
observers connect directly. SSL/GSS negotiation refused, never downgraded
silently: test caller must explicitly select PgSslMode::Disable. No byte logs.
"""
import argparse
import asyncio
import json
import pathlib
import struct

MAX_FRAME = 4 * 1024 * 1024
MAX_CONNECTIONS = 16
CONNECTION_SECONDS = 30
RELAY_SECONDS = 120


async def packet(reader, startup=False):
    tag = b"" if startup else await reader.readexactly(1)
    size_bytes = await reader.readexactly(4)
    size = struct.unpack("!I", size_bytes)[0]
    if not 4 <= size <= MAX_FRAME:
        raise ValueError("invalid frame length")
    payload = await reader.readexactly(size - 4)
    return tag, payload, tag + size_bytes + payload


class Relay:
    def __init__(self, host, port, cut, events):
        self.host, self.port, self.cut, self.events = host, port, cut, events
        self.next_id = 0
        self.used = False
        self.failed = False

    def event(self, code, connection):
        # No SQL, parameter, backend secret, role, tenant or payload contents.
        with self.events.open("a", encoding="utf-8") as f:
            f.write(json.dumps({"event": code, "connection": connection}) + "\n")

    async def connection(self, client_reader, client_writer):
        self.next_id += 1
        cid = self.next_id
        if cid > MAX_CONNECTIONS:
            self.failed = True
            # Log only once: adversarial retries cannot amplify artifact bytes.
            if cid == MAX_CONNECTIONS + 1:
                self.event("relay_fixture_error", cid)
                self.event("total_connection_budget_exhausted", cid)
            client_writer.close()
            return
        try:
            await asyncio.wait_for(self._connection(client_reader, client_writer, cid), CONNECTION_SECONDS)
        except asyncio.TimeoutError:
            self.failed = True
            self.event("relay_fixture_error", cid)
            self.event("connection_deadline_exceeded", cid)
            client_writer.close()

    async def _connection(self, client_reader, client_writer, cid):
        server_writer = None
        tasks = []
        try:
            server_reader, server_writer = await asyncio.open_connection(self.host, self.port)
            _, payload, frame = await packet(client_reader, startup=True)
            if len(payload) < 4 or struct.unpack("!I", payload[:4])[0] != 196608:
                raise ValueError("explicit plaintext PostgreSQL v3 startup required")
            server_writer.write(frame)
            await server_writer.drain()
            commit_pending = False
            statements = {}
            portals = {}

            async def forward():
                nonlocal commit_pending
                while True:
                    tag, payload, frame = await packet(client_reader)
                    commit = False
                    if tag == b"Q":
                        query = payload.rstrip(b"\0").strip().rstrip(b";").strip().upper()
                        # Exact COMMIT only. Multi-statement batches unsupported.
                        commit = query == b"COMMIT"
                    elif tag == b"P":
                        name, query, _ = payload.split(b"\0", 2)
                        statements[name] = query.strip().rstrip(b";").strip().upper() == b"COMMIT"
                        if len(statements) > 256:
                            raise ValueError("prepared statement budget exceeded")
                    elif tag == b"B":
                        portal, statement, _ = payload.split(b"\0", 2)
                        portals[portal] = statements.get(statement, False)
                        if len(portals) > 256:
                            raise ValueError("portal budget exceeded")
                    elif tag == b"E":
                        portal, _ = payload.split(b"\0", 1)
                        commit = portals.get(portal, False)
                    if commit:
                        if commit_pending:
                            raise ValueError("pipelined COMMIT unsupported")
                        commit_pending = True
                        self.event("commit_frontend_witness", cid)
                        if self.cut == "before-send" and not self.used:
                            self.used = True
                            self.event("cut_before_commit_send", cid)
                            return
                    server_writer.write(frame)
                    await server_writer.drain()

            async def backward():
                nonlocal commit_pending
                while True:
                    tag, payload, frame = await packet(server_reader)
                    if tag == b"C" and payload == b"COMMIT\0":
                        if not commit_pending:
                            raise ValueError("unmatched COMMIT response")
                        self.event("commit_backend_success_witness", cid)
                        if self.cut == "after-commit-response" and not self.used:
                            self.used = True
                            self.event("cut_before_commit_response_delivery", cid)
                            return
                    if tag == b"Z":
                        commit_pending = False
                    client_writer.write(frame)
                    await client_writer.drain()

            tasks = [asyncio.create_task(forward()), asyncio.create_task(backward())]
            done, _ = await asyncio.wait(tasks, return_when=asyncio.FIRST_COMPLETED)
            for task in done:
                task.result()
        except (asyncio.IncompleteReadError, ConnectionError):
            self.event("connection_closed", cid)
        except Exception:
            self.failed = True
            self.event("relay_fixture_error", cid)
        finally:
            for task in tasks:
                task.cancel()
            await asyncio.gather(*tasks, return_exceptions=True)
            client_writer.close()
            if server_writer:
                server_writer.close()
            try:
                await asyncio.wait_for(client_writer.wait_closed(), 1)
            except (asyncio.TimeoutError, ConnectionError):
                pass
            if server_writer:
                try:
                    await asyncio.wait_for(server_writer.wait_closed(), 1)
                except (asyncio.TimeoutError, ConnectionError):
                    pass


async def main(args):
    events = pathlib.Path(args.events)
    events.touch(mode=0o600, exist_ok=False)
    relay = Relay("127.0.0.1", args.upstream_port, args.cut, events)
    server = await asyncio.start_server(relay.connection, "127.0.0.1", 0, limit=MAX_FRAME)
    pathlib.Path(args.port_file).write_text(str(server.sockets[0].getsockname()[1]), encoding="ascii")
    async with server:
        try:
            await asyncio.wait_for(server.serve_forever(), RELAY_SECONDS)
        except asyncio.TimeoutError:
            relay.failed = True
            relay.event("relay_fixture_error", 0)
            relay.event("relay_lifetime_exceeded", 0)
            raise SystemExit(124)


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--upstream-port", type=int, required=True)
    p.add_argument("--cut", choices=["before-send", "after-commit-response"], required=True)
    p.add_argument("--events", required=True)
    p.add_argument("--port-file", required=True)
    asyncio.run(main(p.parse_args()))
