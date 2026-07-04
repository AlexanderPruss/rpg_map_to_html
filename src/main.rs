use std::env;
use std::path::PathBuf;
use rpg_map_to_html::generate_map;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        print_help(args.get(0).unwrap())
    }
    if args[1].starts_with("-") && args[1].to_lowercase().contains("h") {
        print_help(args.get(0).unwrap())
    }
    generate_map(PathBuf::from(args.get(1).unwrap()))
}

fn print_help(executable: &String) {
    //TODO: How unusable should the first release be, lol
    println!("Converts map images to markdown content and html.");
    println!(r"Usage: {executable} {{config}}
    config: the config file")
}
