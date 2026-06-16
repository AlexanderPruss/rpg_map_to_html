use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub fn get_test_cases_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.push("test_resources");
    path.push("markdown_templating");
    path.push("test_cases");
    path
}

pub fn assert_files_equal(first: &PathBuf, second: &PathBuf) {
    let mut first_lines = BufReader::new(File::open(first).unwrap()).lines();
    let mut second_lines = BufReader::new(File::open(first).unwrap()).lines();
    let mut line = 0;

    for first_line in first_lines.map_while(Result::ok) {
        let second_line = second_lines.next().unwrap().unwrap();
        assert_eq!(
            first_line, second_line,
            "Lines were different on line {}",
            line
        );
        line += 1;
    }

    if let Some(too_many_lines) = second_lines.next() {
        panic!(
            "File {:?} was too large. The other file ended in {} lines, but line {} exists in the first file with content \n{}",
            second,
            line,
            line + 1,
            too_many_lines.unwrap()
        )
    }
}
