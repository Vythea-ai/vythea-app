//! The `vythea_setup` tool: setting up the Telegram chat app from a session,
//! as the user asks, instead of sending them to click through the window.
//!
//! What it never does is take a secret through the chat. The bot token is
//! read from the clipboard by this tool itself, checked with Telegram, and
//! kept in the vault like any token; the model is told only that it was
//! saved and which bot it belongs to. The card the user answers shows the
//! token only masked.
//!
//! Every call asks the user first, each time. Like `vythea_info`, it is
//! offered only where the user alone reads the answers: not in a chat others
//! read, not in a scheduled run, and not in a run nobody watches.
//!
//! The window keeps the chat apps' settings and the pairing code in memory,
//! so a change made here is announced to it on `changes` and it takes the
//! change up: it reads the settings again and starts listening, or keeps the
//! new pairing code. With no window, the settings file is what changed.

use crate::chat::Connection;
use crate::messaging::{self, pairing};
use crate::tools::{Tool, ToolContext};
use serde_json::{Value, json};
use std::sync::OnceLock;
use std::time::Duration;

/// The tool's name.
pub(crate) const VYTHEA_SETUP: &str = "vythea_setup";

/// Why a call is refused where someone other than the user could read it.
const WITHHELD: &str = "vythea_setup works only in a session the user \
     watches, not in a chat others read, a scheduled run or a run nobody watches.";

/// What a call can do.
const ACTIONS: [&str; 15] = [
    "telegram_token",
    "telegram_pair",
    "telegram_enable",
    "telegram_disable",
    "telegram_allow_tools",
    "telegram_unpair",
    "mcp_add",
    "mcp_remove",
    "mcp_enable",
    "mcp_disable",
    "plugin_add",
    "plugin_enable",
    "plugin_disable",
    "skill_enable",
    "skill_disable",
];

/// How long Telegram gets to say whose token this is.
const ASK_TIME: Duration = Duration::from_secs(10);

/// What the window is told after a change made from a session.
pub enum Change {
    /// The settings file changed: read it again and listen as it says.
    Reload,
    /// A pairing code was made: keep it, to try the next message against.
    Pair(pairing::Code),
}

type Channel = (async_channel::Sender<Change>, async_channel::Receiver<Change>);

fn channel() -> &'static Channel {
    static CHANNEL: OnceLock<Channel> = OnceLock::new();
    CHANNEL.get_or_init(async_channel::unbounded)
}

/// The changes made from sessions, for the window to take up.
pub fn changes() -> async_channel::Receiver<Change> {
    channel().1.clone()
}

fn announce(change: Change) {
    let _ = channel().0.try_send(change);
}

/// The model's way to set up Telegram; each call asks the user first.
pub struct VytheaSetup;

impl Tool for VytheaSetup {
    fn name(&self) -> &'static str {
        VYTHEA_SETUP
    }
    fn description(&self) -> &'static str {
        "Set up Vythea itself when the user asks. Telegram: telegram_token saves the bot token the user copied to the clipboard (it never passes through this chat), telegram_pair makes a six-digit code to send to the bot, and the rest switch Telegram on or off, allow tools for the paired chat, or unpair it. MCP servers: mcp_add (name and target: the command line that starts it, or its https address), mcp_remove, mcp_enable, mcp_disable. Plugins: plugin_add (target: a git URL or owner/repo), plugin_enable, plugin_disable; a plugin's servers and code stay off until the user allows them in the Plugins page. Skills: skill_enable, skill_disable. Secrets such as API keys or headers are never taken here: the user adds them in the window. Each call asks the user first."
    }
    fn parameters(&self) -> Value {
        json!({"type": "object", "properties": {
            "action": {"type": "string", "enum": ACTIONS, "description": "What to do."},
            "value": {"type": "boolean", "description": "With telegram_allow_tools: true lets the paired chat use tools that change things."},
            "name": {"type": "string", "description": "The MCP server, plugin or skill the action is about; for mcp_add, the name to give it (letters, digits, - and _)."},
            "target": {"type": "string", "description": "With mcp_add: the command line that starts the server, or its https address. With plugin_add: a git URL or owner/repo."}
        }, "required": ["action"]})
    }
    // Offered as `vythea_info` is, so a scheduled run, whose answer goes to
    // every chat app set up, never has it.
    fn offered(&self, connection: &Connection) -> bool {
        crate::schedule::running(connection.conversation).is_none()
    }
    fn needs_approval(&self) -> bool {
        true
    }
    fn asked_each_time(&self) -> bool {
        true
    }
    fn mutates(&self) -> bool {
        true
    }
    fn preview(&self, args: &Value, _ctx: &ToolContext) -> String {
        describe(args)
    }
    fn run(&self, args: &Value, ctx: &ToolContext) -> Result<String, String> {
        let connection = ctx.caller.connection.as_deref();
        if !connection
            .is_some_and(|connection| crate::info::read_by_the_user_alone(connection, ctx.caller.root))
        {
            return Err(WITHHELD.into());
        }
        let action = args["action"].as_str().unwrap_or_default().trim();
        if let Some(done) = manage(action, args) {
            return done;
        }
        let mut config = messaging::load();
        let told = match action {
            "telegram_token" => {
                let token = clipboard_token()?;
                let (bot, reads_all) = bot_name(&token)?;
                config.telegram.token = token;
                config.telegram.enabled = true;
                // Another bot is other chats to pair.
                config.telegram.unpair();
                messaging::save(&config)?;
                format!(
                    "Saved the token for @{bot} from the clipboard and switched Telegram on. \
                     The token is not shown here. Next: telegram_pair, then the user sends the \
                     code to the bot in Telegram.{}",
                    group_note(reads_all)
                )
            }
            "telegram_pair" => {
                if !config.telegram.ready() {
                    return Err(
                        "Telegram has no bot token yet, or is off: save a token first (telegram_token)."
                            .into(),
                    );
                }
                let code = pairing::Code::new()?;
                let digits = code.code.clone();
                announce(Change::Pair(code));
                return Ok(format!(
                    "Pairing code {digits}. Tell the user to send exactly these six digits to \
                     their bot in Telegram within {} minutes; the chat that sends it is paired. \
                     In a group the message is the bot's @name, then the code. A chat that is \
                     not paired gets no reply by design, so silence means the bot did not hear \
                     the message: the code ran out, or the group hides plain messages from the \
                     bot. A paired group turned into a supergroup stays paired under its new ID.",
                    pairing::LIFE.as_secs() / 60
                ));
            }
            "telegram_enable" => {
                if config.telegram.token.trim().is_empty() {
                    return Err("There is no bot token to switch on: save one first (telegram_token).".into());
                }
                config.telegram.enabled = true;
                messaging::save(&config)?;
                "Telegram is on.".to_owned()
            }
            "telegram_disable" => {
                config.telegram.enabled = false;
                messaging::save(&config)?;
                "Telegram is off. The token and the pairing are kept.".to_owned()
            }
            "telegram_allow_tools" => {
                let Some(allow) = args["value"].as_bool() else {
                    return Err("`value` is required: true or false.".into());
                };
                config.telegram.allow_tools = allow;
                messaging::save(&config)?;
                match allow {
                    true => "The paired chat may now use tools that change things.".to_owned(),
                    false => "The paired chat no longer uses tools that change things.".to_owned(),
                }
            }
            "telegram_unpair" => {
                config.telegram.unpair();
                messaging::save(&config)?;
                "Telegram is unpaired: no chat is answered until one is paired again.".to_owned()
            }
            "" => {
                return Err(format!("`action` is required: one of {}.", ACTIONS.join(", ")));
            }
            other => {
                return Err(format!(
                    "There is no action `{other}`: one of {}.",
                    ACTIONS.join(", ")
                ));
            }
        };
        announce(Change::Reload);
        Ok(told)
    }
}

/// The MCP, plugin and skill actions, which change what Vythea can do and
/// not the chat apps; `None` for any other action. A server's secrets (an
/// API key, a header) have no field here, so none passes through the chat;
/// and a plugin's servers and code are allowed only by the user, on the
/// Plugins page, never by this tool.
fn manage(action: &str, args: &Value) -> Option<Result<String, String>> {
    let name = args["name"].as_str().unwrap_or_default().trim();
    let target = args["target"].as_str().unwrap_or_default().trim();
    let named = || match name.is_empty() {
        true => Err("`name` is required.".to_owned()),
        false => Ok(name),
    };
    Some(match action {
        "mcp_add" => mcp_add(name, target),
        "mcp_remove" => named().and_then(|name| {
            crate::mcp::remove_server(name)?;
            crate::mcp::reload()?;
            Ok(format!("Removed the MCP server {name}."))
        }),
        "mcp_enable" | "mcp_disable" => named().and_then(|name| {
            let on = action == "mcp_enable";
            crate::mcp::set_enabled(name, on)?;
            Ok(format!("The MCP server {name} is {}.", word(on)))
        }),
        "plugin_add" => plugin_add(target),
        "plugin_enable" | "plugin_disable" => named().and_then(|name| {
            let on = action == "plugin_enable";
            crate::plugins::set_enabled(name, on)?;
            Ok(format!("The plugin {name} is {}.", word(on)))
        }),
        "skill_enable" | "skill_disable" => named().and_then(|name| {
            let on = action == "skill_enable";
            crate::skills::set_enabled(name, on)?;
            Ok(format!("The skill {name} is {}.", word(on)))
        }),
        _ => return None,
    })
}

fn word(on: bool) -> &'static str {
    match on {
        true => "on",
        false => "off",
    }
}

/// Add the MCP server `name` that `target` starts, or its address, with no
/// variables or headers: those are secrets, which the user adds in the window.
fn mcp_add(name: &str, target: &str) -> Result<String, String> {
    let draft = crate::mcp_paste::Draft {
        name: name.to_owned(),
        target: target.to_owned(),
        extras: String::new(),
    };
    let name = crate::mcp_paste::add(&draft)?;
    Ok(format!(
        "Added the MCP server {name} and started it; its tools show once it has answered. \
         If it needs an API key or a header, the user adds that on the MCP page of the \
         window: it is not taken here."
    ))
}

/// Install the plugin at `source`. It is cloned and scanned like any other,
/// and what it brings that runs code stays off until the user allows it.
fn plugin_add(source: &str) -> Result<String, String> {
    if source.is_empty() {
        return Err("`target` is required: a git URL or owner/repo.".into());
    }
    let plugin = crate::plugins::add(source)?;
    Ok(match plugin.allowed {
        true => format!("Installed the plugin {} and switched it on.", plugin.name),
        false => format!(
            "Installed the plugin {} and switched it on. Its MCP servers or code wait for the \
             user to allow them on the Plugins page, which only they can do.",
            plugin.name
        ),
    })
}

/// What the approval card says a call will do, without anything secret.
fn describe(args: &Value) -> String {
    let name = args["name"].as_str().unwrap_or_default().trim();
    let target = args["target"].as_str().unwrap_or_default().trim();
    match args["action"].as_str().unwrap_or_default().trim() {
        // The whole line, since it is what will run on this computer.
        "mcp_add" => format!("Add the MCP server {name}, which runs: {target}"),
        "mcp_remove" => format!("Remove the MCP server {name}"),
        "mcp_enable" => format!("Switch the MCP server {name} on"),
        "mcp_disable" => format!("Switch the MCP server {name} off"),
        "plugin_add" => format!(
            "Install the plugin from {target} (anything in it that runs code stays off until you allow it)"
        ),
        "plugin_enable" => format!("Switch the plugin {name} on"),
        "plugin_disable" => format!("Switch the plugin {name} off"),
        "skill_enable" => format!("Switch the skill {name} on"),
        "skill_disable" => format!("Switch the skill {name} off"),
        _ => describe_telegram(args),
    }
}

fn describe_telegram(args: &Value) -> String {
    match args["action"].as_str().unwrap_or_default().trim() {
        "telegram_token" => match clipboard_token() {
            Ok(token) => format!(
                "Save the Telegram bot token on your clipboard ({}) and switch Telegram on",
                masked(&token)
            ),
            Err(why) => format!("Save a Telegram bot token from your clipboard. {why}"),
        },
        "telegram_pair" => "Make a code to pair a Telegram chat with this agent".to_owned(),
        "telegram_enable" => "Switch Telegram on".to_owned(),
        "telegram_disable" => "Switch Telegram off".to_owned(),
        "telegram_allow_tools" => match args["value"].as_bool() {
            Some(true) => "Let the paired Telegram chat use tools that change things".to_owned(),
            _ => "Stop the paired Telegram chat from using tools that change things".to_owned(),
        },
        "telegram_unpair" => "Unpair the Telegram chat".to_owned(),
        other => format!("vythea_setup: {other}"),
    }
}

/// A token as the card shows it: its first four characters and last three.
fn masked(token: &str) -> String {
    let count = token.chars().count();
    let head: String = token.chars().take(4).collect();
    let tail: String = token.chars().skip(count.saturating_sub(3)).collect();
    format!("{head}…{tail}")
}

/// Whether `text` is shaped like a Telegram bot token: the bot's number, a
/// colon, and a long secret of letters, digits, `_` and `-`.
fn looks_like_a_bot_token(text: &str) -> bool {
    let Some((id, secret)) = text.split_once(':') else {
        return false;
    };
    (5..=14).contains(&id.len())
        && id.bytes().all(|b| b.is_ascii_digit())
        && secret.len() >= 25
        && secret
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// The text on the clipboard, trimmed, asked of the system's own tool.
fn clipboard() -> Result<String, String> {
    use std::process::Command;
    let attempts: Vec<(&str, Vec<&str>)> = if cfg!(windows) {
        vec![(
            "powershell",
            vec!["-NoProfile", "-NonInteractive", "-Command", "Get-Clipboard -Raw"],
        )]
    } else if cfg!(target_os = "macos") {
        vec![("pbpaste", vec![])]
    } else {
        vec![
            ("wl-paste", vec!["--no-newline"]),
            ("xclip", vec!["-selection", "clipboard", "-o"]),
            ("xsel", vec!["--clipboard", "--output"]),
        ]
    };
    for (program, arguments) in attempts {
        let mut command = Command::new(program);
        command.args(arguments);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // No console window flashing up from the window app.
            command.creation_flags(0x0800_0000);
        }
        if let Ok(output) = command.output()
            && output.status.success()
        {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
        }
    }
    Err("The clipboard could not be read.".into())
}

/// The bot token on the clipboard, or why there is none.
fn clipboard_token() -> Result<String, String> {
    let text = clipboard()?;
    if looks_like_a_bot_token(&text) {
        Ok(text)
    } else {
        Err("The clipboard holds no Telegram bot token: copy the one BotFather gave, \
             which looks like 123456789:AA… and has no spaces."
            .into())
    }
}

/// The bot's @name, which Telegram says only for a token that works. Nothing
/// of the request, the token's place in it included, is passed on in an error.
fn bot_name(token: &str) -> Result<(String, Option<bool>), String> {
    let unreachable = || "Telegram could not be reached to check the token.".to_owned();
    let client = reqwest::blocking::Client::builder()
        .timeout(ASK_TIME)
        .build()
        .map_err(|_| unreachable())?;
    let body = client
        .get(format!("https://api.telegram.org/bot{token}/getMe"))
        .send()
        .map_err(|_| unreachable())?
        .text()
        .map_err(|_| unreachable())?;
    let answer: Value = serde_json::from_str(&body).map_err(|_| unreachable())?;
    if answer["ok"].as_bool() != Some(true) {
        return Err("Telegram does not accept that token: copy it again from BotFather.".into());
    }
    let name = answer["result"]["username"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "Telegram accepted the token but named no bot.".to_owned())?;
    Ok((name, answer["result"]["can_read_all_group_messages"].as_bool()))
}

/// What to tell about using the bot in a group. With Group Privacy on,
/// Telegram hands the bot only commands, mentions and replies to it, so a
/// plain message there is never heard; a bot that reads all of them has it
/// off. Said nothing about when Telegram did not say.
fn group_note(reads_all: Option<bool>) -> &'static str {
    match reads_all {
        Some(false) => {
            " This bot has Group Privacy on, so in a group it hears only commands, @mentions \
             and replies to it: the user mentions it in the group (@bot then the code). To \
             hear every message there, they turn Group Privacy off in BotFather (/mybots, \
             Bot Settings, Group Privacy) and add the bot to the group again."
        }
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_privacy_is_explained_only_when_telegram_says_it_is_on() {
        assert!(group_note(Some(false)).contains("Group Privacy"));
        assert_eq!(group_note(Some(true)), "");
        assert_eq!(group_note(None), "");
    }

    #[test]
    fn the_manage_actions_are_told_apart_from_telegram_and_check_their_input_first() {
        // Not Telegram's, so those go on to the chat apps' code.
        assert!(manage("telegram_pair", &json!({})).is_none());
        assert!(manage("nothing_of_the_kind", &json!({})).is_none());
        // Refused before anything is written or run.
        let bad_name = json!({"action": "mcp_add", "name": "bad name!", "target": "npx -y x"});
        assert!(manage("mcp_add", &bad_name).unwrap().is_err());
        let no_target = json!({"action": "mcp_add", "name": "ok", "target": ""});
        assert!(manage("mcp_add", &no_target).unwrap().is_err());
        for action in ["mcp_remove", "mcp_enable", "skill_disable", "plugin_enable"] {
            let said = manage(action, &json!({"action": action})).unwrap();
            assert_eq!(said, Err("`name` is required.".to_owned()), "{action}");
        }
        assert!(manage("plugin_add", &json!({})).unwrap().is_err());
    }

    #[test]
    fn the_card_shows_what_a_server_will_run_and_that_a_plugin_stays_off() {
        let add = describe(&json!({"action": "mcp_add", "name": "gh", "target": "npx -y srv"}));
        assert!(add.contains("npx -y srv") && add.contains("gh"));
        let plugin = describe(&json!({"action": "plugin_add", "target": "owner/repo"}));
        assert!(plugin.contains("owner/repo") && plugin.contains("until you allow it"));
        assert_eq!(
            describe(&json!({"action": "skill_disable", "name": "x"})),
            "Switch the skill x off"
        );
    }

    #[test]
    fn no_action_here_can_allow_a_plugin_or_take_a_secret() {
        assert!(!ACTIONS.iter().any(|a| a.contains("allow") && a.starts_with("plugin")));
        let schema = VytheaSetup.parameters();
        let fields: Vec<&str> = schema["properties"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert!(!fields.iter().any(|f| ["env", "headers", "token", "key"].contains(f)));
    }

    #[test]
    fn a_bot_token_has_a_number_a_colon_and_a_long_secret() {
        assert!(looks_like_a_bot_token(
            "123456789:AAEhBP0av28dFq-3xkCUrN4t_Zy1aBcDeFg"
        ));
        assert!(!looks_like_a_bot_token("123456789"));
        assert!(!looks_like_a_bot_token("123456789:short"));
        assert!(!looks_like_a_bot_token("abc:AAEhBP0av28dFq-3xkCUrN4t_Zy1aBcDeFg"));
        assert!(!looks_like_a_bot_token(
            "123456789:AAEhBP0av28dFq 3xkCUrN4t_Zy1aBcDeFg"
        ));
    }

    #[test]
    fn the_card_shows_a_token_only_masked() {
        let token = "123456789:AAEhBP0av28dFq-3xkCUrN4t_Zy1aBcDeFg";
        let shown = masked(token);
        assert_eq!(shown, "1234…eFg");
        assert!(!shown.contains("AAEhBP0av28"));
    }

    #[test]
    fn the_schema_names_every_action_and_asks_for_one() {
        let schema = VytheaSetup.parameters();
        let named: Vec<&str> = schema["properties"]["action"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .collect();
        assert_eq!(named, ACTIONS);
        assert_eq!(schema["required"][0], "action");
    }

    #[test]
    fn every_call_asks_and_none_runs_in_plan_mode() {
        assert!(VytheaSetup.needs_approval());
        assert!(VytheaSetup.asked_each_time());
        assert!(VytheaSetup.mutates());
    }

    #[test]
    fn the_card_words_each_action_without_a_secret() {
        // Not telegram_token, which reads the clipboard of whoever runs the tests.
        for action in ACTIONS.into_iter().filter(|a| *a != "telegram_token") {
            let words = describe(&json!({"action": action, "value": true}));
            assert!(!words.is_empty() && !words.contains("AAEh"), "{action}: {words}");
        }
    }
}
