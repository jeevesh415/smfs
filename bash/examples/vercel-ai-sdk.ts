// Run with:
//   SUPERMEMORY_API_KEY=... ANTHROPIC_API_KEY=... bun run examples/vercel-ai-sdk.ts
//
// In a downstream project the import is:
//   import { createBash } from "@supermemory/bash";

import { anthropic } from "@ai-sdk/anthropic";
import { generateText, stepCountIs, tool } from "ai";
import { z } from "zod";
import { createBash } from "../src/index.js";

const SUPERMEMORY_API_KEY = process.env.SUPERMEMORY_API_KEY;
const ANTHROPIC_API_KEY = process.env.ANTHROPIC_API_KEY;
if (!SUPERMEMORY_API_KEY) throw new Error("SUPERMEMORY_API_KEY is required");
if (!ANTHROPIC_API_KEY) throw new Error("ANTHROPIC_API_KEY is required");

const CONTAINER_TAG = process.env.CONTAINER_TAG ?? "agent_memory";
const MODEL = process.env.ANTHROPIC_MODEL ?? "claude-sonnet-4-5-20250929";
const PROMPT =
  process.env.PROMPT ??
  "Create /notes/vercel.txt containing 'Hello from Vercel AI SDK!', then cat it back to confirm.";

const { bash, toolDescription } = await createBash({
  apiKey: SUPERMEMORY_API_KEY,
  containerTag: CONTAINER_TAG,
});

const result = await generateText({
  model: anthropic(MODEL),
  prompt: PROMPT,
  tools: {
    bash: tool({
      description: toolDescription,
      inputSchema: z.object({ cmd: z.string().describe("The bash command to execute.") }),
      execute: async ({ cmd }) => {
        console.log(`[bash] $ ${cmd}`);
        const r = await bash.exec(cmd);
        return { stdout: r.stdout, stderr: r.stderr, exitCode: r.exitCode };
      },
    }),
  },
  stopWhen: stepCountIs(8),
});

console.log(`\n[assistant] ${result.text}\n`);
