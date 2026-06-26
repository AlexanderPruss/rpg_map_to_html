use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use crate::document::{ RegexHelper};

pub static MARKDOWN_TEMPLATE_FILENAME: &str = "map_content.md";
static TEMP_PREFIX: &str = "temp_";

pub static CELL_PAGE_MD_HEADER_PREFIX: &str = "# Cell ";
pub static EXTRA_CELL_PAGE_MD_HEADER_PREFIX: &str = "# Extra Cell ";
pub static COORDINATE_TOKEN: &str = "COORDINATE";

/// Each page generates an h1 header that identifies it and its cell coordinate.
pub enum MarkdownHeaders {
    /// Cell page headers look like '# Cell {{COORDINATE}}'
    CellPage,
    /// Extra cell page headers look like '# Extra Cell {{COORDINATE}}'
    ExtraCellPage,
}

impl MarkdownHeaders {
    fn get_prefix(&self) -> &str {
        match self {
            MarkdownHeaders::CellPage => CELL_PAGE_MD_HEADER_PREFIX,
            MarkdownHeaders::ExtraCellPage => EXTRA_CELL_PAGE_MD_HEADER_PREFIX,
        }
    }
}

/// Gets the Markdown file where map content will be written and edited by users.
///
/// Creates this file if it doesn't yet exist. The file is returned in read-only mode.
pub fn get_markdown_content_read_file(target_directory: &PathBuf) -> File {
    let mut path = PathBuf::from(target_directory);
    path.push(MARKDOWN_TEMPLATE_FILENAME);
    if path.exists() {
        File::open(&path).unwrap()
    } else {
        File::create(&path).unwrap();
        File::open(&path).unwrap()
    }
}

/// If content for any cells is missing from the markdown file, adds a page for the cell.
///
/// Tries to order the added pages alphabetically, but this can fail if the markdown file itself
/// is not alphabetically ordered anymore. Failure means we'll have some partially-ordered pages inside
/// the unordered document.
pub fn add_md_content_for_missing_cells(target_directory: &PathBuf, coordinates: Vec<&String>, regex_helper: &RegexHelper) {
    let markdown_file = get_markdown_content_read_file(target_directory);
    let mut temp_path = PathBuf::from(target_directory);
    temp_path.push(TEMP_PREFIX.to_string() + MARKDOWN_TEMPLATE_FILENAME);
    let temp_markdown_file = File::create(&temp_path).unwrap();
    let mut temp_writer = BufWriter::new(temp_markdown_file);

    let existing_page_coordinates = get_existing_cell_page_coordinates(target_directory);
    let mut ordered_cells_to_add: Vec<&String> = coordinates
        .into_iter()
        .filter(|coord| !existing_page_coordinates.contains(*coord))
        .collect();
    ordered_cells_to_add.sort();
    let mut ordered_cells_iter = ordered_cells_to_add.into_iter();
    let mut next_cell_coordinate_to_add = ordered_cells_iter.next();

    for line in BufReader::new(markdown_file).lines().map_while(Result::ok) {
        let found_coordinate_option = cell_header_to_coordinate(&line, MarkdownHeaders::CellPage);
        // Before we write in an existing page, first write in any new coordinates that come before it
        if let Some(found_coordinate) = found_coordinate_option {
            while let Some(coordinate) = next_cell_coordinate_to_add
                && *coordinate < found_coordinate
            {
                write_page_for_coordinate(&mut temp_writer, coordinate, regex_helper);
                next_cell_coordinate_to_add = ordered_cells_iter.next();
            }
        }
        temp_writer.write(line.as_bytes()).unwrap();
        temp_writer.write("\n".as_bytes()).unwrap();
    }
    // Add any other new coordinate pages we haven't gotten to.
    while let Some(coordinate) = next_cell_coordinate_to_add {
        write_page_for_coordinate(&mut temp_writer, coordinate, regex_helper);
        next_cell_coordinate_to_add = ordered_cells_iter.next();
    }
    temp_writer.flush().unwrap();

    let mut final_path = PathBuf::from(target_directory);
    final_path.push(MARKDOWN_TEMPLATE_FILENAME);
    std::fs::rename(temp_path, final_path).unwrap();
}

/// Writes a [cell_page_template.md] filled in with the [coordinate] into the provided [writer].
fn write_page_for_coordinate(writer: &mut BufWriter<File>, coordinate: &String, regex_helper: &RegexHelper) {
    let template_lines = include_bytes!("templates/cell_page_template.md").lines();
    let coordinate_replacement = HashMap::from([(COORDINATE_TOKEN, coordinate)]);
    template_lines.for_each(|line_result| {
        let line = line_result.unwrap();
        let line = regex_helper.replace_tokens(line.as_str(), &coordinate_replacement);
        writer.write(line.as_bytes()).unwrap();
        writer.write("\n".as_bytes()).unwrap();
    });
    writer.write("\n".as_bytes()).unwrap();
}

/// Finds the coordinates of all cell pages in the markdown content file by finding pages prepended
/// by the [CELL_PAGE_MD_HEADER_PREFIX].
fn get_existing_cell_page_coordinates(target_directory: &PathBuf) -> HashSet<String> {
    let markdown_file = get_markdown_content_read_file(target_directory);
    BufReader::new(markdown_file)
        .lines()
        .map_while(Result::ok)
        .map(|header| {
            cell_header_to_coordinate(&header, MarkdownHeaders::CellPage).unwrap_or_default()
        })
        .collect()
}

/// Given a cell_page_header, retrieves the coordinate encoded inside it, stripping whitespace.
pub fn cell_header_to_coordinate(
    cell_page_header: &String,
    header_type: MarkdownHeaders,
) -> Option<String> {
    if !cell_page_header.starts_with(header_type.get_prefix()) {
        return None;
    }
    let mut coordinate = cell_page_header[header_type.get_prefix().len()..].to_string();
    coordinate.retain(|c| !c.is_whitespace());
    Some(coordinate)
}

#[cfg(test)]
mod test {

    mod add_md_content_for_missing_cells {
        use crate::document::markdown_content::add_md_content_for_missing_cells;
        use crate::document::test::fixtures::{assert_files_equal, get_test_cases_path};
        use std::fs;
        use std::path::PathBuf;
        use crate::document::RegexHelper;

        #[test]
        fn create_and_fills_a_content_md_if_none_exists() {
            let mut test_cases_path = get_test_cases_path();
            test_cases_path.push("add_content");
            test_cases_path.push("creates_new_markdown_file");
            let mut expected_file = PathBuf::from(&test_cases_path);
            expected_file.push("expected");
            expected_file.push("map_content.md");
            let mut result_dir = PathBuf::from(&test_cases_path);
            result_dir.push("result");
            let mut result_file = PathBuf::from(&result_dir);
            result_file.push("map_content.md");

            //Cleanup previous test artifacts.
            if fs::exists(&result_file).unwrap() {
                fs::remove_file(&result_file).unwrap();
            }

            add_md_content_for_missing_cells(&result_dir, vec![&"1".to_string(), &"3".to_string()], &RegexHelper::new());

            assert_files_equal(&expected_file, &result_file);
        }

        #[test]
        fn adds_missing_cells_to_already_existing_content_mds() {
            let mut test_cases_path = get_test_cases_path();
            test_cases_path.push("add_content");
            test_cases_path.push("adds_new_cells_to_existing_markdown_file");
            let mut expected_file = PathBuf::from(&test_cases_path);
            expected_file.push("expected");
            expected_file.push("map_content.md");
            let mut result_dir = PathBuf::from(&test_cases_path);
            result_dir.push("result");
            let mut result_file = PathBuf::from(&result_dir);
            result_file.push("map_content.md");

            //Cleanup previous test artifacts.
            if fs::exists(&result_file).unwrap() {
                fs::remove_file(&result_file).unwrap();
            }

            let coords: Vec<String> = (0..=9).map(|coord| coord.to_string()).collect();
            add_md_content_for_missing_cells(&result_dir, coords.iter().collect(), &RegexHelper::new());

            assert_files_equal(&expected_file, &result_file);
        }
    }

    mod get_existing_cell_page_coordinates {
        use crate::document::markdown_content::get_existing_cell_page_coordinates;
        use crate::document::test::fixtures::get_test_cases_path;
        use std::collections::HashSet;

        #[test]
        fn retrieves_all_valid_coordinates_from_a_content_markdown_file() {
            let mut path = get_test_cases_path();
            path.push("get_coordinates");
            let expected: HashSet<String> =
                HashSet::from(["123.456".to_string(), "abc.def".to_string(), "".to_string()]);

            let coordinates = get_existing_cell_page_coordinates(&path);

            assert_eq!(expected, coordinates);
        }
    }

    mod cell_header_to_coordinate {
        use crate::document::markdown_content::MarkdownHeaders::{CellPage, ExtraCellPage};
        use crate::document::markdown_content::cell_header_to_coordinate;

        #[test]
        fn retrieves_coordinates_from_cell_headers() {
            let header = "# Cell 012.345".to_string();
            let coordinate = cell_header_to_coordinate(&header, CellPage);
            assert_eq!("012.345".to_string(), coordinate.unwrap())
        }

        #[test]
        fn retrieves_coordinates_from_extr_cell_headers() {
            let header = "# Extra Cell 012.345".to_string();
            let coordinate = cell_header_to_coordinate(&header, ExtraCellPage);
            assert_eq!("012.345".to_string(), coordinate.unwrap())
        }

        #[test]
        fn retrieves_coordinates_from_cell_headers_stripping_whitespace() {
            let header = "# Cell   012  .345  ".to_string();
            let coordinate = cell_header_to_coordinate(&header, CellPage);
            assert_eq!("012.345".to_string(), coordinate.unwrap())
        }

        #[test]
        fn retrieves_coordinates_from_extra_cell_headers_stripping_whitespace() {
            let header = "# Extra Cell   012  .345  ".to_string();
            let coordinate = cell_header_to_coordinate(&header, ExtraCellPage);
            assert_eq!("012.345".to_string(), coordinate.unwrap())
        }

        #[test]
        fn returns_none_if_the_header_is_invalid() {
            let header = "# Wrong 012.345".to_string();
            let coordinate = cell_header_to_coordinate(&header, CellPage);
            assert_eq!(None, coordinate);
            let coordinate = cell_header_to_coordinate(&header, ExtraCellPage);
            assert_eq!(None, coordinate)
        }
    }
}
