# aclaude Design Questions

Open questions about what aclaude should be. These are not issues or TODOs —
they're uncertainties that probing is expected to resolve. Cross-referenced
with aae-orc charter frontier items.

Last updated: 2026-03-18

---

## What is aclaude for?

Two use cases, possibly two products:

**Human operator** — a person at a terminal using Claude Code with added
value: persona theming, status bars, context tracking, portrait images,
tmux integration, configuration management. The UX should be richer than
vanilla Claude Code. TUI matters. Startup time matters. Persona flair matters.

**Autonomous agent workload** — a process managed by marvel, running in a
tmux pane, executing tasks without human interaction. Doesn't need persona
flair. Doesn't need TUI. Needs fast startup, minimal overhead, programmatic
control, predictable behavior. May not need themes at all — or needs exactly
one, injected at launch.

These may not be the same binary. Or the same language. Or the same SDK.
The current implementation serves neither optimally: the SDK wrapper adds
startup lag (F14: ~15s first prompt) that hurts human UX, while the embedded
themes and persona system add binary size and complexity that autonomous
agents don't use.

Charter: F13 (UX parity), F14 (TUI), F16 (assumption provenance).

## Should aclaude be TypeScript?

TypeScript was inherited from pennyfarthing. It works: bun compile produces
signed binaries, themes embed, the Agent SDK is Node.js-native. But:

- 15-second first prompt latency (F14). Structural? Fixable? Intrinsic to
  subprocess startup?
- No TUI without additional frameworks (ink, blessed). Claude Code's TUI
  is not exposed through the SDK.
- bun compile creates a virtual FS boundary that broke 5+ features
  (F5-F7, F16, F19). Consistent fix pattern but persistent friction.
- 63MB binary for what is essentially a config wrapper + persona injector.

Alternatives not yet evaluated:
- **Go** — would match marvel/switchboard. ldflags for version injection.
  Native binary, fast startup. No Agent SDK (would need to spawn claude
  CLI directly). TUI libraries available (bubbletea, tcell).
- **Rust** — performance, small binary, plugin system possibilities.
  No Agent SDK. Higher development cost.
- **Python** — Agent SDK exists. Packaging is worse (pyinstaller, etc.).
  Performance likely worse.

Not proposing a rewrite. Proposing that this question be examined rather
than assumed settled. The distribution probe (F11) validated that bun
compile *works* — it did not validate that TypeScript is the right choice.

Charter: F8 (bootstrapping through probe code), F16 (assumption provenance).

## Should themes/personas be bundled or loaded?

Currently: 100 theme YAMLs embedded in the binary via embed-themes.ts
(~1.7MB). Portraits are already external (global cache). The agent
subprocess (Claude Code) cannot see embedded themes (F20).

Arguments for loading at runtime:
- Autonomous agents need one theme or none. Bundling 100 is waste.
- Pack system (marvel) should manage themes as content, not code.
- Decoupling enables other consoles (zclaude, dclaude) to use same themes.
- Smaller binary if themes are external.

Arguments for bundling:
- Zero-dependency: works without any additional install steps.
- No network fetch at startup. Offline-friendly.
- Version coherence: binary + themes are always in sync.

Possible hybrid: bundle a minimal default set, load additional themes
from `~/.local/share/aclaude/themes/` or via marvel pack resolution.

Charter: F12 (persona themes as content pack), F15 (persona model).

## How should aclaude sessions be bootstrapped?

Docker-like layering model:

1. **Entrypoint** (baked in) — "you are aclaude." System prompt segment
   via `preset: 'claude_code', append: personaPrompt`. Always present.
2. **Installed base** (on disk) — packs, themes, commands. Updateable
   independently of the binary.
3. **Session injection** (per-launch) — persona, model, permissions,
   feature flags. Ephemeral.

Open tension: aclaude inherits Claude Code's config (`~/.claude/`,
`.claude/rules/`, CLAUDE.md) via `settingSources`. Users may want full
inheritance, isolation, or switching between vanilla Claude Code and
aclaude. This needs to be a preference, not hardcoded.

For autonomous agents (marvel teams): bootstrap is different. No TUI
instructions. Task assignment, tool permissions, workspace boundaries.
Same binary, different entrypoint.

Charter: F18 (session bootstrap and context layering).

## Version identity and distribution coordination

Solved: gen-version.ts injects VERSION, CHANNEL, COMMIT, BUILD_TIME at
build time. Self-update validated end-to-end.

Open: brew and self-update are parallel version managers (F27). No
coordination. Design question: should aclaude detect brew install and
defer to `brew upgrade`? Or should one channel win?

Charter: F11 (distribution model).
