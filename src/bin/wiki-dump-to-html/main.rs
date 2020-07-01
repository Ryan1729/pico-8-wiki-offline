use std::{
    path::PathBuf,
    fs::File,
};

type Res<A> = Result<A, Box<dyn std::error::Error>>; 

fn main() -> Res<()> {
    let mut args = std::env::args();
    if args.len() < 2 {
        return print_usage();
    }

    args.next(); // exe name

    let mut verbose = false;

    while let Some(s) = args.next() {
        if s == "--help" {
            return print_usage();
        }

        if s == "--verbose" {
            verbose = true;
            continue;
        }

        let path = PathBuf::from(s);
        println!("Processing: {}", path.display());
        
        let path = path.canonicalize()?;
        println!("    ({})", path.display());

        let file = File::open(path)?;

        process_file(file, verbose)?;
    }

    Ok(())
}

fn process_file(file: File, verbose: bool) -> Res<()> {
    let file = std::io::BufReader::new(file);

    for result in parse_mediawiki_dump::parse(file) {
        let page = result.map_err(|e| e.to_string())?;

        type Namespace = u32;
        const TALK: Namespace = 1;
        const USER: Namespace = 2;
        const USER_TALK: Namespace = 3;
        const FILE: Namespace = 6;
        const MEDIA_WIKI: Namespace = 8;

        const TEMPLATE: Namespace = 10;
        const CATEGORY: Namespace = 14;
        const CATEGORY_TALK: Namespace = 15;

        //const USER_BLOG: Namespace = 500;
        const USER_BLOG_COMMENT: Namespace = 501;
        const BLOG: Namespace = 502;

        const MESSAGE_WALL: Namespace = 1200;
        const THREAD: Namespace = 1201;
        const MESSAGE_WALL_GREETING: Namespace = 1202;

        const BOARD: Namespace = 2000;

        match page.namespace {
            TALK
            | USER
            | USER_TALK
            | MESSAGE_WALL
            | THREAD
            | MESSAGE_WALL_GREETING
            | USER_BLOG_COMMENT
            | TEMPLATE
            | CATEGORY
            | CATEGORY_TALK
            | BOARD
            | MEDIA_WIKI
            | BLOG => {
                if verbose {
                    println!("Seems like the page {:?} is meta-content, not true content.", page.title);
                    println!("{:#?}", page);
                }
            }
            FILE => {
                if verbose {
                    println!("The page {:?} seems to be a file which we are skipping for now.", page.title);
                    println!("{:#?}", page);
                }
            }
            _ => {
                println!(
                    "The page {title:?} seems to be a content article with byte length {length} and namespace {namespace}.",
                    title = page.title,
                    length = page.text.len(),
                    namespace = page.namespace
                );
            }
        }
    }

    Ok(())
}

fn print_usage() -> Res<()> {
    println!("USAGE: wiki-dump-to-html [--verbose] FILENAME1 [FILENAME2 [...]]");
    Ok(())
}