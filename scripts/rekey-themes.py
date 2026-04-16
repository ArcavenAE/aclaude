#!/usr/bin/env python3
"""Rekey theme YAML files: agents (keyed by role) → characters (keyed by character slug).

The ONLY structural change: the HashMap key changes from role-name to
character-slug. All character fields are PRESERVED — ocean, visual, helper,
trait, style, expertise, quirks, catchphrases, emoji, shortName.

The old role field is renamed to backstory_role (what slot they filled in
pennyfarthing/forestage v1). The role-name key becomes backstory_role value.

Usage: python3 scripts/rekey-themes.py personas/themes/
"""

import sys
import re
from pathlib import Path
import yaml


def slugify(name: str) -> str:
    """Convert character name to a slug key."""
    slug = name.lower()
    slug = re.sub(r'[()]', ' ', slug)
    slug = re.sub(r'[^a-z0-9\s-]', '', slug)
    slug = re.sub(r'\s+', '-', slug.strip())
    slug = re.sub(r'-+', '-', slug)
    return slug


def transform_theme(content: str) -> str:
    """Transform a theme YAML: rename agents→characters, rekey by character slug.

    Preserves ALL fields. Only changes: key name and adds backstory_role.
    """
    data = yaml.safe_load(content)
    if not data or 'agents' not in data:
        return content

    agents = data['agents']

    # Build the new characters section preserving all fields
    lines = []
    lines.append("characters:")

    seen_slugs = {}

    for role_name, agent in agents.items():
        char_name = agent.get('character', role_name)
        slug = slugify(char_name)

        # Handle duplicate characters (same person in different old roles)
        if slug in seen_slugs:
            slug = f"{slug}-{role_name}"
        seen_slugs[slug] = True

        lines.append(f"  {slug}:")
        lines.append(f'    character: "{char_name}"')

        if agent.get('shortName'):
            lines.append(f'    shortName: "{agent["shortName"]}"')

        # Visual — preserved for portraits
        if agent.get('visual'):
            lines.append(f'    visual: "{_esc(agent["visual"])}"')

        # OCEAN — preserved for analysis
        if agent.get('ocean'):
            o = agent['ocean']
            lines.append("    ocean:")
            for key in ['O', 'C', 'E', 'A', 'N']:
                if key in o:
                    lines.append(f"      {key}: {o[key]}")

        # Style
        if agent.get('style'):
            lines.append(f"    style: {_yaml_str(agent['style'])}")

        # Expertise
        if agent.get('expertise'):
            lines.append(f"    expertise: {_yaml_str(agent['expertise'])}")

        # Trait — preserved for discrimination
        if agent.get('trait'):
            lines.append(f"    trait: {_yaml_str(agent['trait'])}")

        # Old role description → backstory_role_description (the prose, not the key)
        if agent.get('role'):
            lines.append(f"    backstory_role_description: {_yaml_str(agent['role'])}")

        # The key they were filed under → backstory_role
        lines.append(f"    backstory_role: {role_name}")

        # Quirks
        if agent.get('quirks'):
            lines.append("    quirks:")
            for q in agent['quirks']:
                lines.append(f"      - {_yaml_str(q)}")

        # Catchphrases
        if agent.get('catchphrases'):
            lines.append("    catchphrases:")
            for c in agent['catchphrases']:
                lines.append(f"      - {_yaml_str(c)}")

        # Emoji
        if agent.get('emoji'):
            lines.append(f'    emoji: "{agent["emoji"]}"')

        # Helper — preserved for flavor
        if agent.get('helper'):
            h = agent['helper']
            lines.append("    helper:")
            if h.get('name'):
                lines.append(f'      name: "{_esc(h["name"])}"')
            if h.get('style'):
                lines.append(f'      style: "{_esc(h["style"])}"')

        lines.append("")

    new_section = "\n".join(lines)

    # Replace agents: section with characters: section
    agent_pattern = re.compile(r'^agents:\s*$', re.MULTILINE)
    match = agent_pattern.search(content)
    if not match:
        return content

    before = content[:match.start()]
    return before + new_section


def _esc(s: str) -> str:
    """Escape a string for double-quoted YAML."""
    return s.replace('\\', '\\\\').replace('"', '\\"')


def _yaml_str(s: str) -> str:
    """Format a string for YAML, quoting if needed."""
    if not s:
        return '""'
    needs_quote = any(c in s for c in ':#{}[]|>&*!?,\'"')
    if needs_quote:
        return f'"{_esc(s)}"'
    return s


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 scripts/rekey-themes.py personas/themes/")
        sys.exit(1)

    themes_dir = Path(sys.argv[1])
    if not themes_dir.is_dir():
        print(f"Not a directory: {themes_dir}")
        sys.exit(1)

    yaml_files = sorted(themes_dir.glob("*.yaml"))
    print(f"Found {len(yaml_files)} theme files")

    for f in yaml_files:
        content = f.read_text()
        try:
            new_content = transform_theme(content)
            f.write_text(new_content)
            print(f"  ✓ {f.name}")
        except Exception as e:
            print(f"  ✗ {f.name}: {e}")

    print(f"\nDone. {len(yaml_files)} files transformed.")


if __name__ == "__main__":
    main()


if __name__ == "__main__":
    main()
