use clap::{Parser as ClapParser, Subcommand};
use migration_guide::generate_migration_guide;
use release_notes::generate_release_note;
use release_notes_website::generate_release_notes_website;
use std::path::PathBuf;

mod github_client;
mod helpers;
mod migration_guide;
mod release_notes;
mod release_notes_website;

/// Generates markdown files used for a bevy releases.
///
/// Migration Guide:
/// * Gets all PRs with the `C-Breaking-Change` label and that were merged by bors.
/// * For each PR:
///     * Generate the title with a link to the relevant PR and
///     * Generate the migration guide section. This parses the markdown and generates valid makrdown that should pass markdownlint rules.
///
/// Release notes:
/// * Gets all PRs merged by bors
/// * Collect each author of closed PRs (Should this just list all contributors?)
/// * Sort each PR per area label
/// * Generate the list of merge PR
///
/// Requires a valid GITHUB_TOKEN environment variable, you can use a .env file or use your prefered method of passing env arguments.
///
/// Example used to generate for 0.9:
/// cargo run -- migration-guide --from v0.9.0 --to main --title "0.9 to 0.10" --weight 6
/// cargo run -- release-note --from v0.9.0 --to main
/// cargo run -- release-note-website --from bd4f611f7576c55739b466c6f0039e8421dab57e --to HEAD
#[derive(ClapParser)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    MigrationGuide {
        /// Date of the release of the previous version. Format: YYYY-MM-DD
        #[arg(short, long)]
        date: String,

        /// Title of the frontmatter
        #[arg(short, long)]
        title: String,

        /// Weight used for sorting
        #[arg(short, long)]
        weight: i32,

        /// Path used to output the generated file. Defaults to ./migration-guide.md
        #[arg(short, long)]
        path: Option<std::path::PathBuf>,
    },
    ReleaseNote {
        /// The name of the branch / tag to start from
        #[arg(short, long)]
        from: String,

        /// The name of the branch / tag to end on
        #[arg(short, long)]
        to: String,

        /// Path used to output the generated file. Defaults to ./release-notes.md
        #[arg(short, long)]
        path: Option<std::path::PathBuf>,
    },
    ReleaseNoteWebsite {
        /// The name of the branch / tag to start from
        #[arg(short, long)]
        from: String,

        /// The name of the branch / tag to end on
        #[arg(short, long)]
        to: String,

        /// Path used to output the generated file. Defaults to ./release-notes-website.md
        #[arg(short, long)]
        path: Option<std::path::PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let args = Args::parse();
    let repo = if let Commands::ReleaseNoteWebsite { .. } = args.command {
        "bevy-website"
    } else {
        "bevy"
    };
    let mut client = github_client::GithubClient::new(
        std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not found"),
        repo.to_string(),
    );

    match args.command {
        Commands::MigrationGuide {
            date,
            title,
            weight,
            path,
        } => generate_migration_guide(
            &title,
            weight,
            &date,
            path.unwrap_or_else(|| PathBuf::from("./migration-guide.md")),
            &mut client,
        )?,
        Commands::ReleaseNote { from, to, path } => generate_release_note(
            &from,
            &to,
            path.unwrap_or_else(|| PathBuf::from("./release-notes.md")),
            &mut client,
        )?,
        Commands::ReleaseNoteWebsite { from, to, path } => generate_release_notes_website(
            &from,
            &to,
            path.unwrap_or_else(|| PathBuf::from("./release-notes-website.md")),
            &mut client,
        )?,
    };

    Ok(())
}
