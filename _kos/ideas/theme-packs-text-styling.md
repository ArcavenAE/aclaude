# Idea: Theme Packs for TUI Text Styling

**Date:** 2026-04-12
**Origin:** User testing session — observed that user text lacks visual
distinction (no background color, no differentiation from assistant text)

## The Idea

Allow theme packs to control TUI text styling beyond persona portraits.
The TUI should support configurable colors for:

- User input text (Claude Code uses light-on-grey full-line background)
- Assistant response text
- Markdown rendering (headers, code blocks, links, bold/italic)
- Diff rendering (additions, deletions, context)
- Agent input/output (different agent types distinguished visually)
- Input box styling (border, placeholder, active state)
- Status bar colors

### Design Tension

The TUI should **honor the terminal's existing colors** as the default
(respect the user's terminal theme), but provide **overrides** so users
and theme packs can customize. This means:

1. Default: inherit terminal colors (no forced palette)
2. Theme pack override: pack provides a color scheme
3. User override: user config wins over pack and terminal

This maps to forestage's existing config merge chain:
defaults → global → local → env → CLI flags. Colors are just another
config layer.

### Relationship to Existing Concepts

- **Persona themes** (118 YAML files) currently define fictional universe
  + character. Text styling would be a new section in theme YAML.
- **F12 (persona themes as content pack)** — text styling would ship
  with the theme pack, not be a separate pack type.
- **F15 (persona model)** — theme provides the color scheme, persona
  inherits it, role doesn't affect colors.
- **F6 (content pack format)** — color scheme would be a new artifact
  type in pack.yaml.

### Not Yet

This is a post-MVP idea. The immediate fix is removing the "You:"
preamble and matching Claude Code's visual language for user text.
Full theme-pack text styling is a later concern.
