use crate::PixelPoint;
use crate::config::TemplateConfig;
use crate::document::html::NextLineMatch::*;
use crate::document::markdown_content::{
    MarkdownHeaders, add_md_content_for_missing_cells, cell_header_to_coordinate,
    get_markdown_content_read_file,
};
use crate::document::{DocumentContext, Template, TemplateFiles, Token, replace_if_contains};
use crate::geometry::CellMap;
use crate::image_handling::map_cutout::CutoutImage;
use crate::image_handling::table_of_contents::TableOfContentsMapImage;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Lines, Write};
use std::iter::Peekable;
use std::path::PathBuf;

static HTML_DOC_FILENAME: &str = "rpg_map_doc.html";
static STYLES_FILENAME: &str = "styles.css";

struct ReaderWriter<I: Iterator<Item = std::io::Result<String>>> {
    writer: BufWriter<File>,
    md_lines: Peekable<I>,
}

impl ReaderWriter<Lines<BufReader<File>>> {
    fn new(output_file: File, md_content_file: File) -> ReaderWriter<Lines<BufReader<File>>> {
        ReaderWriter {
            writer: BufWriter::new(output_file),
            md_lines: BufReader::new(md_content_file).lines().peekable(),
        }
    }
}

/// Generates or appends to any existing markdown content found. Then uses this markdown content
/// to generate the final html document.
pub fn write_html_doc(
    target_directory: &PathBuf,
    title: &String,
    cell_map: &CellMap,
    cutout_width_height: PixelPoint,
    table_of_contents_images: &Vec<TableOfContentsMapImage>,
    cutout_images: &Vec<CutoutImage>,
    config: &Option<TemplateConfig>,
) {
    let coordinates_to_keep: HashSet<&String> = cutout_images
        .iter()
        .map(|cutout| &cutout.coordinate)
        .collect();
    let mut ordered_cells: Vec<&String> = cell_map
        .cells_by_coordinate
        .iter()
        .filter(|(coordinate, _)| coordinates_to_keep.contains(coordinate))
        .map(|(coordinate, _cell)| coordinate)
        .collect();
    ordered_cells.sort();
    add_md_content_for_missing_cells(target_directory, ordered_cells);

    let offset_by_coordinate = cutout_images
        .iter()
        .map(|cutout_image| (cutout_image.coordinate.clone(), cutout_image))
        .collect();
    let context = DocumentContext::new(
        title,
        cutout_width_height,
        target_directory,
        &config,
        cell_map,
        &table_of_contents_images,
        offset_by_coordinate,
    );
    generate_styles(target_directory, &config, &cutout_width_height);
    generate_html_doc(context);
}

fn generate_styles(
    target_directory: &PathBuf,
    config: &Option<TemplateConfig>,
    cutout_width_height: &PixelPoint,
) {
    let mut map_docs_path = PathBuf::from(target_directory);
    map_docs_path.push(STYLES_FILENAME);
    let mut result_writer = BufWriter::new(File::create(map_docs_path).unwrap());
    let width_height_css = format!(
        "width:{}px; height: {}px;",
        cutout_width_height.x, cutout_width_height.y
    );
    let width_height_css_token = &Token::WidthHeightCss;
    for line in TemplateFiles::Styles
        .get_template_lines(config)
        .map_while(Result::ok)
    {
        let line = replace_if_contains(line, width_height_css_token, &width_height_css);
        result_writer.write(line.as_bytes()).unwrap();
        result_writer.write("\n".as_bytes()).unwrap();
    }
    result_writer.flush().unwrap();
}

fn create_map_docs_file(target_directory: &PathBuf) -> File {
    let mut map_docs_path = PathBuf::from(target_directory);
    map_docs_path.push(HTML_DOC_FILENAME);
    File::create(map_docs_path).unwrap()
}

fn generate_html_doc(mut context: DocumentContext) {
    let output_file = create_map_docs_file(&context.target_directory);
    let md_content_file = get_markdown_content_read_file(&context.target_directory);
    let mut reader_writer = ReaderWriter::new(output_file, md_content_file);
    write_template(
        &mut context,
        &mut reader_writer,
        &"".to_string(),
        TemplateFiles::MapDocs,
    );
    reader_writer.writer.flush().unwrap();
}

/// If the [line] contains a [Template], the template is unwrapped and filled.
///
/// Otherwise, any [Token]s the line contains are replaced with their values, and the
/// line is written with an endline.
fn fill_template_or_write_line<I: Iterator<Item = Result<String, std::io::Error>>>(
    mut context: &mut DocumentContext,
    mut reader_writer: &mut ReaderWriter<I>,
    line: String,
) {
    let line_str = line.as_str();
    let templates_filled = fill_templates_if_found(line_str, &mut reader_writer, &mut context);
    if templates_filled {
        return;
    }
    let line = context.replace_tokens(line_str);
    reader_writer.writer.write(line.as_bytes()).unwrap();
    reader_writer.writer.write("\n".as_bytes()).unwrap();
}

fn fill_templates_if_found<I: Iterator<Item = Result<String, std::io::Error>>>(
    line: &str,
    reader_writer: &mut ReaderWriter<I>,
    context: &mut DocumentContext,
) -> bool {
    let mut template_filled = false;
    context
        .template_matches(line)
        .iter()
        .for_each(|template_match| match template_match.template {
            Template::TableOfContents => {
                template_filled = true;
                fill_table_of_contents_templates(
                    context,
                    reader_writer,
                    &template_match.leading_whitespace,
                );
            }
            Template::TableOfContentsPolygonLinks => {
                template_filled = true;
                fill_table_of_contents_polygon_links(
                    context,
                    reader_writer,
                    &template_match.leading_whitespace,
                );
            }
            Template::ZoomedInMapPolygonLinks => {
                template_filled = true;
                fill_zoomed_in_map_polygon_links(
                    context,
                    reader_writer,
                    &template_match.leading_whitespace,
                );
            }
            Template::CellPages => {
                template_filled = true;
                fill_cell_pages(context, reader_writer, &template_match.leading_whitespace);
            }
            Template::LeftColumn => {
                template_filled = true;
                fill_column(
                    context,
                    reader_writer,
                    &template_match.leading_whitespace,
                    Column::Left,
                )
            }
            Template::RightColumn => {
                template_filled = true;
                fill_column(
                    context,
                    reader_writer,
                    &template_match.leading_whitespace,
                    Column::Right,
                )
            }
            Template::ExtraRightColumn => {
                template_filled = true;
                fill_column(
                    context,
                    reader_writer,
                    &template_match.leading_whitespace,
                    Column::ExtraRight,
                )
            }
            Template::Sections => {
                template_filled = true;
                fill_sections(context, reader_writer, &template_match.leading_whitespace)
            }
        });
    template_filled
}

enum Column {
    Left,
    Right,
    ExtraRight,
}

fn fill_table_of_contents_templates<I: Iterator<Item = Result<String, std::io::Error>>>(
    context: &mut DocumentContext,
    reader_writer: &mut ReaderWriter<I>,
    prefix: &String,
) {
    for table_of_contents_image in context.table_of_contents_images {
        context.set_current_table_of_contents_image(&table_of_contents_image);
        write_template(
            context,
            reader_writer,
            prefix,
            TemplateFiles::TableOfContents,
        );
    }
}

fn fill_table_of_contents_polygon_links<I: Iterator<Item = Result<String, std::io::Error>>>(
    context: &mut DocumentContext,
    reader_writer: &mut ReaderWriter<I>,
    prefix: &String,
) {
    let table_of_contents_image = context.current_table_of_contents_image.expect(
        format!("Tried to fill a [[{}]] template, but the context had not set a current table of contents image.",
                Template::TableOfContentsPolygonLinks).as_str()
    );
    write_polygon_links(
        &mut reader_writer.writer,
        context.cell_map,
        prefix,
        table_of_contents_image.offset * -1,
        table_of_contents_image
            .coordinates_contained
            .iter()
            .filter(|coordinate| {
                context
                    .cutout_images_by_coordinate
                    .get(*coordinate)
                    .is_some()
            })
            .collect(),
    )
}

fn fill_zoomed_in_map_polygon_links<I: Iterator<Item = Result<String, std::io::Error>>>(
    context: &mut DocumentContext,
    reader_writer: &mut ReaderWriter<I>,
    prefix: &String,
) {
    let coordinate = context.current_coordinate.as_ref().expect(
        format!(
            "Tried to fill a [[{}]] template, but the context had not set current coordinate",
            Template::ZoomedInMapPolygonLinks
        )
        .as_str(),
    );
    let cell = context
        .cell_map
        .cells_by_coordinate
        .get(coordinate)
        .unwrap();
    let cutout_image = *context.cutout_images_by_coordinate.get(coordinate).unwrap();
    write_polygon_links(
        &mut reader_writer.writer,
        context.cell_map,
        prefix,
        cutout_image.offset_from_original_image,
        cell.neighbor_coordinates.iter().collect(),
    )
}

fn write_polygon_links(
    writer: &mut BufWriter<File>,
    cell_map: &CellMap,
    prefix: &str,
    offset: PixelPoint,
    coordinates_to_link: HashSet<&String>,
) {
    let mut coordinates_to_link: Vec<&String> = coordinates_to_link.into_iter().collect();
    coordinates_to_link.sort();
    for coordinate in coordinates_to_link {
        let cell = cell_map.cells_by_coordinate.get(coordinate).expect(
            format!(
                "Expected to draw a link for a cell with coordinate {}, but no such cell was found in the cell map",
                coordinate
            ).as_str()
        );
        let link_box = &cell.inscribed_rectangle;
        let top_left = link_box.top_left_corner + offset;
        let bottom_right = link_box.bottom_right_corner + offset;
        let width = bottom_right.x - top_left.x;
        let height = bottom_right.y - top_left.y;
        let x = top_left.x;
        let y = top_left.y;
        let html = format!(
            r#"
{prefix}    <a href=#{coordinate}>
{prefix}        <rect class="cell-link" x="{x}" y="{y}" width="{width}" height ="{height}"/>
{prefix}    </a>
"#,
        );
        writer.write(html.as_bytes()).unwrap();
    }
}

#[derive(PartialEq, Debug)]
struct StartOfCellPage {
    coordinate: Option<String>,
    title: String,
    description: String,
    extra_page: bool,
}

/// Iterates over the markdown content until it is consumed and all pages contained inside are mapped
/// to html. Further calls to templates based on the markdown reader will fail.
fn fill_cell_pages<I: Iterator<Item = Result<String, std::io::Error>>>(
    context: &mut DocumentContext,
    reader_writer: &mut ReaderWriter<I>,
    prefix: &String,
) {
    if reader_writer.md_lines.peek().is_none() {
        panic!(
            "Tried to fill a [[{}]] template, but the markdown content file had already been read through to the end.",
            Template::CellPages
        )
    }
    loop {
        let start_of_cell_page: Option<StartOfCellPage> =
            iterate_until_page_starts(&mut reader_writer.md_lines);
        if start_of_cell_page.is_none() {
            break;
        }
        let start_of_cell_page = start_of_cell_page.unwrap();
        if start_of_cell_page.extra_page {
            context.start_new_extra_page(
                start_of_cell_page.coordinate,
                start_of_cell_page.title,
                start_of_cell_page.description,
            )
        } else {
            context.start_new_cell_page(
                start_of_cell_page.coordinate.unwrap(),
                start_of_cell_page.title,
                start_of_cell_page.description,
            );
        }
        let template = if start_of_cell_page.extra_page {
            TemplateFiles::ExtraPage
        } else {
            TemplateFiles::CellPage
        };

        write_template(context, reader_writer, prefix, template);
    }
}
fn fill_column<I: Iterator<Item = Result<String, std::io::Error>>>(
    context: &mut DocumentContext,
    reader_writer: &mut ReaderWriter<I>,
    prefix: &String,
    column: Column,
) {
    let (required_markdown_prefix, template_file, template) = match column {
        Column::Left => (
            MarkdownHeaders::LeftColumn.get_prefix(),
            TemplateFiles::LeftColumn,
            Template::LeftColumn,
        ),
        Column::Right => (
            MarkdownHeaders::RightColumn.get_prefix(),
            TemplateFiles::RightColumn,
            Template::RightColumn,
        ),
        Column::ExtraRight => (
            MarkdownHeaders::RightColumn.get_prefix(),
            TemplateFiles::ExtraRightColumn,
            Template::ExtraRightColumn,
        ),
    };
    iterate_until_header_or_eof(&mut reader_writer.md_lines);
    let expect_column_header_line = reader_writer.md_lines.next().expect(
        format!(
            "Tried to fill in a [[{template}]] Template for page {:?}, but the markdown content file was already exhausted.\n\
            You may have deleted the '## Left Column' or '## Right Column' markdown headers from that page; add them back to fix this error.",
            context.current_page_title
        ).as_str()
    );
    let expect_column_header_line = expect_column_header_line.unwrap();
    if !expect_column_header_line.starts_with(required_markdown_prefix) {
        panic!(
            "Tried to fill in a [[{template}]] Template, but the next header encountered was '{}'. The markdown line has to start with '{}' instead.",
            expect_column_header_line, required_markdown_prefix
        )
    }
    write_template(context, reader_writer, &prefix, template_file);
}

fn write_template<I: Iterator<Item = Result<String, std::io::Error>>>(
    context: &mut DocumentContext,
    reader_writer: &mut ReaderWriter<I>,
    prefix: &String,
    template: TemplateFiles,
) {
    for page_line in template
        .get_template_lines(context.config)
        .map_while(Result::ok)
    {
        let prefixed_line = format!("{}{}", &prefix, page_line);
        fill_template_or_write_line(context, reader_writer, prefixed_line);
    }
}

#[derive(PartialEq, Debug)]
struct Section {
    title: String,
    content: String,
    template: TemplateFiles,
}

fn fill_sections<I: Iterator<Item = Result<String, std::io::Error>>>(
    context: &mut DocumentContext,
    reader_writer: &mut ReaderWriter<I>,
    prefix: &String,
) {
    loop {
        let next_section: Option<Section> = read_next_section(&mut reader_writer.md_lines, prefix);
        if next_section.is_none() {
            break;
        }
        let next_section = next_section.unwrap();
        context.start_new_section(next_section.title, next_section.content);
        write_template(context, reader_writer, prefix, next_section.template);
    }
}

fn read_next_section<I: Iterator<Item = Result<String, std::io::Error>>>(
    md_lines: &mut Peekable<I>,
    prefix: &String,
) -> Option<Section> {
    iterate_until_header_or_eof(md_lines);
    let header_line = md_lines.peek();
    if header_line.is_none() {
        // It's ok if the user hasn't actually added any sections for a page.
        return None;
    }
    let header_line = header_line.unwrap().as_ref().unwrap();
    let (title, template) = if header_line.starts_with(MarkdownHeaders::Section.get_prefix()) {
        (
            header_line
                .strip_prefix(MarkdownHeaders::Section.get_prefix())
                .unwrap()
                .to_string(),
            TemplateFiles::Section,
        )
    } else if header_line.starts_with(MarkdownHeaders::HighlightedSection.get_prefix()) {
        (
            header_line
                .strip_prefix(MarkdownHeaders::HighlightedSection.get_prefix())
                .unwrap()
                .to_string(),
            TemplateFiles::HighlightedSection,
        )
    } else {
        return None;
    };
    md_lines.next();
    let raw_content = iterate_until_header_or_eof(md_lines).join("\n");
    let parser = pulldown_cmark::Parser::new(raw_content.as_str());
    let mut content = String::new();
    pulldown_cmark::html::push_html(&mut content, parser);
    let content_lines: Vec<String> = content
        .lines()
        .map(|line| format!("{prefix}{line}"))
        .collect();
    Some(Section {
        title,
        content: content_lines.join("\n"),
        template,
    })
}

fn iterate_until_page_starts<I: Iterator<Item = Result<String, std::io::Error>>>(
    md_lines: &mut Peekable<I>,
) -> Option<StartOfCellPage> {
    // Cells are H1 headers. If the next line is not a header, then we're in the wrong part of the markdown file and give up
    iterate_until_header_or_eof(md_lines);
    let page_header = match next_line_starts_with(md_lines, "# ") {
        Match => md_lines.next().unwrap().unwrap(),
        NoMatch => panic!(
            "Tried to start a new page, but the next markdown line was not a page header, but instead {}",
            md_lines.next().unwrap().unwrap()
        ),
        Eof => return None,
    };

    //The header contains the coordinate if one is present.
    let (coordinate, extra_page) =
        if page_header.starts_with(MarkdownHeaders::CellPage.get_prefix()) {
            (
                cell_header_to_coordinate(&page_header, MarkdownHeaders::CellPage),
                false,
            )
        } else {
            (
                cell_header_to_coordinate(&page_header, MarkdownHeaders::ExtraCellPage),
                true,
            )
        };

    //The first header after the page header contains the title.
    iterate_until_header_or_eof(md_lines);
    let title = match next_line_starts_with(md_lines, MarkdownHeaders::PageTitle.get_prefix()) {
        Match => {
            let page_title_line = md_lines.next().unwrap().unwrap();
            page_title_line
                .strip_prefix(MarkdownHeaders::PageTitle.get_prefix())
                .unwrap()
                .to_string()
        }
        NoMatch => panic!(
            "Tried to find a page's title, but the next markdown line was not a page title, but instead {}",
            md_lines.next().unwrap().unwrap()
        ),
        Eof => panic!(
            "Tried to find a page's title, but the markdown file ended before a title could be found.",
        ),
    };

    //The lines until the next header contain the description, if any.
    let raw_content = iterate_until_header_or_eof(md_lines).join("\n");
    let parser = pulldown_cmark::Parser::new(raw_content.as_str());
    let mut content = String::new();
    pulldown_cmark::html::push_html(&mut content, parser);
    let content_lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();

    Some(StartOfCellPage {
        coordinate,
        title,
        description: content_lines.join("\n"),
        extra_page,
    })
}

#[derive(PartialEq, Debug)]
enum NextLineMatch {
    Match,
    NoMatch,
    Eof,
}

fn next_line_starts_with<I: Iterator<Item = Result<String, std::io::Error>>>(
    md_lines: &mut Peekable<I>,
    prefix: &str,
) -> NextLineMatch {
    let peek = md_lines.peek();
    if peek.is_none() {
        return Eof;
    }
    let peek = peek.as_ref().unwrap();
    if peek.as_ref().unwrap().starts_with(prefix) {
        return Match;
    }
    NoMatch
}

/// Iterates until the next (peeked) line is either a header or EOF. Returns all lines encountered.
fn iterate_until_header_or_eof<I: Iterator<Item = Result<String, std::io::Error>>>(
    md_lines: &mut Peekable<I>,
) -> Vec<String> {
    let mut lines: Vec<String> = vec![];
    loop {
        if next_line_starts_with(md_lines, "#") != NoMatch {
            return lines;
        }
        lines.push(md_lines.next().unwrap().unwrap());
    }
}

#[cfg(test)]
mod test {
    //TODO: Should these be integration tests? But then I can't use my fixtures.
    mod write_html_doc {
        use crate::config::TemplateConfig;
        use crate::document::html::write_html_doc;
        use crate::document::test::fixtures::assert_files_equal;
        use crate::geometry::{BoundingPolygon, Cell, CellMap};
        use crate::image_handling::map_cutout::CutoutImage;
        use crate::image_handling::table_of_contents::TableOfContentsMapImage;
        use crate::{PixelBox, PixelPoint};
        use std::collections::{HashMap, HashSet};
        use std::fs;
        use std::path::PathBuf;

        struct HtmlDocTestCase {
            target_directory: PathBuf,
            expected_directory: PathBuf,
            cell_map: CellMap,
            table_of_contents_images: Vec<TableOfContentsMapImage>,
            cutout_images: Vec<CutoutImage>,
        }

        fn standard_test_case(test_case: String) -> HtmlDocTestCase {
            let mut test_case_path = PathBuf::new();
            test_case_path.push(env!("CARGO_MANIFEST_DIR"));
            test_case_path.push("test_resources");
            test_case_path.push("write_html_doc");
            test_case_path.push(&test_case);
            let mut target_directory = PathBuf::from(&test_case_path);
            target_directory.push("result");
            let mut expected_directory = PathBuf::from(&test_case_path);
            expected_directory.push("expected");

            //Remove any previous results.
            fs::read_dir(&target_directory)
                .unwrap()
                .filter(|file| {
                    !file
                        .as_ref()
                        .unwrap()
                        .file_name()
                        .to_str()
                        .unwrap()
                        .starts_with(".")
                })
                .for_each(|file| fs::remove_file(file.unwrap().path()).unwrap());

            let cell_map = CellMap {
                cells_by_coordinate: HashMap::from([
                    (
                        "coord-1".to_string(),
                        Cell {
                            coordinate: "coord-1".to_string(),
                            neighbor_coordinates: HashSet::from(["coord-2".to_string()]),
                            center_point: PixelPoint { x: 5, y: 10 },
                            bounding_polygon: BoundingPolygon { points: vec![] },
                            inscribed_rectangle: PixelBox {
                                top_left_corner: PixelPoint { x: 1, y: 1 },
                                bottom_right_corner: PixelPoint { x: 10, y: 20 },
                            },
                        },
                    ),
                    (
                        "coord-2".to_string(),
                        Cell {
                            coordinate: "coord-2".to_string(),
                            neighbor_coordinates: HashSet::from([
                                "coord-1".to_string(),
                                "coord-3".to_string(),
                            ]),
                            center_point: PixelPoint { x: 25, y: 30 },
                            bounding_polygon: BoundingPolygon { points: vec![] },
                            inscribed_rectangle: PixelBox {
                                top_left_corner: PixelPoint { x: 21, y: 21 },
                                bottom_right_corner: PixelPoint { x: 30, y: 30 },
                            },
                        },
                    ),
                    (
                        "coord-3".to_string(),
                        Cell {
                            coordinate: "coord-3".to_string(),
                            neighbor_coordinates: HashSet::from(["coord-2".to_string()]),
                            center_point: PixelPoint { x: 45, y: 50 },
                            bounding_polygon: BoundingPolygon { points: vec![] },
                            inscribed_rectangle: PixelBox {
                                top_left_corner: PixelPoint { x: 41, y: 41 },
                                bottom_right_corner: PixelPoint { x: 50, y: 50 },
                            },
                        },
                    ),
                ]),
            };
            let table_of_contents_images: Vec<TableOfContentsMapImage> = vec![
                TableOfContentsMapImage {
                    filename: "first_toc.jpg".to_string(),
                    size: PixelPoint { x: 1, y: 2 },
                    offset: PixelPoint { x: 3, y: 4 },
                    coordinates_contained: HashSet::from([
                        "coord-1".to_string(),
                        "coord-2".to_string(),
                    ]),
                },
                TableOfContentsMapImage {
                    filename: "second_toc.jpg".to_string(),
                    size: PixelPoint { x: 4, y: 5 },
                    offset: PixelPoint { x: 6, y: 7 },
                    coordinates_contained: HashSet::from([
                        "coord-2".to_string(),
                        "coord-3".to_string(),
                    ]),
                },
            ];
            let cutout_images: Vec<CutoutImage> = vec![
                CutoutImage {
                    coordinate: "coord-1".to_string(),
                    offset_from_original_image: PixelPoint { x: 10, y: 20 },
                    image_size: PixelPoint { x: 30, y: 40 },
                },
                CutoutImage {
                    coordinate: "coord-2".to_string(),
                    offset_from_original_image: PixelPoint { x: 100, y: 200 },
                    image_size: PixelPoint { x: 300, y: 400 },
                },
                CutoutImage {
                    coordinate: "coord-3".to_string(),
                    offset_from_original_image: PixelPoint { x: 150, y: 250 },
                    image_size: PixelPoint { x: 350, y: 450 },
                },
            ];

            HtmlDocTestCase {
                target_directory,
                expected_directory,
                cell_map,
                table_of_contents_images,
                cutout_images,
            }
        }

        #[test]
        fn creates_a_new_html_document_with_default_content() {
            let test_case = standard_test_case("new_document".to_string());
            write_html_doc(
                &test_case.target_directory,
                &"new_document".to_string(),
                &test_case.cell_map,
                PixelPoint { x: 99, y: 199 },
                &test_case.table_of_contents_images,
                &test_case.cutout_images,
                &None,
            );

            let mut matching_file = test_case.target_directory;
            fs::read_dir(test_case.expected_directory)
                .unwrap()
                .for_each(|entry| {
                    let entry = entry.unwrap();
                    matching_file.push(entry.file_name());
                    assert_files_equal(&entry.path(), &matching_file);
                    matching_file.push("..");
                })
        }

        #[test]
        fn updates_html_documents_using_user_content() {
            let test_case = standard_test_case("updated_document".to_string());

            //Write the 'user-input' map_content.md into the target directory.
            let mut user_content = PathBuf::from(&test_case.target_directory);
            user_content.push("..");
            user_content.push("map_content.md");
            let mut user_content_destination = PathBuf::from(&test_case.target_directory);
            user_content_destination.push("map_content.md");
            fs::copy(user_content, user_content_destination).unwrap();

            write_html_doc(
                &test_case.target_directory,
                &"updated_document".to_string(),
                &test_case.cell_map,
                PixelPoint { x: 99, y: 199 },
                &test_case.table_of_contents_images,
                &test_case.cutout_images,
                &None,
            );

            let mut matching_file = test_case.target_directory;
            fs::read_dir(test_case.expected_directory)
                .unwrap()
                .for_each(|entry| {
                    let entry = entry.unwrap();
                    matching_file.push(entry.file_name());
                    assert_files_equal(&entry.path(), &matching_file);
                    matching_file.push("..");
                })
        }

        #[test]
        fn allows_user_defined_templates() {
            let test_case = standard_test_case("custom_template".to_string());
            let mut custom_html_template = PathBuf::from(&test_case.target_directory);
            custom_html_template.push("..");
            custom_html_template.push("custom_template.html");
            let template_config = Some(TemplateConfig {
                styles_override: None,
                document_html_override: Some(custom_html_template),
                table_of_contents_html_override: None,
                cell_page_html_override: None,
                extra_cell_page_html_override: None,
                left_column_html_override: None,
                right_column_html_override: None,
                extra_right_column_html_override: None,
                section_html_override: None,
                highlighted_section_html_override: None,
                zoomed_in_map_image_size: None,
            });

            //The custom template here just prints a single "Custom template!" string.
            write_html_doc(
                &test_case.target_directory,
                &"updated_document".to_string(),
                &test_case.cell_map,
                PixelPoint { x: 99, y: 199 },
                &test_case.table_of_contents_images,
                &test_case.cutout_images,
                &template_config,
            );

            let mut matching_file = test_case.target_directory;
            fs::read_dir(test_case.expected_directory)
                .unwrap()
                .for_each(|entry| {
                    let entry = entry.unwrap();
                    matching_file.push(entry.file_name());
                    assert_files_equal(&entry.path(), &matching_file);
                    matching_file.push("..");
                })
        }
    }

    mod read_next_section {
        use crate::document::TemplateFiles;
        use crate::document::html::{Section, read_next_section};

        #[test]
        fn iterates_to_pages_and_identifies_the_next_section() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("Filler line".to_string()),
                Ok("### Section Title".to_string()),
                Ok("".to_string()),
                Ok("Markdown, including [links](#000.001)".to_string()),
                Ok("Another **line** of markdown".to_string()),
                Ok("".to_string()),
                Ok("# Next Header starting".to_string()),
            ];
            let prefix = "prefix: ".to_string();
            let mut iterator = md_lines.into_iter().peekable();

            let section = read_next_section(&mut iterator, &prefix);

            let expected = Some(Section {
                title: "Section Title".to_string(),
                content: "prefix: <p>Markdown, including <a href=\"#000.001\">links</a>\nprefix: Another <strong>line</strong> of markdown</p>".to_string(),
                template: TemplateFiles::Section,
            });
            assert_eq!(expected, section);
        }

        #[test]
        fn iterates_to_pages_and_identifies_the_next_highlighted_section() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("Filler line".to_string()),
                Ok("#### Section Title".to_string()),
                Ok("".to_string()),
                Ok("Markdown, including [links](#000.001)".to_string()),
                Ok("Another **line** of markdown".to_string()),
                Ok("".to_string()),
                Ok("# Next Header starting".to_string()),
            ];
            let prefix = "prefix: ".to_string();
            let mut iterator = md_lines.into_iter().peekable();

            let section = read_next_section(&mut iterator, &prefix);

            let expected = Some(Section {
                title: "Section Title".to_string(),
                content: "prefix: <p>Markdown, including <a href=\"#000.001\">links</a>\nprefix: Another <strong>line</strong> of markdown</p>".to_string(),
                template: TemplateFiles::HighlightedSection,
            });
            assert_eq!(expected, section);
        }

        #[test]
        fn identifies_sections_if_the_iterator_is_already_at_a_valid_section_header() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("### Section Title".to_string()),
                Ok("".to_string()),
                Ok("Markdown, including [links](#000.001)".to_string()),
                Ok("Another **line** of markdown".to_string()),
                Ok("".to_string()),
                Ok("# Next Header starting".to_string()),
            ];
            let prefix = "prefix: ".to_string();
            let mut iterator = md_lines.into_iter().peekable();

            let section = read_next_section(&mut iterator, &prefix);

            let expected = Some(Section {
                title: "Section Title".to_string(),
                content: "prefix: <p>Markdown, including <a href=\"#000.001\">links</a>\nprefix: Another <strong>line</strong> of markdown</p>".to_string(),
                template: TemplateFiles::Section,
            });
            assert_eq!(expected, section);
        }

        #[test]
        fn returns_none_if_eof_is_reached() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("Filler line".to_string()),
                Ok("File is about to end".to_string()),
            ];
            let prefix = "prefix: ".to_string();
            let mut iterator = md_lines.into_iter().peekable();

            let section = read_next_section(&mut iterator, &prefix);

            assert_eq!(None, section);
        }

        #[test]
        fn returns_none_if_the_next_header_is_not_a_section() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("Filler line".to_string()),
                Ok("# Wrong header".to_string()),
                Ok("### Now we get a valid section".to_string()),
                Ok("But it's too late".to_string()),
            ];
            let prefix = "prefix: ".to_string();
            let mut iterator = md_lines.into_iter().peekable();

            let section = read_next_section(&mut iterator, &prefix);

            assert_eq!(None, section);
        }
    }

    mod iterate_until_page_starts {
        use crate::document::html::{StartOfCellPage, iterate_until_page_starts};

        #[test]
        fn iterates_to_pages_and_identifies_cell_page_information() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("Filler line".to_string()),
                Ok("# Cell ABC".to_string()),
                Ok("".to_string()),
                Ok("## Title - Page Title".to_string()),
                Ok("Markdown, including [links](#000.001)".to_string()),
                Ok("Another **line** of markdown".to_string()),
                Ok("".to_string()),
                Ok("# Next Header starting".to_string()),
            ];
            let mut iterator = md_lines.into_iter().peekable();

            let start_of_page = iterate_until_page_starts(&mut iterator);

            let expected = Some(StartOfCellPage {
                title: "Page Title".to_string(),
                description: "<p>Markdown, including <a href=\"#000.001\">links</a>\nAnother <strong>line</strong> of markdown</p>".to_string(),
                coordinate: Some("ABC".to_string()),
                extra_page: false,
            });
            assert_eq!(expected, start_of_page);
        }

        #[test]
        fn iterates_to_pages_and_identifies_extra_cell_page_information() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("Filler line".to_string()),
                Ok("# Extra Cell ABC".to_string()),
                Ok("".to_string()),
                Ok("## Title - Page Title".to_string()),
                Ok("Markdown, including [links](#000.001)".to_string()),
                Ok("Another **line** of markdown".to_string()),
                Ok("".to_string()),
                Ok("# Next Header starting".to_string()),
            ];
            let mut iterator = md_lines.into_iter().peekable();

            let start_of_page = iterate_until_page_starts(&mut iterator);

            let expected = Some(StartOfCellPage {
                title: "Page Title".to_string(),
                description: "<p>Markdown, including <a href=\"#000.001\">links</a>\nAnother <strong>line</strong> of markdown</p>".to_string(),
                coordinate: Some("ABC".to_string()),
                extra_page: true,
            });
            assert_eq!(expected, start_of_page);
        }

        #[test]
        fn identifies_pages_if_the_iterator_is_already_at_a_valid_page_header() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("# Cell ABC".to_string()),
                Ok("".to_string()),
                Ok("## Title - Page Title".to_string()),
                Ok("Markdown, including [links](#000.001)".to_string()),
                Ok("Another **line** of markdown".to_string()),
                Ok("".to_string()),
                Ok("# Next Header starting".to_string()),
            ];
            let mut iterator = md_lines.into_iter().peekable();

            let start_of_page = iterate_until_page_starts(&mut iterator);

            let expected = Some(StartOfCellPage {
                title: "Page Title".to_string(),
                description: "<p>Markdown, including <a href=\"#000.001\">links</a>\nAnother <strong>line</strong> of markdown</p>".to_string(),
                coordinate: Some("ABC".to_string()),
                extra_page: false,
            });
            assert_eq!(expected, start_of_page);
        }

        #[test]
        fn allows_pages_to_not_have_coordinates() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("Filler line".to_string()),
                Ok("# Extra Cell".to_string()),
                Ok("".to_string()),
                Ok("## Title - Page Title".to_string()),
                Ok("Markdown, including [links](#000.001)".to_string()),
                Ok("Another **line** of markdown".to_string()),
                Ok("".to_string()),
                Ok("# Next Header starting".to_string()),
            ];
            let mut iterator = md_lines.into_iter().peekable();

            let start_of_page = iterate_until_page_starts(&mut iterator);

            let expected = Some(StartOfCellPage {
                title: "Page Title".to_string(),
                description: "<p>Markdown, including <a href=\"#000.001\">links</a>\nAnother <strong>line</strong> of markdown</p>".to_string(),
                coordinate: None,
                extra_page: true,
            });
            assert_eq!(expected, start_of_page);
        }

        #[test]
        fn allows_empty_page_descriptions() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("# Cell ABC".to_string()),
                Ok("".to_string()),
                Ok("## Title - Page Title".to_string()),
                Ok("# Next Header starting, no page description here".to_string()),
            ];
            let mut iterator = md_lines.into_iter().peekable();

            let start_of_page = iterate_until_page_starts(&mut iterator);

            let expected = Some(StartOfCellPage {
                title: "Page Title".to_string(),
                description: "".to_string(),
                coordinate: Some("ABC".to_string()),
                extra_page: false,
            });
            assert_eq!(expected, start_of_page);
        }

        #[test]
        fn returns_none_if_the_file_ends_before_a_page_is_encountered() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("Filler".to_string()),
                Ok("File is about to end".to_string()),
            ];
            let mut iterator = md_lines.into_iter().peekable();

            let start_of_page = iterate_until_page_starts(&mut iterator);

            assert_eq!(None, start_of_page);
        }

        #[test]
        #[should_panic]
        fn panics_if_a_header_that_is_not_a_page_header_is_encountered() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("Filler".to_string()),
                Ok("## This is not a page header!".to_string()),
            ];
            let mut iterator = md_lines.into_iter().peekable();

            iterate_until_page_starts(&mut iterator);
        }

        #[test]
        #[should_panic]
        fn panics_if_the_page_has_no_title_header() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("Filler".to_string()),
                Ok("# Cell ABC".to_string()),
                Ok("# This is not a title header!".to_string()),
            ];
            let mut iterator = md_lines.into_iter().peekable();

            iterate_until_page_starts(&mut iterator);
        }

        #[test]
        #[should_panic]
        fn panics_if_the_file_ends_before_a_pages_title_is_found() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("Filler".to_string()),
                Ok("# Cell ABC".to_string()),
                Ok("The file is about to end with no title in sight".to_string()),
            ];
            let mut iterator = md_lines.into_iter().peekable();

            iterate_until_page_starts(&mut iterator);
        }
    }

    mod next_line_starts_with {
        use crate::document::html::{NextLineMatch, next_line_starts_with};

        #[test]
        fn determines_that_the_iterator_is_at_eof() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![];
            let mut iterator = md_lines.into_iter().peekable();
            let prefix = "eof, doesn't matter";

            assert_eq!(
                NextLineMatch::Eof,
                next_line_starts_with(&mut iterator, prefix)
            );
        }

        #[test]
        fn determines_that_the_iterator_is_at_a_match_without_advancing_the_iterator() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![Ok("Match found".to_string())];
            let mut iterator = md_lines.into_iter().peekable();
            let prefix = "Match";

            assert_eq!(
                NextLineMatch::Match,
                next_line_starts_with(&mut iterator, prefix)
            );
            assert_eq!("Match found", iterator.next().unwrap().unwrap().as_str());
        }

        #[test]
        fn determines_that_the_iterator_is_not_at_a_match_without_advancing_the_iterator() {
            let md_lines: Vec<Result<String, std::io::Error>> =
                vec![Ok("No Match found".to_string())];
            let mut iterator = md_lines.into_iter().peekable();
            let prefix = "Match";

            assert_eq!(
                NextLineMatch::NoMatch,
                next_line_starts_with(&mut iterator, prefix)
            );
            assert_eq!("No Match found", iterator.next().unwrap().unwrap().as_str());
        }
    }

    mod iterate_until_header_or_eof {
        use crate::document::html::iterate_until_header_or_eof;

        #[test]
        fn iterates_until_a_header_is_reached() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("First line".to_string()),
                Ok("Second line".to_string()),
                Ok("# Found a header!".to_string()),
            ];
            let mut iterator = md_lines.into_iter().peekable();

            let lines = iterate_until_header_or_eof(&mut iterator);

            let expected_lines = vec!["First line".to_string(), "Second line".to_string()];
            assert_eq!(expected_lines, lines);
            assert_eq!(
                "# Found a header!",
                iterator.next().unwrap().unwrap().as_str()
            )
        }

        #[test]
        fn iterates_until_eof_is_reached() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("First line".to_string()),
                Ok("Second line".to_string()),
                Ok("File is about to end".to_string()),
            ];
            let mut iterator = md_lines.into_iter().peekable();

            let lines = iterate_until_header_or_eof(&mut iterator);

            let expected_lines = vec![
                "First line".to_string(),
                "Second line".to_string(),
                "File is about to end".to_string(),
            ];
            assert_eq!(expected_lines, lines);
        }

        #[test]
        fn stops_immediately_if_the_next_line_is_already_a_header() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![
                Ok("# Found a header!".to_string()),
                Ok("First line".to_string()),
                Ok("Second line".to_string()),
            ];
            let mut iterator = md_lines.into_iter().peekable();

            let lines = iterate_until_header_or_eof(&mut iterator);

            let expected_lines: Vec<String> = vec![];
            assert_eq!(expected_lines, lines);
            assert_eq!(
                "# Found a header!",
                iterator.next().unwrap().unwrap().as_str()
            )
        }

        #[test]
        fn stops_immediately_if_the_next_line_is_already_eof() {
            let md_lines: Vec<Result<String, std::io::Error>> = vec![];
            let mut iterator = md_lines.into_iter().peekable();

            let lines = iterate_until_header_or_eof(&mut iterator);

            let expected_lines: Vec<String> = vec![];
            assert_eq!(expected_lines, lines);
        }
    }
}
