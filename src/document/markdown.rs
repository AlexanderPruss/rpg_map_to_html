use std::fs::File;
use std::path::{PathBuf};
use crate::geometry::Cell;

static MARKDOWN_TEMPLATE_FILENAME: &str = "map_content.md";

/// Gets the Markdown file where map content will be written and edited. Creates this
/// file if it doesn't yet exist.
fn get_markdown_content_file(cell: Cell, target_directory: &PathBuf) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(MARKDOWN_TEMPLATE_FILENAME);
    if !path.exists() {
        File::create(&path).unwrap();
    }
    File::open(&path).unwrap()
    path

}