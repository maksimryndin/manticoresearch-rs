use clap::{Parser, Subcommand};
use manticoresearch_rs::migrator::Migrator;
use tracing::Level;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Manticoresearch migration cli.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Connection url
    #[arg(short, long, default_value_t={"http://localhost:9308".to_string()})]
    connection: String,

    /// Directory with migrations
    #[arg(short, long, default_value_t={"migrations".to_string()})]
    directory: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Migrate forward applying up migrations
    Up {
        /// Don't apply - only print sql statements
        #[arg(long, default_value_t = false)]
        fake: bool,
    },
    /// Rollback to a specific down migration
    Down {
        /// Don't apply - only print sql statements
        #[arg(long, default_value_t = false)]
        fake: bool,
        /// Migration name (`yyyymmddhhmmss`)
        migration: String,
    },
    /// List migrations with their status
    List,
    /// Log of applied migrations
    History,
    /// Add a migration template
    Add,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();
    let connection = cli.connection.to_string();
    let directory = &cli.directory;
    tracing_subscriber::registry()
        .with(LevelFilter::from_level(Level::INFO))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let migrator = Migrator::new(connection, directory);

    match &cli.command {
        Commands::Up { fake } => {
            let applied = migrator.up(*fake).await.unwrap();
            if applied.is_empty() {
                println!("All migrations are already applied");
            } else {
                let names: Vec<String> =
                    applied.into_iter().map(|m| m.name().to_string()).collect();
                println!("Applied:\n{}", names.join("\n"));
            }
        }
        Commands::Down { fake, migration } => {
            let applied = migrator.down(*fake, migration).await.unwrap();
            if applied.is_empty() {
                println!(
                    "All migrations back to {migration} (including it) are already rolled back"
                );
            } else {
                let names: Vec<String> =
                    applied.into_iter().map(|m| m.name().to_string()).collect();
                println!("Rolled back:\n{}", names.join("\n"));
            }
        }
        Commands::List => {
            let migrations = migrator.list().await.unwrap();
            println!("\n{: <14} | {: ^20}", "name", "applied");
            println!("-------------------------------------",);
            for migration in migrations {
                println!(
                    "{: <14} | {: ^20}",
                    migration.name(),
                    migration
                        .applied()
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "x".to_string())
                );
            }
            println!("\n");
        }
        Commands::History => {
            let migrations = migrator.history().await.unwrap();
            println!(
                "\n{: <14} | {: <4} | {: ^20} | {: ^20}",
                "name", "typ", "applied", "checksum"
            );
            println!("--------------------------------------------------------------------",);
            for migration in migrations {
                println!(
                    "{: <14} | {: <4} | {: ^20} | {: ^20}",
                    migration.name(),
                    format!("{}", migration.typ()),
                    migration
                        .applied()
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "x".to_string()),
                    migration.checksum()
                );
            }
            println!("\n");
        }
        Commands::Add => {
            let dir_name = migrator.add().await.unwrap();
            println!(
                "added migration template at {}/{dir_name}",
                migrator.directory()
            );
        }
    }
}
