"""Relay framing tests only. No payroll, durability or PostgreSQL acceptance."""
import asyncio
import json
import pathlib
import struct
import tempfile
import unittest
from unittest.mock import patch
from commit_relay import Relay, packet


def frame(tag, body):
    return tag + struct.pack("!I", len(body) + 4) + body


class Writer:
    def __init__(self, peer=None):
        self.frames, self.peer = [], peer

    def write(self, value):
        self.frames.append(value)
        if self.peer and (value == frame(b"Q", b"COMMIT\0") or value[:1] == b"E"):
            self.peer.feed_data(frame(b"C", b"COMMIT\0") + frame(b"Z", b"I"))

    async def drain(self):
        pass

    def close(self):
        pass

    async def wait_closed(self):
        pass


class RelayTests(unittest.IsolatedAsyncioTestCase):
    async def trial(self, cut, extended=False):
        client, upstream = asyncio.StreamReader(), asyncio.StreamReader()
        client.feed_data(frame(b"", struct.pack("!I", 196608) + b"user\0synthetic\0\0"))
        if extended:
            client.feed_data(frame(b"P", b"s\0COMMIT\0\0\0"))
            client.feed_data(frame(b"B", b"\0s\0\0\0\0\0\0\0"))
            client.feed_data(frame(b"E", b"\0\0\0\0\0"))
            client.feed_data(frame(b"S", b""))
        else:
            client.feed_data(frame(b"Q", b"COMMIT\0"))
        to_client, to_upstream = Writer(), Writer(upstream)

        async def connect(*_):
            return upstream, to_upstream

        with tempfile.TemporaryDirectory() as d:
            events = pathlib.Path(d) / "events.jsonl"
            relay = Relay("127.0.0.1", 1, cut, events)
            with patch("commit_relay.asyncio.open_connection", connect):
                await asyncio.wait_for(relay.connection(client, to_client), 1)
            rows = [json.loads(s) for s in events.read_text().splitlines()]
            self.assertFalse(relay.failed)
            self.assertTrue(relay.used)
            self.assertFalse(any(x[:1] == b"C" for x in to_client.frames))
            self.assertTrue(all(set(row) == {"event", "connection"} for row in rows))
            return to_upstream.frames, [row["event"] for row in rows]

    async def test_simple_before_send_does_not_forward_commit(self):
        frames, events = await self.trial("before-send")
        self.assertNotIn(frame(b"Q", b"COMMIT\0"), frames)
        self.assertIn("cut_before_commit_send", events)

    async def test_simple_after_response_forwards_once_and_suppresses_success(self):
        frames, events = await self.trial("after-commit-response")
        self.assertEqual(frames.count(frame(b"Q", b"COMMIT\0")), 1)
        self.assertIn("commit_backend_success_witness", events)

    async def test_extended_execute_commit_is_witnessed(self):
        _, events = await self.trial("after-commit-response", extended=True)
        self.assertIn("cut_before_commit_response_delivery", events)

    async def test_invalid_frame_size_fails(self):
        reader = asyncio.StreamReader()
        reader.feed_data(b"Q\0\0\0\3")
        with self.assertRaises(ValueError):
            await packet(reader)

    async def test_connection_budget_refuses_without_upstream_access(self):
        with tempfile.TemporaryDirectory() as d:
            relay = Relay("127.0.0.1", 1, "before-send", pathlib.Path(d) / "events")
            with patch("commit_relay.MAX_CONNECTIONS", 0), patch("commit_relay.asyncio.open_connection") as connect:
                await relay.connection(asyncio.StreamReader(), Writer())
                connect.assert_not_called()
            self.assertTrue(relay.failed)
            self.assertIn("total_connection_budget_exhausted", relay.events.read_text())

    async def test_connection_deadline_is_fixture_error(self):
        async def never_connect(*_):
            await asyncio.sleep(30)
        with tempfile.TemporaryDirectory() as d:
            relay = Relay("127.0.0.1", 1, "before-send", pathlib.Path(d) / "events")
            with patch("commit_relay.CONNECTION_SECONDS", .01), patch("commit_relay.asyncio.open_connection", never_connect):
                await relay.connection(asyncio.StreamReader(), Writer())
            self.assertTrue(relay.failed)
            self.assertIn("connection_deadline_exceeded", relay.events.read_text())


if __name__ == "__main__":
    unittest.main()
