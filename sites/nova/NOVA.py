"""
NOVA.py - a lightweight client for NOVA.

Quick start:

    import os
    from NOVA import NOVABot

    bot = NOVABot(
        base_url="https://nova.xytro.site",
        session_cookie=os.environ["NOVA_SESSION_COOKIE"],
        prefix="!",
    )

    @bot.event
    async def on_ready():
        print("NOVA bot connected")

    @bot.command("ping")
    async def ping(ctx, *args):
        await ctx.reply("pong")

    bot.run()

Notes:
- Uses existing authenticated NOVA session cookie.
- REST APIs are used for channels, users, and channel messages.
- WebSocket is used for real-time events and DM thread replies.
"""

from __future__ import annotations

import asyncio
import inspect
from dataclasses import dataclass
from typing import Any, Awaitable, Callable, Dict, List, Optional

try:
    import aiohttp
except Exception as exc:  # pragma: no cover
    raise RuntimeError("NOVA.py requires aiohttp. Install with: pip install aiohttp") from exc


EventHandler = Callable[..., Awaitable[None] | None]


@dataclass
class Message:
    id: str
    content: str
    created_at: str
    user_id: Optional[str] = None
    username: Optional[str] = None
    channel_id: Optional[str] = None
    sender_id: Optional[str] = None
    sender_username: Optional[str] = None
    thread_id: Optional[str] = None


@dataclass
class Channel:
    id: str
    name: str
    description: Optional[str] = None
    type: str = "text"


@dataclass
class User:
    id: str
    username: str


class NOVAClient:
    def __init__(
        self,
        base_url: str,
        session_cookie: str,
        cookie_name: str = "connect.sid",
        reconnect_delay: float = 2.0,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.session_cookie = session_cookie
        self.cookie_name = cookie_name
        self.reconnect_delay = reconnect_delay

        self._handlers: Dict[str, List[EventHandler]] = {}
        self._http: Optional[aiohttp.ClientSession] = None
        self._ws: Optional[aiohttp.ClientWebSocketResponse] = None
        self._running = False
        self._closed = False
        self._me: Optional[dict[str, Any]] = None

    @property
    def me(self) -> Optional[dict[str, Any]]:
        return self._me

    def event(self, fn: EventHandler) -> EventHandler:
        """Decorator for events like on_ready, on_message, on_dm, on_presence."""
        self._handlers.setdefault(fn.__name__, []).append(fn)
        return fn

    async def _dispatch(self, event_name: str, *args: Any) -> None:
        handlers = self._handlers.get(event_name, [])
        for fn in handlers:
            try:
                result = fn(*args)
                if inspect.isawaitable(result):
                    await result
            except Exception as exc:
                await self._dispatch_error(exc, event_name)

    async def _dispatch_error(self, exc: Exception, source: str) -> None:
        handlers = self._handlers.get("on_error", [])
        if not handlers:
            print(f"[NOVA.py] unhandled error in {source}: {exc}")
            return
        for fn in handlers:
            result = fn(exc, source)
            if inspect.isawaitable(result):
                await result

    def _cookie_header(self) -> dict[str, str]:
        return {"Cookie": f"{self.cookie_name}={self.session_cookie}"}

    async def _api_get(self, path: str) -> Any:
        if self._http is None:
            raise RuntimeError("Client not started")
        url = f"{self.base_url}{path}"
        async with self._http.get(url, headers=self._cookie_header()) as resp:
            if resp.status >= 400:
                txt = await resp.text()
                raise RuntimeError(f"GET {path} failed [{resp.status}]: {txt}")
            return await resp.json()

    async def _api_post(self, path: str, payload: dict[str, Any]) -> Any:
        if self._http is None:
            raise RuntimeError("Client not started")
        url = f"{self.base_url}{path}"
        async with self._http.post(url, json=payload, headers=self._cookie_header()) as resp:
            if resp.status >= 400:
                txt = await resp.text()
                raise RuntimeError(f"POST {path} failed [{resp.status}]: {txt}")
            return await resp.json()

    async def fetch_me(self) -> dict[str, Any]:
        me = await self._api_get("/auth/me")
        self._me = me
        return me

    async def channels(self) -> List[Channel]:
        payload = await self._api_get("/nova/api/channels")
        return [Channel(**c) for c in payload]

    async def users(self) -> List[User]:
        payload = await self._api_get("/nova/api/users")
        return [User(**u) for u in payload]

    async def channel_messages(self, channel_id: str, limit: int = 50) -> list[dict[str, Any]]:
        return await self._api_get(f"/nova/api/channels/{channel_id}/messages?limit={limit}")

    async def send_channel_message(self, channel_id: str, content: str) -> dict[str, Any]:
        return await self._api_post(f"/nova/api/channels/{channel_id}/messages", {"content": content})

    async def send_dm_to_user(self, user_id: str, content: str) -> dict[str, Any]:
        return await self._api_post(f"/nova/api/dm/{user_id}/send", {"content": content})

    async def react_message(self, message_id: str, emoji: str) -> dict[str, Any]:
        return await self._api_post(f"/nova/api/messages/{message_id}/react", {"emoji": emoji})

    async def send_dm_thread(self, thread_id: str, content: str) -> None:
        """Send in an existing DM thread over WebSocket (no REST endpoint exists for thread send)."""
        if self._ws is None or self._ws.closed:
            raise RuntimeError("WebSocket is not connected")
        await self._ws.send_json({"type": "dm", "threadId": thread_id, "content": content})

    async def send_typing_channel(self, channel_id: str) -> None:
        if self._ws is None or self._ws.closed:
            return
        await self._ws.send_json({"type": "typing", "channelId": channel_id})

    async def send_typing_thread(self, thread_id: str) -> None:
        if self._ws is None or self._ws.closed:
            return
        await self._ws.send_json({"type": "typing", "threadId": thread_id})

    async def _connect_ws_once(self) -> None:
        if self._http is None:
            raise RuntimeError("Client not started")

        ws_scheme = "wss" if self.base_url.startswith("https://") else "ws"
        host = self.base_url.split("://", 1)[1]
        ws_url = f"{ws_scheme}://{host}/nova/ws"

        async with self._http.ws_connect(ws_url, headers=self._cookie_header()) as ws:
            self._ws = ws
            await self._dispatch("on_ws_open")

            async for msg in ws:
                if msg.type == aiohttp.WSMsgType.TEXT:
                    await self._handle_ws_payload(msg.json())
                elif msg.type == aiohttp.WSMsgType.ERROR:
                    raise RuntimeError(f"WebSocket error: {ws.exception()}")
                elif msg.type in (aiohttp.WSMsgType.CLOSE, aiohttp.WSMsgType.CLOSED):
                    break

    async def _handle_ws_payload(self, payload: dict[str, Any]) -> None:
        t = payload.get("type")

        if t == "message":
            msg = payload.get("message", {})
            wrapped = Message(
                id=msg.get("id", ""),
                content=msg.get("content", ""),
                created_at=msg.get("created_at", ""),
                user_id=msg.get("user_id"),
                username=msg.get("username"),
                channel_id=msg.get("channel_id"),
            )
            await self._dispatch("on_message", wrapped)
            return

        if t == "dm":
            msg = payload.get("message", {})
            wrapped = Message(
                id=msg.get("id", ""),
                content=msg.get("content", ""),
                created_at=msg.get("created_at", ""),
                sender_id=msg.get("sender_id"),
                sender_username=msg.get("sender_username"),
                thread_id=payload.get("threadId") or msg.get("thread_id"),
            )
            await self._dispatch("on_dm", wrapped)
            return

        if t == "presence":
            await self._dispatch("on_presence", payload)
            return

        if t == "typing":
            await self._dispatch("on_typing", payload)
            return

        if t == "reaction":
            await self._dispatch("on_reaction", payload)
            return

        await self._dispatch("on_raw_event", payload)

    async def start(self) -> None:
        if self._running:
            return

        self._closed = False
        self._running = True
        self._http = aiohttp.ClientSession(timeout=aiohttp.ClientTimeout(total=20))

        try:
            await self.fetch_me()
            await self._dispatch("on_ready")

            while not self._closed:
                try:
                    await self._connect_ws_once()
                except Exception as exc:
                    await self._dispatch_error(exc, "websocket")
                    if self._closed:
                        break
                    await asyncio.sleep(self.reconnect_delay)
        finally:
            self._running = False
            if self._http and not self._http.closed:
                await self._http.close()
            self._http = None
            self._ws = None
            await self._dispatch("on_close")

    async def close(self) -> None:
        self._closed = True
        if self._ws and not self._ws.closed:
            await self._ws.close()
        if self._http and not self._http.closed:
            await self._http.close()

    def run(self) -> None:
        try:
            asyncio.run(self.start())
        except KeyboardInterrupt:
            pass


class Context:
    def __init__(self, bot: "NOVABot", message: Message) -> None:
        self.bot = bot
        self.message = message

    async def reply(self, content: str) -> None:
        if self.message.channel_id:
            await self.bot.send_channel_message(self.message.channel_id, content)
            return

        if self.message.thread_id:
            await self.bot.send_dm_thread(self.message.thread_id, content)
            return

        if self.message.sender_id:
            await self.bot.send_dm_to_user(self.message.sender_id, content)


class NOVABot(NOVAClient):
    def __init__(
        self,
        base_url: str,
        session_cookie: str,
        prefix: str = "!",
        cookie_name: str = "connect.sid",
        reconnect_delay: float = 2.0,
    ) -> None:
        super().__init__(
            base_url=base_url,
            session_cookie=session_cookie,
            cookie_name=cookie_name,
            reconnect_delay=reconnect_delay,
        )
        self.prefix = prefix
        self._commands: Dict[str, EventHandler] = {}
        self._handlers.setdefault("on_message", []).append(self._internal_message_router)

    def command(self, name: str) -> Callable[[EventHandler], EventHandler]:
        key = name.strip().lower()

        def deco(fn: EventHandler) -> EventHandler:
            self._commands[key] = fn
            return fn

        return deco

    async def _internal_message_router(self, msg: Message) -> None:
        # Internal router is bound to on_message via self.event() above.
        await self._dispatch("on_channel_message", msg)

        if not msg.content.startswith(self.prefix):
            return

        # Optional self-message guard.
        if self.me and msg.user_id == self.me.get("userId"):
            return

        body = msg.content[len(self.prefix):].strip()
        if not body:
            return

        parts = body.split()
        command_name = parts[0].lower()
        args = parts[1:]
        fn = self._commands.get(command_name)
        if not fn:
            return

        ctx = Context(self, msg)
        try:
            result = fn(ctx, *args)
            if inspect.isawaitable(result):
                await result
        except Exception as exc:
            await self._dispatch_error(exc, f"command:{command_name}")
