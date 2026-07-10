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
    let table_of_contents_image = context.current_table_of_contents_image;
    if table_of_contents_image.is_none() {
        panic!(
            "Tried to fill a [[{}]] template, but the context had not set a current table of contents image.",
            Template::TableOfContentsPolygonLinks
        )
    }
    let table_of_contents_image = table_of_contents_image.unwrap();
    write_polygon_links(
        &mut reader_writer.writer,
        context.cell_map,
        prefix,
        table_of_contents_image.offset * -1, //TODO: This is silly, standardize it lol
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
    let coordinate = context.current_coordinate.as_ref();
    if coordinate.is_none() {
        panic!(
            "Tried to fill a [[{}]] template, but the context had not set current coordinate",
            Template::ZoomedInMapPolygonLinks
        )
    }
    let coordinate = coordinate.unwrap();
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
    for coordinate in coordinates_to_link {
        let cell = cell_map.cells_by_coordinate.get(coordinate);
        if cell.is_none() {
            panic!(
                "Expected to draw a link for a cell with coordinate {}, but no such cell was found in the cell map",
                coordinate
            );
        }
        let cell = cell.unwrap();
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
    let expect_column_header_line = reader_writer.md_lines.next();
    if expect_column_header_line.is_none() {
        panic!(
            "Tried to fill in a [[{template}]] Template for page {:?}, but the markdown content file was already exhausted.\n\
            You may have deleted the '## Left Column' or '## Right Column' markdown headers from that page; add them back to fix this error.",
            context.current_page_title
        )
    }
    let expect_column_header_line = expect_column_header_line.unwrap().unwrap();
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
    //If multiple lines are found, just take the first one.
    let description = iterate_until_header_or_eof(md_lines)
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .next()
        .unwrap_or_else(|| "".to_string());
    Some(StartOfCellPage {
        coordinate,
        title,
        description,
        extra_page,
    })
}

#[derive(PartialEq)]
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
