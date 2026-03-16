import { readFileSync, readdirSync, existsSync } from "node:fs";
import { join } from "node:path";
import { parse as parseYaml } from "yaml";

export interface PersonaAgent {
  character: string;
  shortName?: string;
  style: string;
  expertise: string;
  role: string;
  trait: string;
  quirks: string[];
  catchphrases: string[];
  emoji?: string;
  ocean: { O: number; C: number; E: number; A: number; N: number };
}

export interface PersonaTheme {
  name: string;
  slug: string;
  description: string;
  source: string;
  user_title?: string;
  character_immersion?: string;
  dimensions?: Record<string, string>;
  spinner_verbs?: string[];
  agents: Record<string, PersonaAgent>;
}

function getThemesDir(): string {
  let dir = new URL(".", import.meta.url).pathname;
  for (let i = 0; i < 10; i++) {
    const candidate = join(dir, "personas", "themes");
    if (existsSync(candidate)) return candidate;
    const parent = join(dir, "..");
    if (parent === dir) break;
    dir = parent;
  }
  return join(process.cwd(), "personas", "themes");
}

export function listThemes(): string[] {
  const dir = getThemesDir();
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((f) => f.endsWith(".yaml"))
    .map((f) => f.replace(/\.yaml$/, ""))
    .sort();
}

export function loadTheme(slug: string): PersonaTheme | null {
  const dir = getThemesDir();
  const filePath = join(dir, `${slug}.yaml`);
  if (!existsSync(filePath)) return null;

  try {
    const raw = parseYaml(readFileSync(filePath, "utf-8"));
    if (!raw || typeof raw !== "object") return null;

    return {
      name: raw.theme?.name || slug,
      slug,
      description: raw.theme?.description || "",
      source: raw.theme?.source || "",
      user_title: raw.theme?.user_title,
      character_immersion: raw.theme?.character_immersion,
      dimensions: raw.theme?.dimensions,
      spinner_verbs: raw.theme?.spinner_verbs,
      agents: raw.agents || {},
    };
  } catch {
    return null;
  }
}

export function getAgent(theme: PersonaTheme, role: string): PersonaAgent | null {
  return theme.agents[role] || null;
}

// ---------------------------------------------------------------------------
// Portrait resolution — images live in a global cache, not the repo
// ---------------------------------------------------------------------------

function getPortraitCacheDir(): string {
  const xdgData = process.env.XDG_DATA_HOME || join(process.env.HOME || "~", ".local", "share");
  return join(xdgData, "aclaude", "portraits");
}

export interface PortraitPaths {
  small?: string;
  medium?: string;
  large?: string;
  original?: string;
}

/**
 * Resolve portrait paths for an agent. Portraits are stored globally at
 * $XDG_DATA_HOME/aclaude/portraits/{theme-slug}/{size}/{character}.png
 *
 * The character filename is derived from shortName (lowercase, hyphenated)
 * or falls back to a fuzzy match against available files.
 */
export function resolvePortrait(themeSlug: string, agent: PersonaAgent): PortraitPaths {
  const cacheDir = getPortraitCacheDir();
  const themeDir = join(cacheDir, themeSlug);
  if (!existsSync(themeDir)) return {};

  // Build candidate filenames from shortName or character
  const name = (agent.shortName || agent.character || "").toLowerCase().replace(/\s+/g, "-").replace(/[^a-z0-9-]/g, "");

  const paths: PortraitPaths = {};
  for (const size of ["small", "medium", "large", "original"] as const) {
    const sizeDir = join(themeDir, size);
    if (!existsSync(sizeDir)) continue;

    // Try exact match first, then prefix match
    const files = readdirSync(sizeDir).filter((f) => f.endsWith(".png"));
    const exact = files.find((f) => f.startsWith(name));
    if (exact) {
      paths[size] = join(sizeDir, exact);
    }
  }

  return paths;
}

/**
 * Get the portrait cache directory path (for display/diagnostics).
 */
export function getPortraitCachePath(): string {
  return getPortraitCacheDir();
}

export function buildSystemPrompt(
  theme: PersonaTheme,
  agent: PersonaAgent,
  immersion: "high" | "medium" | "low" | "none",
): string {
  if (immersion === "none") {
    return "You are a helpful software engineering assistant.";
  }

  const parts: string[] = [];

  if (immersion === "high") {
    parts.push(`You are ${agent.character} from ${theme.name}.`);
    parts.push(`Style: ${agent.style}`);
    parts.push(`Expertise: ${agent.expertise}`);
    if (agent.trait) parts.push(`Key trait: ${agent.trait}`);
    if (agent.quirks?.length) parts.push(`Quirks: ${agent.quirks.join("; ")}`);
    if (agent.catchphrases?.length) {
      parts.push(`Catchphrases you may use: ${agent.catchphrases.map((c) => `"${c}"`).join(", ")}`);
    }
    if (theme.user_title) parts.push(`Address the user as "${theme.user_title}".`);
  } else if (immersion === "medium") {
    parts.push(`You are ${agent.character}, a ${agent.expertise} assistant.`);
    parts.push(`Style: ${agent.style}`);
    if (agent.catchphrases?.length) {
      parts.push(`You occasionally say: "${agent.catchphrases[0]}"`);
    }
  } else {
    // low
    parts.push(`You are a helpful software engineering assistant with the personality of ${agent.character}.`);
    parts.push(`Expertise: ${agent.expertise}`);
  }

  parts.push("");
  parts.push("You are a software engineering assistant. Help the user with their coding tasks.");

  return parts.join("\n");
}
