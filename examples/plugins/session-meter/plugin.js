// session-meter: what each turn used -- tokens, cost, time -- in the status
// bar, and a word to the model when it keeps making a call that fails the
// same way, which is how a run burns its budget going nowhere.
const total = { turns: 0, input: 0, output: 0, cost: 0, ms: 0, failures: 0 };
// This turn's tokens so far, and its failed calls, by conversation.
const live = new Map();
const failures = new Map();

const tokens = (n) => (n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n));
const dollars = (cost) => (cost ? ` · $${cost.toFixed(3)}` : "");

vythea.on("model.reply", (reply) => {
  const used = (live.get(reply.conversation) ?? 0) + reply.input_tokens + reply.output_tokens;
  live.set(reply.conversation, used);
  vythea.status(`${tokens(used)} tokens so far`);
});

vythea.on("turn.end", (turn) => {
  live.delete(turn.conversation);
  failures.delete(turn.conversation);
  total.turns += 1;
  total.input += turn.input_tokens;
  total.output += turn.output_tokens;
  total.cost += turn.cost ?? 0;
  total.ms += turn.ms;
  total.failures += turn.tool_failures;
  const seconds = (turn.ms / 1000).toFixed(1);
  vythea.status(`${tokens(turn.input_tokens + turn.output_tokens)} tokens${dollars(turn.cost)} · ${seconds} s`);
});

vythea.on("tool.failed", (call) => {
  const seen = failures.get(call.conversation) ?? new Map();
  failures.set(call.conversation, seen);
  const key = `${call.tool}\n${String(call.error ?? "").slice(0, 300)}`;
  const times = (seen.get(key) ?? 0) + 1;
  seen.set(key, times);
  if (times >= Number(vythea.settings.get("repeats") ?? 2)) {
    return `${call.tool} has failed ${times} times this turn with the same error. Do not repeat it: read the error, change the approach, or ask the user.`;
  }
});

vythea.action("meter", () =>
  `${total.turns} turns · ${tokens(total.input)} in · ${tokens(total.output)} out${dollars(total.cost)}` +
  ` · ${(total.ms / 1000).toFixed(0)} s · ${total.failures} failed calls`,
);
