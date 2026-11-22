# The command line

Every `vythea` verb and flag. `vythea --help` prints the same list.

- `vythea` opens a conversation in the terminal. `--memory when-asked|ask|auto` sets when that session saves to memory, `--memory-scope auto|project|chat` where it keeps memory and what of it loads, and `--private` starts a private chat whose notes stay its own. `--add-dir <folder>` (or /add-dir) lets it work in another folder too.
- `vythea cloud "task"` runs it on Vythea's servers (Pro, from credit); `--repo <url>`, `--tier light|normal|heavy`, `--thread <id>`; `vythea cloud list|show|stop|relay|apps|logs|down|up`.
- `vythea wallet` (/wallet): the crypto wallet, see `--help`; sends and keys need Windows Hello or Touch ID.
- `vythea local` (/local): on-device models, adding from Hugging Face.
- `vythea -p "question"` (`--print`, or just `vythea "question"`) asks once, prints the answer and exits. With -p: `--output text|json|stream-json`, `--max-rounds <n>` (tool rounds a message gets, 100 unless set), `--tools a,b` (only those tools), `--budget <tokens>` (stops once that many are spent), `--input-format stream-json` (messages, interrupts and answers as JSON lines on stdin), `--permission-prompt-tool stdio` (asks stdin about each call that needs a yes), `--json-schema <file>` (answers as JSON that the schema checks; a new session only). `--dump-config` prints the settings and limits a -p run would get, as JSON. Its environment variables are in `settings-and-data.md`.
- `vythea --continue` (`-c`) picks up this folder's latest session, `vythea --resume` (`-r`) offers the recent ones, `--resume <id>` the one `/where` names. `--fork-session` carries on in a copy, `--name <name>` names it. `--worktree` starts a new session in a git worktree of its own, on a branch made from HEAD, leaving the checkout alone; not with -c or -r.
- `vythea --bg "task"` works on, terminal closed; `vythea sessions` lists those, `vythea attach <id>` watches and answers one, `vythea logs <id>`, `vythea stop <id>`.
- `--mode <mode>`: default (asks before writing or running), accept-edits (project edits run, commands ask), plan (reads and searches only), auto (edits run, the user's model judges the rest), dont-ask (runs only what rules allow), bypass (runs without asking, bar risky calls and the user's rules). `--yes` is bypass, `--plan` is plan.
- `--system-prompt <text>`/`--system-prompt-file <file>` replace Vythea's own instructions for the run; `--append-system-prompt`/`--append-system-prompt-file` add to them.
- `--model <id>` with `--gateway <name>` picks this session's model, and `--effort <level>` (or auto) how hard it thinks; the defaults stay as they are.
- `vythea gui` opens the window.
- `vythea serve` answers the chat apps and runs the schedules with no window.
- `vythea setup` asks for a name, sign-in, provider, model and what to install; run it again to change any answer.
- `vythea update` installs the newest version.
- `vythea doctor` checks the version, system, data folder, settings, gateway, model, sign-in, keys, logs and last crash. It exits 1 on any problem and never prints a key; `--offline` leaves the network alone. Suggest it first when something is wrong.
- `vythea mcp` manages MCP servers; `vythea mcp serve` offers Vythea's browser, media and search tools to another agent over stdio (`--yes` runs them unasked, `--allow-turn` adds an agent turn).
- `vythea acp` serves an editor over the Agent Client Protocol.
- `vythea --uninstall` takes it off again. `--help` (`-h`), `--version` (`-V`).
