#![forbid(unsafe_code)]

use aclaude::config;
use aclaude::persona;
use aclaude::session;
use aclaude::updater;
use clap::{Parser, Subcommand};

/// Build-time version info injected by build.rs.
const VERSION: &str = env!("ACLAUDE_VERSION");
const COMMIT: &str = env!("ACLAUDE_COMMIT");
const BUILD_TIME: &str = env!("ACLAUDE_BUILD_TIME");
const CHANNEL: &str = env!("ACLAUDE_CHANNEL");

#[derive(Parser)]
#[command(
    name = "aclaude",
    version = VERSION,
    about = "Opinionated Claude Code distribution with persona theming"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Model override
    #[arg(short = 'm', long)]
    model: Option<String>,

    /// Theme override
    #[arg(short = 't', long)]
    theme: Option<String>,

    /// Role override
    #[arg(short = 'r', long)]
    role: Option<String>,

    /// Immersion level override
    #[arg(short = 'i', long)]
    immersion: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show resolved configuration
    Config,

    /// Persona management
    Persona {
        #[command(subcommand)]
        action: PersonaAction,
    },

    /// Check for and install updates
    Update,

    /// Show version details
    Version,

    /// List installed versions
    Versions {
        /// Clean old versions, keeping N most recent
        #[arg(long)]
        clean: Option<usize>,
    },
}

#[derive(Subcommand)]
enum PersonaAction {
    /// List available themes
    List,

    /// Show theme details
    Show {
        /// Theme slug
        name: String,

        /// Show specific agent role
        #[arg(long, default_value = "dev")]
        agent: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Build CLI overrides table
    let mut overrides = toml::Table::new();
    if cli.model.is_some() || cli.theme.is_some() || cli.role.is_some() || cli.immersion.is_some() {
        if let Some(model) = &cli.model {
            let mut session = toml::Table::new();
            session.insert("model".to_string(), toml::Value::String(model.clone()));
            overrides.insert("session".to_string(), toml::Value::Table(session));
        }
        let mut persona_overrides = toml::Table::new();
        if let Some(theme) = &cli.theme {
            persona_overrides.insert("theme".to_string(), toml::Value::String(theme.clone()));
        }
        if let Some(role) = &cli.role {
            persona_overrides.insert("role".to_string(), toml::Value::String(role.clone()));
        }
        if let Some(immersion) = &cli.immersion {
            persona_overrides.insert(
                "immersion".to_string(),
                toml::Value::String(immersion.clone()),
            );
        }
        if !persona_overrides.is_empty() {
            overrides.insert("persona".to_string(), toml::Value::Table(persona_overrides));
        }
    }

    let cli_overrides = if overrides.is_empty() {
        None
    } else {
        Some(&overrides)
    };

    match cli.command {
        None => {
            // Default: start interactive session
            let cfg = config::load_config(cli_overrides)?;
            session::start_session(&cfg)?;
        }

        Some(Commands::Config) => {
            let cfg = config::load_config(cli_overrides)?;
            let paths = config::config_paths();
            println!("Config paths:");
            println!("  defaults: {}", paths.defaults.display());
            println!("  global:   {}", paths.global.display());
            println!("  local:    {}", paths.local.display());
            println!();
            println!("{}", toml::to_string_pretty(&cfg)?);
        }

        Some(Commands::Persona { action }) => match action {
            PersonaAction::List => {
                let themes = persona::list_themes();
                println!("{} themes available:", themes.len());
                for slug in &themes {
                    if let Ok(theme) = persona::load_theme(slug) {
                        println!("  {:<30} {}", slug, theme.theme.description);
                    } else {
                        println!("  {slug}");
                    }
                }
            }
            PersonaAction::Show { name, agent } => {
                let theme = persona::load_theme(&name)?;
                println!("Theme: {} ({})", theme.theme.name, theme.category);
                println!("Description: {}", theme.theme.description);
                if let Some(title) = &theme.theme.user_title {
                    println!("User title: {title}");
                }
                println!("Roles: {}", {
                    let mut roles: Vec<_> = theme.agents.keys().collect();
                    roles.sort();
                    roles
                        .iter()
                        .map(|r| r.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                });
                println!();

                if let Ok(a) = persona::get_agent(&theme, &agent) {
                    println!("Agent: {} (role: {agent})", a.character);
                    println!("  Style: {}", a.style);
                    println!("  Expertise: {}", a.expertise);
                    println!("  Trait: {}", a.r#trait);
                    if !a.quirks.is_empty() {
                        println!("  Quirks: {}", a.quirks.join("; "));
                    }
                    if !a.catchphrases.is_empty() {
                        println!("  Catchphrases:");
                        for phrase in &a.catchphrases {
                            println!("    - \"{phrase}\"");
                        }
                    }
                } else {
                    eprintln!("Role '{agent}' not found in theme '{name}'");
                    eprintln!(
                        "Available: {}",
                        theme
                            .agents
                            .keys()
                            .map(String::as_str)
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
        },

        Some(Commands::Update) => {
            let channel = updater::Channel::parse(CHANNEL);
            println!(
                "Checking for updates ({})...",
                if channel == updater::Channel::Stable {
                    "stable"
                } else {
                    "alpha"
                }
            );
            match updater::check_for_update(channel)? {
                Some(tag) => println!("Latest: {tag} (current: {VERSION}-{COMMIT})"),
                None => println!("No updates available."),
            }
        }

        Some(Commands::Version) => {
            println!("aclaude {VERSION}");
            println!("  commit:  {COMMIT}");
            println!("  built:   {BUILD_TIME}");
            println!("  channel: {CHANNEL}");
        }

        Some(Commands::Versions { clean }) => {
            if let Some(keep) = clean {
                let removed = updater::clean_old_versions(keep)?;
                println!("Removed {removed} old version(s).");
            }
            let versions = updater::list_versions()?;
            if versions.is_empty() {
                println!("No installed versions found.");
            } else {
                println!("Installed versions:");
                for v in &versions {
                    println!("  {v}");
                }
            }
        }
    }

    Ok(())
}
