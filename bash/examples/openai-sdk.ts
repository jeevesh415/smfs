// Run with:
//   SUPERMEMORY_API_KEY=... OPENAI_API_KEY=... bun run examples/openai-sdk.ts
//
// In a downstream project the import is:
//   import { createBash } from "@supermemory/bash";

import OpenAI from "openai";
import { createBash } from "../src/index.js";

const SUPERMEMORY_API_KEY = process.env.SUPERMEMORY_API_KEY;
const OPENAI_API_KEY = process.env.OPENAI_API_KEY;
if (!SUPERMEMORY_API_KEY) throw new Error("SUPERMEMORY_API_KEY is required");
if (!OPENAI_API_KEY) throw new Error("OPENAI_API_KEY is required");

const CONTAINER_TAG = process.env.CONTAINER_TAG ?? "agent_memory";
const MODEL = process.env.OPENAI_MODEL ?? "gpt-4o";
const PROMPT =
  process.env.PROMPT ??
  "Create /notes/openai.txt containing 'Hello from OpenAI SDK!', then cat it back to confirm.";

const { bash, toolDescription } = await createBash({
  apiKey: SUPERMEMORY_API_KEY,
  containerTag: CONTAINER_TAG,
});

const client = new OpenAI({ apiKey: OPENAI_API_KEY });

const tools: OpenAI.Chat.Completions.ChatCompletionTool[] = [
  {
    type: "function",
    function: {
      name: "bash",
      description: toolDescription,
      parameters: {
        type: "object",
        properties: {
          cmd: { type: "string", description: "The bash command to execute." },
        },
        required: ["cmd"],
        additionalProperties: false,
      },
    },
  },
];

const messages: OpenAI.Chat.Completions.ChatCompletionMessageParam[] = [
  { role: "user", content: PROMPT },
];

for (let step = 0; step < 8; step++) {
  const response = await client.chat.completions.create({
    model: MODEL,
    messages,
    tools,
  });

  const msg = response.choices[0]?.message;
  if (!msg) throw new Error("OpenAI returned no message");
  messages.push(msg);

  if (!msg.tool_calls || msg.tool_calls.length === 0) {
    console.log(`\n[assistant] ${msg.content ?? ""}\n`);
    break;
  }

  for (const call of msg.tool_calls) {
    if (call.type !== "function") continue;
    const args = JSON.parse(call.function.arguments) as { cmd: string };
    console.log(`[bash] $ ${args.cmd}`);
    const result = await bash.exec(args.cmd);
    const out = [
      result.stdout.length > 0 ? result.stdout : null,
      result.stderr.length > 0 ? `[stderr]\n${result.stderr}` : null,
      `[exit ${result.exitCode}]`,
    ]
      .filter(Boolean)
      .join("\n");
    messages.push({ role: "tool", tool_call_id: call.id, content: out });
  }
}
