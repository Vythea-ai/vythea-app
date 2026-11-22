---
name: vythea
description: "Vythea itself: its commands, settings, chat apps, MCP servers, profiles and doctor. Load before answering how to use or set up Vythea."
---

# Vythea

You run inside Vythea, an agent with a window (`vythea gui`) and a terminal (`vythea`) that share one data folder. When the user asks how to use, set up or fix it, answer from this page and its three files, and give the command to type or the place in the window to go.

- `commands.md`: every `vythea` verb and flag: `-p` and its options, resuming, modes, models, worktrees, `cloud`, `wallet`, `mcp`, `acp`, `update`, `--uninstall`.
- `chat-apps.md`: Telegram, Discord and Slack, pairing, and `vythea serve`.
- `settings-and-data.md`: the data folder and its files, `settings.json`, environment variables, profiles, a project's own files.

## Command line

`vythea` opens a session in the terminal and `vythea gui` the window; `vythea -p "question"` asks once and prints the answer; `vythea setup` changes the first-run answers; `vythea doctor` checks everything, so suggest it first when something is wrong. Every other verb and flag is in `commands.md`.

## MCP servers

`vythea mcp add [options] <name> <command-or-url> [args...]` takes Claude Code's flags, so a README's `claude mcp add` line works with `vythea` in its place:

- `vythea mcp add files -- npx -y @modelcontextprotocol/server-filesystem ~/notes`
- `vythea mcp add --transport http sentry https://mcp.sentry.dev/mcp`
- `-e KEY=value` sets the command's environment, `-H "K: v"` a URL's header.

Then `vythea mcp list`, `get <name>`, `remove <name>`, and `login <name>` or `logout <name>` for one reached by URL. They are kept in the data folder's `mcp.json`; in a session `/mcp` shows them, and `/mcp reload` reads the file again.

From a session in the window or terminal, when asked, `vythea_setup` can do this itself: `mcp_add` (a name and the command line or https address), `mcp_remove`, `mcp_enable`, `mcp_disable`; `plugin_add` (a git URL or owner/repo), `plugin_enable`, `plugin_disable`; `skill_enable`, `skill_disable`. Each asks the user first and shows the whole command line a server would run. It has no place for an API key or header, so those the user adds on the MCP page, and a plugin's servers and code are allowed only by the user on the Plugins page.

## Slash commands in the terminal

- /help lists them all, /new or /clear starts afresh (ctrl+l clears the screen), /terminal-setup (shift+enter), /theme (colours), /vim (vim keys), /keybindings (move keys), /exit leaves.
- Model: /model (also image, speech or video models, and `/model side` for a cheaper one for summaries), /fallback (models to carry on with when one fails), /connect, /limits (what is left of each plan), /effort, /thinking, /style (reply styles).
- Asking first: /mode, /plan, /decisions (auto mode's judge), /permissions (kept allow rules), /verify (test after edits), /format (its formatter after edits), /coauthor (Vythea as co-author of commits).
- Account: /login, /logout, /whoami, /team (bill hosted models to a team).
- Web and voice: /search (web search; /search services sets up the user's own), /voice (read replies aloud), /notify (say when a reply ends or needs the user).
- Sessions: /resume, /branch (a copy), /background (leave it working), /rename, /rewind, /undo (any message), /redo, /retry, /compact (sum up old turns), /context, /diff, /where, /window, /status, /statusline, /export, /import (Claude Code, Codex), /btw (an aside), /recap, /copy, /image, /preview, /ide (VS Code), /usage, /insights.
- Long work: /goal (keep working until a goal is met, `--check <command>`), /loop (run a prompt every so often; /loop stop), /heartbeat (look in on this session every few minutes), /jobs (background commands), /agents (helpers), /workflows, /crew (workflows of helpers for big tasks).
- Cloud: ask in chat to run or host something (a bot 24/7), check it or take it down.
- Keeping and adding: /remember (keep something now, or change when this chat saves), /memory (`/memory edit, log or restore user|memory|project|soul`), /memory-review, /retention (delete old sessions), /init (write AGENTS.md), /skills, /mcp, /plugins, /hooks, /tools, /tool-search, /lsp (language servers check edits).

## The window

The sidebar has New session, Capabilities (tabs Skills, Tools and MCP), Activity, Plugins, Messaging, Artifacts, Scheduled jobs, Wallet and Workflows. The account menu has Account, Usage, Settings, Profile, Set up Vythea, Guide, and signing in or out. Settings has Account, Usage, Gateways, Search, Model, Appearance, Thinking, Memory, Privacy, Wallet, Shortcuts, Plugins and About.

## Skills, profiles and schedules

- Skills: `/skills` switches them on or off, `/skills <name>` puts one in the message box, and `/skills add <owner/repo, link or path>` installs one (`anthropics/skills` has Word, Excel, PowerPoint and PDF ones). This one comes with Vythea and can be switched off; a skill of the user's called `vythea` takes its place, and Copy to edit on the window's Skills page makes one from this.
- Profiles: separate data folders, switched or made (empty, or cloned) under Profile in the window's account menu; see `settings-and-data.md`.
- Schedules: a prompt that runs on a timer, made with New job on the Scheduled jobs page, or by asking in a chat (added once the user says yes). They run while the window, the terminal or `vythea serve` is open.

If this skill is silent, say so and web_fetch https://vythea.com/docs; never invent a menu, setting or command.
