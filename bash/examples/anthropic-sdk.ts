// Run with:
//   SUPERMEMORY_API_KEY=... ANTHROPIC_API_KEY=... bun run examples/anthropic-sdk.ts
//
// In a downstream project the import is:
//   import { createBash } from "@supermemory/bash";

import Anthropic from "@anthropic-ai/sdk";
import { createBash } from "../src/index.js";

const SUPERMEMORY_API_KEY = process.env.SUPERMEMORY_API_KEY;
const ANTHROPIC_API_KEY = process.env.ANTHROPIC_API_KEY;
if (!SUPERMEMORY_API_KEY) throw new Error("SUPERMEMORY_API_KEY is required");
if (!ANTHROPIC_API_KEY) throw new Error("ANTHROPIC_API_KEY is required");

const CONTAINER_TAG = process.env.CONTAINER_TAG ?? "agent_memory";
const MODEL = process.env.ANTHROPIC_MODEL ?? "claude-sonnet-4-5-20250929";
const PROMPT =
  process.env.PROMPT ??
  "Create /notes/anthropic.txt containing 'Hello from Anthropic SDK!', then cat it back to confirm.";

const { bash, toolDescription } = await createBash({
  apiKey: SUPERMEMORY_API_KEY,
  containerTag: CONTAINER_TAG,
});

const client = new Anthropic({ apiKey: ANTHROPIC_API_KEY });

const tools: Anthropic.Tool[] = [
  {
    name: "bash",
    description: toolDescription,
    input_schema: {
      type: "object",
      properties: {
        cmd: { type: "string", description: "The bash command to execute." },
      },
      required: ["cmd"],
    },
  },
];

const messages: Anthropic.MessageParam[] = [{ role: "user", content: PROMPT }];

for (let step = 0; step < 8; step++) {
  const response = await client.messages.create({
    model: MODEL,
    max_tokens: 1024,
    tools,
    messages,
  });

  messages.push({ role: "assistant", content: response.content });

  if (response.stop_reason !== "tool_use") {
    const text = response.content
      .filter((b): b is Anthropic.TextBlock => b.type === "text")
      .map((b) => b.text)
      .join("\n");
    console.log(`\n[assistant] ${text}\n`);
    break;
  }

  const toolResults: Anthropic.ToolResultBlockParam[] = [];
  for (const block of response.content) {
    if (block.type !== "tool_use") continue;
    const cmd = (block.input as { cmd: string }).cmd;
    console.log(`[bash] $ ${cmd}`);
    const result = await bash.exec(cmd);
    const out = [
      result.stdout.length > 0 ? result.stdout : null,
      result.stderr.length > 0 ? `[stderr]\n${result.stderr}` : null,
      `[exit ${result.exitCode}]`,
    ]
      .filter(Boolean)
      .join("\n");
    toolResults.push({
      type: "tool_result",
      tool_use_id: block.id,
      content: out,
      is_error: result.exitCode !== 0,
    });
  }
  messages.push({ role: "user", content: toolResults });
}
