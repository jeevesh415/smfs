# supermemory-bash (Python)

A virtual bash environment for AI agents, backed by your [Supermemory](https://supermemory.ai) container. Files persist across sessions, and a built-in `sgrep` command does semantic search across the entire filesystem.

## Install

```bash
pip install supermemory-bash
```

You'll need a Supermemory API key. Get one at https://supermemory.ai.

## Quickstart

```python
import asyncio
from supermemory_bash import create_bash

async def main():
    result = await create_bash(
        api_key="sm-...",
        container_tag="user_42",
    )
    bash = result.bash

    # Run any shell command:
    r = await bash.exec("echo 'hello' > /a.md && cat /a.md")
    print(r.stdout)  # "hello\n"

    # Files persist across sessions:
    r2 = await bash.exec("cat /a.md")
    print(r2.stdout)  # "hello\n"

    # Semantic search across the whole container:
    r3 = await bash.exec("sgrep 'authentication tokens'")
    print(r3.stdout)

asyncio.run(main())
```

## Hand the bash tool to your LLM

`create_bash` returns a `tool_description` string ready to drop into your tool schema.

### Anthropic

```python
import anthropic
from supermemory_bash import create_bash

result = await create_bash(api_key="sm-...", container_tag="user_42")
bash, tool_description = result.bash, result.tool_description

client = anthropic.Anthropic()
response = client.messages.create(
    model="claude-sonnet-4-20250514",
    max_tokens=4096,
    tools=[{
        "name": "bash",
        "description": tool_description,
        "input_schema": {
            "type": "object",
            "properties": {"cmd": {"type": "string"}},
            "required": ["cmd"],
        },
    }],
    messages=[{"role": "user", "content": "Find my notes about authentication."}],
)

# In your tool-use loop, call `await bash.exec(cmd)` and feed the result back.
```

### OpenAI

```python
from openai import OpenAI
from supermemory_bash import create_bash

result = await create_bash(api_key="sm-...", container_tag="user_42")
bash, tool_description = result.bash, result.tool_description

client = OpenAI()
response = client.chat.completions.create(
    model="gpt-4o",
    messages=[{"role": "user", "content": "Search my notes for auth."}],
    tools=[{
        "type": "function",
        "function": {
            "name": "bash",
            "description": tool_description,
            "parameters": {
                "type": "object",
                "properties": {"cmd": {"type": "string"}},
                "required": ["cmd"],
            },
        },
    }],
)
```

## Options

```python
await create_bash(
    api_key="sm-...",
    container_tag="user_42",        # one container per user / project
    base_url=None,                  # API override
    eager_load=True,                # warm path index at construction
    eager_content=True,             # also warm content cache
    cache_ttl_ms=150_000,           # 2.5 min. None = never expires. 0 = no cache.
    cwd="/home/user",               # default working directory
    env=None,                       # extra environment variables
)
```

For very large containers (10k+ docs), set `eager_content=False` to skip the content warm and pay HTTP per `cat`. Path resolution stays warm.

`cache_ttl_ms` controls how long the in-memory content cache trusts itself. The default (2.5 min) assumes other writers exist. Single-writer apps can pass `None` for max speed.

## Supported commands

The built-in shell interpreter handles the commands agents use most:

- **Files**: `cat`, `head`, `tail`, `touch`, `stat`, `tee`
- **Directories**: `ls`, `mkdir`, `rmdir`, `pwd`, `cd`
- **Management**: `rm`, `mv`, `cp`
- **Search**: `grep` (regex), `sgrep` (semantic)
- **Text**: `echo`, `printf`, `wc`, `sort`, `uniq`, `sed`, `cut`, `tr`
- **Utility**: `find`, `test`/`[`, `basename`, `dirname`, `seq`, `date`, `true`, `false`
- **Operators**: pipes (`|`), redirects (`>`, `>>`), chaining (`&&`, `||`, `;`), variables (`$VAR`)

## What's not supported

- `chmod`, `utimes`, symlinks — Supermemory has no permission/symlink model.
- `/dev/null` redirects — not a real device.
- For loops, while loops, if/then/fi — use `&&` / `||` chaining instead.
- Binary uploads — content is text-extracted server-side.

## License

MIT
