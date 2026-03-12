use anyhow::Context;
use bookstore_app::CatalogService;
use bookstore_domain::Book;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "bookstore")]
#[command(about = "Church bookstore CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    List,
    Add {
        #[arg(long)]
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        author: String,
        #[arg(long)]
        category: String,
        #[arg(long)]
        price_cents: i64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let catalog = CatalogService::with_seed();

    match cli.command {
        Command::List => {
            let books = catalog.list_books().await;
            let out = serde_json::to_string_pretty(&books).context("failed to serialize books")?;
            println!("{out}");
        }
        Command::Add { id, title, author, category, price_cents } => {
            catalog.add_book(Book { id, title, author, category, price_cents }).await?;
            let books = catalog.list_books().await;
            let out = serde_json::to_string_pretty(&books).context("failed to serialize books")?;
            println!("{out}");
        }
    }

    Ok(())
}
