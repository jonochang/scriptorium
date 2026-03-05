use anyhow::Context;
use bookstore_core::{Book, seed_church_bookstore};
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
        price_cents: u32,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut inventory = seed_church_bookstore();

    match cli.command {
        Command::List => {
            let out = serde_json::to_string_pretty(inventory.books())
                .context("failed to serialize books")?;
            println!("{out}");
        }
        Command::Add { id, title, author, category, price_cents } => {
            inventory.add_book(Book { id, title, author, category, price_cents })?;
            let out = serde_json::to_string_pretty(inventory.books())
                .context("failed to serialize books")?;
            println!("{out}");
        }
    }

    Ok(())
}
