use std::{
    env::current_dir,
    io::{BufWriter, Write},
    fs::{
        create_dir_all,
        File,
        OpenOptions,
    },
    path::PathBuf,
};

type Res<A> = Result<A, Box<dyn std::error::Error>>; 

const EXE_NAME: &str = "wiki-dump-to-html";

fn main() -> Res<()> {
    let mut args = std::env::args();
    if args.len() < 2 {
        return print_usage();
    }

    args.next(); // exe name

    let mut verbose = false;
    let mut output_dir_spec = None;

    let mut files = Vec::new();

    while let Some(s) = args.next() {
        if s == "--help" {
            return print_usage();
        }

        if s == "--verbose" {
            verbose = true;
            continue;
        }

        if s == "--output-dir" {
            output_dir_spec = args.next();
            if output_dir_spec.is_none() {
                println!("Missing output dir!");
                return print_usage();
            }
            continue;
        }

        let path = PathBuf::from(s);

        println!("found input file: {}", path.display());
        
        let path = path.canonicalize()?;
        println!("    ({})", path.display());

        let file = File::open(path)?;

        files.push(file);
    }

    let output_dir = if let Some(s) = output_dir_spec {
        confirm_out_dir(PathBuf::from(s))?
    } else {
        let mut default_dir = current_dir()?;
        default_dir.push(
            PathBuf::from(format!("{}-output", EXE_NAME))
        );

        confirm_out_dir(default_dir)?
    };

    println!("will output to {}", output_dir.display());

    let mut pages = Vec::new();

    for file in files {
        let new_pages = extract_pages(file, verbose)?;
        pages.extend(new_pages.into_iter());
    }

    let mut index_path = output_dir.join("index");
    index_path.set_extension("html");

    let index_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(index_path)?;

    let mut writer = BufWriter::new(&index_file);

    let header = r##"<!DOCTYPE html>
<html><head>
<meta http-equiv="content-type" content="text/html; charset=UTF-8"><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><style type="text/css">body{
margin:40px auto;
max-width:650px;
line-height:1.6;
font-size:18px;
color:#888;
background-color:#111;
padding:0 10px
}
h1,h2,h3{line-height:1.2}
a:link {color: #999;}
a:visited {color: #666;}
</style></head>
<body>"##;

    writeln!(&mut writer, "{}", header)?;

    for page in pages.iter() {        
        writeln!(&mut writer, "<h2>{}</h2>", page.title)?;

        writeln!(&mut writer, "<p>{}</p>", page.text)?;

        writeln!(&mut writer, "<hr/>")?;
    }
    
    writeln!(&mut writer, "</body></html>")?;

    Ok(())
}

fn confirm_out_dir(path: PathBuf) -> Res<PathBuf> {
    println!("probing output dir: {}", path.display());

    if path.exists() {
        if !path.is_dir() {
            return Err(
                format!(
                    "{} exists but is not a directory!",
                    path.display()
                ).into()
            );
        }
    } else {
        create_dir_all(&path)?;
    }

    let path = path.canonicalize()?;
    println!("    ({})", path.display());

    Ok(path)
}

type Page = parse_mediawiki_dump::Page;

fn extract_pages(file: File, verbose: bool) -> Res<Vec<Page>> {
    let file = std::io::BufReader::new(file);

    let mut pages = Vec::new();

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
                if verbose {
                    println!(
                        "The page {title:?} seems to be a content article with byte length {length} and namespace {namespace}.",
                        title = page.title,
                        length = page.text.len(),
                        namespace = page.namespace
                    );
                }

                pages.push(page);
            }
        }
    }

    Ok(pages)
}

fn print_usage() -> Res<()> {
    println!(
        "USAGE: {} [--verbose] [--output-dir DIRNAME] FILENAME1 [FILENAME2 [...]]",
        EXE_NAME
    );
    Ok(())
}