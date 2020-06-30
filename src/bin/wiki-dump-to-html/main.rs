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

    while let Some(s) = args.next() {
        if s == "--help" {
            return print_usage();
        }

        let path = PathBuf::from(s);
        println!("Processing: {}", path.display());
        
        let path = path.canonicalize()?;
        println!("    ({})", path.display());

        let file = File::open(path)?;

        process_file(file)?;
    }

    Ok(())
}

fn process_file(file: File) -> Res<()> {
    let file = std::io::BufReader::new(file);

    for result in parse_mediawiki_dump::parse(file) {
        match result {
            Err(error) => {
                return Err(error.to_string().into());
            }
            Ok(page) => if page.namespace == 0 && match &page.format {
                None => false,
                Some(format) => format == "text/x-wiki"
            } && match &page.model {
                None => false,
                Some(model) => model == "wikitext"
            } {
                println!(
                    "The page {title:?} is an ordinary article with byte length {length}.",
                    title = page.title,
                    length = page.text.len()
                );
            } else {
                println!("The page {:?} has something special to it.", page.title);
            }
        }
    }

    Ok(())
}

fn print_usage() -> Res<()> {
    println!("USAGE: wiki-dump-to-html file-name1 [file-name2 [...]]");
    Ok(())
}