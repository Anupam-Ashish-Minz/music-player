mod ui;

use clap::Parser;
use std::fs;
use std::{error::Error, path::PathBuf};
use ui::ui::draw_lists;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    #[arg(short, long, default_value_t = String::from("assets"))]
    path: String,
}

fn get_file_list(root: PathBuf, list: &mut Vec<String>) -> Result<(), std::io::Error> {
    let mut rval = Ok(());
    for file in fs::read_dir(root)? {
        let file = file?;
        let file_type = file.file_type()?;
        if file_type.is_file() {
            list.push(file.path().to_string_lossy().to_string());
        } else if file_type.is_dir() {
            rval = get_file_list(file.path(), list);
        }
    }
    return rval;
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut file_list = Vec::new();
    get_file_list(PathBuf::from(&args.path[..]), &mut file_list)?;
    draw_lists(file_list)?;

    return Ok(());
}
