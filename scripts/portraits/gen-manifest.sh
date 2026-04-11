#!/usr/bin/env bash
# Generate manifest.json from packed theme archives and theme YAML files.
#
# The manifest has two sections:
#   themes:   pack metadata (sha256, size, persona count)
#   personas: role → filename-stem mapping (exact format portrait.rs expects)
#
# Requires: yq (https://github.com/mikefarah/yq)
#
# Usage:
#   ./scripts/portraits/gen-manifest.sh <dist-dir> <themes-yaml-dir> [base-url]
#   ./scripts/portraits/gen-manifest.sh dist/portraits ~/work/penny-orc/pennyfarthing/pennyfarthing-dist/personas/themes

set -euo pipefail

DIST_DIR="${1:?Usage: gen-manifest.sh <dist-dir> <themes-yaml-dir> [base-url]}"
THEMES_DIR="${2:?Usage: gen-manifest.sh <dist-dir> <themes-yaml-dir> [base-url]}"
BASE_URL="${3:-https://portraits.darkatelier.org/v1}"

if ! command -v yq &>/dev/null; then
    echo "Error: yq not found. Install with: brew install yq" >&2
    exit 1
fi

if [[ ! -d "$DIST_DIR" ]]; then
    echo "Error: dist directory not found: $DIST_DIR" >&2
    exit 1
fi

if [[ ! -d "$THEMES_DIR" ]]; then
    echo "Error: themes directory not found: $THEMES_DIR" >&2
    exit 1
fi

# Collect theme slugs from packed archives
themes=()
for pack in "$DIST_DIR"/*.tar.gz; do
    [[ ! -f "$pack" ]] && continue
    themes+=("$(basename "$pack" .tar.gz)")
done

if [[ ${#themes[@]} -eq 0 ]]; then
    echo "Error: no .tar.gz files found in $DIST_DIR" >&2
    exit 1
fi

# --- Build JSON ---

# Header
printf '{\n'
printf '  "schema": 1,\n'
printf '  "updated": "%s",\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '  "base_url": "%s",\n' "$BASE_URL"

# Themes section
printf '  "themes": {\n'
first=true
for theme in "${themes[@]}"; do
    pack="$DIST_DIR/${theme}.tar.gz"
    sha=$(cat "$DIST_DIR/${theme}.sha256")
    # stat -f%z (macOS) or stat -c%s (Linux)
    bytes=$(stat -f%z "$pack" 2>/dev/null || stat -c%s "$pack" 2>/dev/null)
    persona_count=$(tar tzf "$pack" | grep "original/.*\.png$" | wc -l | tr -d ' ')

    $first || printf ',\n'
    first=false
    printf '    "%s": {"pack_sha256": "%s", "pack_bytes": %s, "persona_count": %s}' \
        "$theme" "$sha" "$bytes" "$persona_count"
done
printf '\n  },\n'

# Personas section — parse theme YAMLs for role → filename-stem mapping.
# Uses a single yq call per theme to extract all role→stem mappings at once,
# avoiding O(roles × fields) subprocess overhead.
printf '  "personas": {\n'
first_theme=true
for theme in "${themes[@]}"; do
    yaml="$THEMES_DIR/${theme}.yaml"
    if [[ ! -f "$yaml" ]]; then
        echo "Warning: no theme YAML for $theme, skipping persona map" >&2
        continue
    fi

    # Extract all role→stem pairs. One yq call gets role, shortName, and OCEAN;
    # bash handles the slug transformation (yq/Go lacks jq's ascii_downcase/sub).
    persona_lines=""
    while IFS=$'\t' read -r role short ocean_o ocean_c ocean_e ocean_a ocean_n; do
        [[ -z "$role" || -z "$ocean_o" ]] && continue
        slug=$(echo "$short" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g; s/^-*//; s/-*$//')
        persona_lines+="${role}"$'\t'"${slug}-${ocean_o}${ocean_c}${ocean_e}${ocean_a}${ocean_n}"$'\n'
    done < <(yq -r '
      .agents | to_entries[] |
      select(.value.ocean.O != null) |
      [.key, (.value.shortName // (.value.character | split(" ") | .[0])),
       .value.ocean.O, .value.ocean.C, .value.ocean.E, .value.ocean.A, .value.ocean.N] |
      @tsv
    ' "$yaml" 2>/dev/null)

    [[ -z "$persona_lines" ]] && continue

    $first_theme || printf ',\n'
    first_theme=false
    printf '    "%s": {' "$theme"

    first_role=true
    while IFS=$'\t' read -r role stem; do
        [[ -z "$role" ]] && continue
        $first_role || printf ', '
        first_role=false
        printf '"%s": "%s"' "$role" "$stem"
    done <<< "$persona_lines"

    printf '}'
done
printf '\n  }\n'
printf '}\n'
