use rpg_map_to_html::config::LAST_USED_CONFIG;
use rpg_map_to_html::{config, generate_docs, read_input_yes_no_with_default};
use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    let executable = args.get(0).unwrap();
    if args.len() > 2 {
        print_help(executable)
    }
    if args.len() > 1 && args[1].starts_with("-h") {
        print_help(executable)
    }
    if args.len() == 2 {
        let config_path = PathBuf::from(args.get(1).unwrap());
        return generate_docs(
            config::parse_config(config_path).expect("Couldn't parse the config file"),
        );
    }

    interactive_mode()
}

fn interactive_mode() {
    if fs::exists(LAST_USED_CONFIG).unwrap() {
        println!(
            "Found an existing config at {LAST_USED_CONFIG}. Use this existing config? Y/N (default Y)"
        );
        let mut input = String::new();
        if read_input_yes_no_with_default(&mut input, true) {
            return generate_with_existing_config();
        }
        println!(
            "Ok, the existing config will not be used. Proceeding with generating a config interactively."
        );
    }
    let config = config::generate_config_interactively();
    println!("Starting document generation.");
    generate_docs(config);
}

fn generate_with_existing_config() {
    println!("Using the existing config at {LAST_USED_CONFIG}");
    generate_docs(
        config::parse_config(PathBuf::from(LAST_USED_CONFIG))
            .expect("Couldn't parse the last-used config file."),
    )
}

fn print_help(executable: &String) {
    println!(
        r"
    Converts map images to markdown content and html.
    Usage: {executable} {{config}}
        config(optional): the config file

        If no config is present, an interactive mode will start and try to create one for you."
    )
}
