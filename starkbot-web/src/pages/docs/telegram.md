---
name: Telegram Integration
---

Set up a Telegram bot with @BotFather and connect it to StarkBot.

## Create Your Bot

### Step 1: BotFather

1. Open Telegram, search for **@BotFather**
2. Send `/newbot`
3. Enter a display name (e.g., "My StarkBot")
4. Enter a username ending in `bot` (e.g., `mystarkbot_bot`)
5. Save your bot token

Token format: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`

> **Keep this secure.** The token grants full control over your bot.

### Step 2: Add to StarkBot

1. Go to **Channels** in the dashboard
2. Click **Add Channel**
3. Select **Telegram**
4. Paste your bot token
5. Save and start

---

## Bot Configuration

Customize your bot with BotFather commands:

| Command | Description |
|---------|-------------|
| `/setname` | Change display name |
| `/setdescription` | Set bot description |
| `/setabouttext` | Set "About" text |
| `/setuserpic` | Upload profile picture |
| `/setjoingroups` | Allow/disallow groups |
| `/setprivacy` | Control message visibility |

---

## Privacy Mode

By default, bots in groups only receive:
- Messages starting with `/`
- Replies to the bot
- @mentions of the bot

### To receive all messages:

**Option 1: Disable Privacy**
1. Send `/setprivacy` to @BotFather
2. Select your bot
3. Choose **Disable**
4. Remove and re-add bot to groups

**Option 2: Admin Status**
- Make the bot an admin in the group

---

## Testing

1. Start the channel in StarkBot dashboard
2. Find your bot on Telegram by username
3. Send `/start` or any message
4. Verify response appears

Check the dashboard logs if no response.

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Bot not responding | Check channel is running in dashboard |
| "Unauthorized" error | Verify token is correct |
| No group messages | Check privacy mode or make bot admin |
| Token compromised | Use `/revoke` in BotFather |

---

## Resources

- [Telegram Bot API](https://core.telegram.org/bots/api)
- [@BotFather](https://t.me/BotFather)
- [Privacy Mode](https://core.telegram.org/bots#privacy-mode)
