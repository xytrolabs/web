import os

from NOVA import NOVABot

BOT = NOVABot(
    base_url=os.getenv("NOVA_BASE_URL", "https://nova.xytro.site"),
    session_cookie=os.environ["NOVA_SESSION_COOKIE"],
    prefix="!",
)


@BOT.event
async def on_ready():
    me = BOT.me or {}
    print(f"Connected as: {me.get('username', 'unknown')}")


@BOT.event
async def on_dm(message):
    # Example: auto-acknowledge DMs containing 'help'.
    if "help" in message.content.lower() and message.thread_id:
        await BOT.send_dm_thread(message.thread_id, "How can I help you?")


@BOT.command("ping")
async def ping(ctx, *args):
    await ctx.reply("pong")


@BOT.command("say")
async def say(ctx, *args):
    if not args:
        await ctx.reply("Usage: !say <text>")
        return
    await ctx.reply(" ".join(args))


if __name__ == "__main__":
    BOT.run()
