# Agent SDK Pass-Through Analysis

date: 2026-03-18
question: can aclaude be additive to Claude Code, or does wrapping via the Agent SDK lose features?
relates-to: F13 (UX Parity and Enhancement vs Vanilla Claude Code)

## Summary

The Agent SDK is a coordination layer over the Claude Code subprocess, not
a reimplementation. aclaude inherits nearly all Claude Code capabilities
by default — but only if configured correctly. The critical finding: the
SDK defaults to **isolated mode** (no settings loaded) unless you explicitly
opt in via `settingSources`.

## What passes through (inherited for free)

- All built-in tools (Read, Edit, Bash, Glob, Grep, Write, Agent, etc.)
- Authentication (OAuth, API key, Bedrock, Vertex)
- Model inference and context management
- CLAUDE.md loading — **requires `settingSources: ['project', 'user', 'local']`**
- Project and user settings (.claude/settings.json, settings.local.json)
- MCP servers (stdio/sse/http managed by subprocess)
- Tool schemas and permission enforcement
- Session persistence and resume
- Context compaction
- File checkpointing (opt-in via `enableFileCheckpointing`)

## What aclaude adds (the value layer)

- Persona theming (100 theme rosters, immersion levels)
- TOML config with 5-layer merge chain
- tmux session management and statusline
- Context window usage tracking and visualization
- In-process JS hooks (replacing shell-based hooks)
- Self-updating binary distribution

## What the SDK does NOT pass through

| Feature | Impact | Mitigation |
|---------|--------|------------|
| Slash command invocation | Commands enumerable but not invocable via SDK message flow | Parse and route manually, or accept limitation |
| Shell-based hooks (.claude/settings) | User-configured shell hooks don't fire | aclaude's JS hooks replace these; document the difference |
| Interactive CLI commands (/clear, /continue) | CLI-specific UX not exposed | Implement equivalents (aclaude already has /quit, /usage) |
| Interactive permission dialogs | Allow/Deny/Always Allow UI not exposed | `permissionMode: 'default'` delegates to subprocess; `canUseTool()` callback available for custom logic |
| Real-time context window display | Context size only in final SDKResultMessage | Track per-turn via SDKAssistantMessage.message.usage (already implemented) |
| Error recovery UX | CLI shows user-friendly dialogs; SDK returns error subtypes | Must present authentication_failed, billing_error, rate_limit, etc. |

## SDK query() options surface (50+ fields)

Key options aclaude should expose or consider:

| Option | Current state | Action |
|--------|--------------|--------|
| `settingSources` | **Was missing** — fixed to `['project', 'user', 'local']` | Critical fix: without this, CLAUDE.md and settings don't load |
| `systemPrompt` | Set to persona prompt | Could use `{ type: 'preset', preset: 'claude_code', append: personaPrompt }` to layer on top of Claude Code's built-in prompt |
| `permissionMode` | 'default' | Could expose in TOML config |
| `maxBudgetUsd` | Not set | Could expose in TOML config for cost control |
| `enableFileCheckpointing` | Not set | Enables rewind; worth enabling |
| `sandbox` | Not configured | Could expose for security-conscious users |
| `additionalDirectories` | Not set | Could expose in config |
| `agents` | Not set | Could define custom subagents programmatically |
| `mcpServers` | Not set (inherited from Claude Code) | Could add aclaude-specific MCP servers |
| `maxThinkingTokens` | Not set | Could expose in config |
| `betas` | Not set | Could enable 1M context (`context-1m-2025-08-07`) |

## Hooks available (12 events)

1. PreToolUse — before tool execution (can intercept/modify)
2. PostToolUse — after successful tool execution
3. PostToolUseFailure — after tool failure
4. Notification — system notifications
5. UserPromptSubmit — prompt submission
6. SessionStart — session init (startup, resume, clear, compact)
7. SessionEnd — session termination (with reason)
8. Stop — stop state change
9. SubagentStart / SubagentStop — subagent lifecycle
10. PreCompact — before context compaction
11. PermissionRequest — permission prompt (can allow/deny/ask)

aclaude currently uses PreToolUse, PostToolUse, PostToolUseFailure,
SessionStart, and SessionEnd. The others are available for future use.

## Key finding: additive by default

With `settingSources` set correctly, aclaude is **additive by default**.
Claude Code's full configuration loads (CLAUDE.md, settings, MCP servers),
and aclaude's persona/config/hooks layer on top. Users get everything
vanilla Claude Code provides, plus aclaude's enhancements.

The SDK is designed for this pattern — it's a wrapper API, not a fork.
The subprocess is real Claude Code running with real tools. aclaude's
value is in the configuration, theming, and operational layer around it.

## systemPrompt architecture — RESOLVED

**Problem:** aclaude was replacing Claude Code's entire system prompt
with the persona prompt. This lost all built-in tool instructions,
safety guidelines, and capabilities. Users got a worse Claude Code.

**Fix:** use the SDK's preset system prompt with append:
```typescript
systemPrompt: {
  type: 'preset',
  preset: 'claude_code',
  append: personaSystemPrompt
}
```

This layers the persona on top of Claude Code's own prompt. The user
gets everything vanilla Claude Code provides, plus the persona theming.
With immersion "none", the preset is used with no append — identical
to vanilla Claude Code.

**Finding:** this was silently degrading aclaude from day one. The
persona prompt included "You are a software engineering assistant"
as a poor substitute for Claude Code's full system prompt. The preset
approach is the correct architecture for any wrapper.

## References

- Agent SDK types: `node_modules/@anthropic-ai/claude-agent-sdk/entrypoints/sdk/`
  - `coreTypes.d.ts` — message types, hooks, permission modes
  - `runtimeTypes.d.ts` — Options, Query interface
  - `controlTypes.d.ts` — internal control protocol
- Prior research: `docs/research/cc-vs-sdk-20260316.md`
- Charter: F13 (UX Parity and Enhancement)
