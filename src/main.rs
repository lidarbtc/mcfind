use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::PathBuf;

mod find;
use find::find_dropped_items_in_entity_file;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    path: PathBuf,

    #[arg(
        short,
        long,
        help = "Filter by specific items (comma-separated). ex. diamond_sword,minecraft:shulker_shell",
        value_delimiter = ','
    )]
    items: Option<Vec<String>>,
}

fn main() {
    let cli = Cli::parse();
    let entities_path = cli.path;

    let filter_items: Option<Vec<String>> = cli.items.map(|items| {
        items
            .into_iter()
            .map(|item| {
                if !item.contains("minecraft:") {
                    format!("minecraft:{}", item)
                } else {
                    item
                }
            })
            .collect()
    });

    println!(
        "Current working directory: {:?}",
        std::env::current_dir().unwrap()
    );
    println!(
        "Search path: {:?}",
        entities_path.canonicalize().unwrap_or_default()
    );
    if let Some(ref items) = filter_items {
        println!("Filtering for items: {:?}", items);
    }
    println!("Starting search for dropped items...");

    let entries: Vec<_> = std::fs::read_dir(entities_path)
        .unwrap_or_else(|e| panic!("Cannot read directory: {}", e))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let pb = ProgressBar::new(entries.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
        .unwrap()
        .progress_chars("#>-"));

    let all_items: Vec<_> = entries
        .par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("mca") {
                pb.inc(1);
                let items = find_dropped_items_in_entity_file(&path);

                if let Some(ref filter_items) = filter_items {
                    Some(
                        items
                            .into_iter()
                            .filter(|item| {
                                filter_items.iter().any(|filter| &item.item_type == filter)
                            })
                            .collect::<Vec<_>>(),
                    )
                } else {
                    Some(items)
                }
            } else {
                None
            }
        })
        .flatten()
        .collect();

    pb.finish_with_message("Processing complete");

    if all_items.is_empty() {
        println!("No dropped items found.");
    } else {
        println!("\nFound dropped items:");
        for item in all_items {
            println!(
                "Entity file: {}, Item: {}, Count: {}, Position: ({:.1}, {:.1}, {:.1})",
                item.entity_file,
                item.item_type,
                item.count,
                item.position.0,
                item.position.1,
                item.position.2
            );
        }
    }
}
