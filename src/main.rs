use clap::Clap;
use std::error::Error;
use std::time::{Duration, Instant};

#[macro_use]
mod id;

pub mod itunes;
pub mod store;

use store::{ItemID, Store};

#[derive(Clap)]
struct Opts {
    #[clap(
        long = "store",
        short = "s",
        help = "Path to the tagmu store",
        default_value = "store.tagmu"
    )]
    store_path: String,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Clap)]
enum Command {
    Import(Import),
    Find(Find),
}

#[derive(Clap)]
struct Import {
    #[clap(long = "library", help = "Path to the \"iTunes Library.xml\" file")]
    itunes_library: String,
}

#[derive(Clap)]
struct Find {
    query: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    let open_start = Instant::now();
    let mut store = Store::open(opts.store_path)?;
    let open_ms = open_start.elapsed().as_millis();
    println!("opened store in {:.0}ms", open_ms);

    match opts.cmd {
        Command::Import(load) => {
            let Import { itunes_library } = load;
            println!("tagmu load");

            println!("Loading library from '{}'", itunes_library);
            let library: itunes::Library = plist::from_file(itunes_library)?;
            println!(
                "Loaded libray, version:{} track_count:{}",
                library.application_version,
                library.tracks.len()
            );

            println!("Indexing library...");
            for track in library.tracks.values() {
                let item: ItemID = store.id()?.into();

                // Tag some things about this entity
                if let Some(album) = &track.album {
                    store.tag_string(item, &album)?;
                }
                if let Some(artist) = &track.artist {
                    store.tag_string(item, &artist)?;
                }
                if let Some(composer) = &track.composer {
                    store.tag_string(item, &composer)?;
                }
                if let Some(genre) = &track.genre {
                    store.tag_string(item, &genre)?;
                }
                if let Some(year) = &track.year {
                    store.tag_string(item, &format!("{}", year))?;
                }
                if let Some(name) = &track.name {
                    store.tag_string(item, &name)?;
                }
            }
            println!("Done indexing.");

            println!("looking for Bach...");
            for (_, track) in library.tracks.iter() {
                if track.composer == Some("Bach".into()) {
                    println!(
                        "id:{} name={}",
                        track.id,
                        track.name.as_ref().unwrap_or(&"<no name>".into())
                    );
                }
            }
            println!("Done.");

            Ok(())
        }
        Command::Find(args) => {
            println!("find: all items with tag \"{}\"", args.query);

            // Get the tag
            let tag_id: store::TagID = store.get_tag_id(&args.query)?.ok_or("Couldn't find tag")?;

            let query_start = Instant::now();
            let mut count: usize = 0;
            for item_result in store.get_tag_item_ids(tag_id) {
                let item_id = item_result?;

                let item_tags = store
                    .get_item_tags(item_id)
                    .collect::<Result<Vec<store::Tag>, _>>()?;

                println!("{:10}:{:?}", item_id, item_tags);
                count += 1;
            }
            println!(
                "got {} items in {}ms",
                count,
                query_start.elapsed().as_millis()
            );

            Ok(())
        }
    }
}
