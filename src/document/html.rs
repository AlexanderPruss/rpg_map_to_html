use std::collections::HashSet;
use pulldown_cmark::{Event, Parser, Tag, TextMergeStream};
use crate::{PixelBox, PixelPoint};
use crate::config::TemplateConfig;
use crate::document::markdown_content::{add_md_content_for_missing_cells, get_markdown_content_read_file};
use crate::geometry::CellMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use crate::document::{replace_if_contains, RegexHelper};
use crate::image_handling::table_of_contents::TableOfContentsMapImage;

mod markdown_to_html;

static HTML_DOC_FILENAME: &str = "rpg_map_doc.html";
static STYLES_FILENAME: &str = "styles.css";

pub static CUTOUT_WIDTH_HEIGHT_TOKEN : &str = "{{CUTOUT_WIDTH_HEIGHT}}";
pub static WIDTH_HEIGHT_TOKEN : &str = "{{WIDTH_HEIGHT}}";
pub static TITLE_TOKEN : &str = "{{TITLE}}";
pub static TABLE_OF_CONTENTS_TOKEN : &str = "{{TABLE_OF_CONTENTS_TEMPLATES}}";
pub static CELL_PAGE_TOKEN : &str = "{{CELL_PAGE_TEMPLATES}}";
pub static MAP_IMAGE_TOKEN : &str = "{{MAP_IMAGE}}";
pub static SVG_LINKS_TOKEN : &str = "{{SVG_POLYGON_LINKS}}";

pub static PAGE_ID_PREFIX : &str = "page-";

/// The templates that are used to generate the final html document.
enum TemplateFiles {
    Styles,
    MapDocs,
    TableOfContents,
    CellPage,
    ExtraPage,
}

impl TemplateFiles {

    fn get_template_lines(&self, template_config: &Option<TemplateConfig>) -> Box<dyn Iterator<Item = std::io::Result<String>>> {
        if let Some(override_path) = self.get_override_path(template_config) {
            let override_file = File::open(override_path).unwrap();
            return Box::new(BufReader::new(override_file).lines());
        }
        match self {
            TemplateFiles::Styles => Box::new(include_bytes!("templates/styles_template.css").lines()),
            TemplateFiles::MapDocs => Box::new(include_bytes!("templates/table_of_contents_template.html").lines()),
            TemplateFiles::TableOfContents => Box::new(include_bytes!("templates/table_of_contents_template.html").lines()),
            TemplateFiles::CellPage => Box::new(include_bytes!("templates/cell_page_template.html").lines()),
            TemplateFiles::ExtraPage => Box::new(include_bytes!("templates/extra_page_template.html").lines())
        }
    }

    fn get_override_path<'config>(&self,
                             template_config: &'config Option<TemplateConfig>) -> &'config Option<PathBuf>{
        if template_config.is_none() {
            return &None;
        }
        let config = template_config.as_ref().unwrap();
        match self {
            TemplateFiles::Styles => &config.styles_override,
            TemplateFiles::MapDocs => &config.document_html_override,
            TemplateFiles::TableOfContents => &config.table_of_contents_html_override,
            TemplateFiles::CellPage => &config.cell_page_html_override,
            TemplateFiles::ExtraPage => &config.extra_cell_page_html_override
        }
    }
}

/// Generates or appends to any existing markdown content found. Then uses this markdown content
/// to generate the final html document.
pub fn write_html_doc(
    target_directory: &PathBuf,
    title: String,
    cell_map: &CellMap,
    cutout_width_height: PixelPoint,
    config: Option<TemplateConfig>,
) {
    let regex_helper = RegexHelper::new();
    let mut ordered_cells: Vec<&String> = cell_map
        .cells_by_coordinate
        .iter()
        .map(|(coordinate, _cell)| coordinate)
        .collect();
    ordered_cells.sort();
    add_md_content_for_missing_cells(target_directory, ordered_cells, &regex_helper);

    generate_styles(target_directory, &config, &cutout_width_height);
    generate_html_doc(
        target_directory,
        title,
        &config,
        cell_map,
        cutout_width_height,
    );
}

fn generate_styles(target_directory: &PathBuf, config : &Option<TemplateConfig>, cutout_width_height: &PixelPoint) {
    let mut map_docs_path = PathBuf::from(target_directory);
    map_docs_path.push(STYLES_FILENAME);
    let mut result_writer = BufWriter::new(File::create(map_docs_path).unwrap());
    let width_height_css = format!("width:{}px; height: {}px;", cutout_width_height.x, cutout_width_height.y);
    for line in TemplateFiles::Styles
        .get_template_lines(config)
        .map_while(Result::ok)
    {
        let line = replace_if_contains(line, CUTOUT_WIDTH_HEIGHT_TOKEN, &width_height_css);
        result_writer.write(line.as_bytes()).unwrap();
        result_writer.write("\n".as_bytes()).unwrap();
    }
    result_writer.flush().unwrap();
}

fn generate_html_doc(
    target_directory: &PathBuf,
    title: String,
    config: &Option<TemplateConfig>,
    cell_map: &CellMap,
    cutout_width_height: PixelPoint,
) {
    let mut map_docs_path = PathBuf::from(target_directory);
    map_docs_path.push(HTML_DOC_FILENAME);
    let mut result_writer = BufWriter::new(File::create(map_docs_path).unwrap());

    for line in TemplateFiles::MapDocs
        .get_template_lines(config)
        .map_while(Result::ok)
    {
        let templates_filled = fill_map_docs_templates(target_directory, &line, &mut result_writer, config);
        if templates_filled {
            continue;
        }
        let line = replace_if_contains(line, TITLE_TOKEN, &title);
        result_writer.write(line.as_bytes()).unwrap();
        result_writer.write("\n".as_bytes()).unwrap();
    }
    result_writer.flush().unwrap();
}

fn fill_map_docs_templates(
    target_directory: &PathBuf,
    line: &String,
    writer: &mut BufWriter<File>,
    config: &Option<TemplateConfig>,
) -> bool {
    if line.as_str() == TABLE_OF_CONTENTS_TOKEN{
        //TODO: fill_table_of_contents_templates(writer, config);
        return true;
    }
    if line.as_str() == CELL_PAGE_TOKEN {
        fill_cell_page_templates(target_directory, writer, config);
        return true;
    }
    false
}

fn fill_table_of_contents_templates(writer: &mut BufWriter<File>, cell_map: &CellMap, config: &Option<TemplateConfig>, table_of_contents_images :Vec<TableOfContentsMapImage> ) {
    //TODO: Pass in prefix as well
    for table_of_contents_image in table_of_contents_images {
        let width_height_css = format!("width:{}px; height: {}px;", table_of_contents_image.size.x, table_of_contents_image.size.y);
        for line in TemplateFiles::TableOfContents
            .get_template_lines(config)
            .map_while(Result::ok){
            if line.contains(SVG_LINKS_TOKEN) {
                generate_svg_links(writer, cell_map, "\t\t", table_of_contents_image.size, table_of_contents_image.offset, &table_of_contents_image.coordinates_contained);
                continue;
            }
            let line = replace_if_contains(line, MAP_IMAGE_TOKEN, table_of_contents_image.filename.as_str());
            let line = replace_if_contains(line, WIDTH_HEIGHT_TOKEN, width_height_css.as_str());
            writer.write(line.as_bytes()).unwrap();
            writer.write("\n".as_bytes()).unwrap();
        }
    }
}

fn fill_cell_page_templates(target_directory: &PathBuf, writer: &mut BufWriter<File>, config: &Option<TemplateConfig>) {
    let markdown_content = get_markdown_content_read_file(target_directory);
    // If the line is one of our special lines, create the html element we need.
    for md_line in BufReader::new(markdown_content).lines().map_while(Result::ok) {
        let md_iterator = TextMergeStream::new(Parser::new(&md_line.as_str()));
        for md_event in md_iterator {
            match md_event{
                Event::Start(tag) => {}
                Event::End(end_tag) => {}
                _ => {
                    md_event;
                }
            }
        }
    }
    todo!()
}

fn generate_svg_links(writer: &mut BufWriter<File>, cell_map: &CellMap, tab_prefix: &str, image_size: PixelPoint, offset: PixelPoint, coordinates_contained: &HashSet<String>) {
    let svg_tag = format!("{}<svg class=\"link-overlay cutout-overlay\" viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\">\n", tab_prefix, image_size.x, image_size.y);
    writer.write(svg_tag.as_bytes()).unwrap();
    let close_svg_tag = format!("{}</svg>\n", tab_prefix);
    for coordinate in coordinates_contained {
        let cell = cell_map.cells_by_coordinate.get(coordinate);
        if cell.is_none() {
            eprintln!("Expected a cell for coordinate {} in the table of contents, but no such cell was found in the cell map", coordinate);
            continue;
        }
        let cell = cell.unwrap();
        let a_tag = format!("{}\t<a href=\"{}{}\">\n", tab_prefix, PAGE_ID_PREFIX, coordinate);
        let close_a_tag = format!("{}\t</a>\n", tab_prefix);
        writer.write(a_tag.as_bytes()).unwrap();
        let link_box :PixelBox = cell.bounding_polygon.get_inscribed_rectangle();
        let top_left = link_box.top_left_corner - offset;
        let bottom_right = link_box.bottom_right_corner - offset;
        let width = bottom_right.x - top_left.x;
        let height = bottom_right.y - top_left.y;
        let rect_tag = format!("{}\t\t<rect class=\"cell-link\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\">\n", tab_prefix,
        top_left.x, top_left.y, width, height);
        writer.write(rect_tag.as_bytes()).unwrap();
        writer.write(close_a_tag.as_bytes()).unwrap();
    }
    writer.write(close_svg_tag.as_bytes()).unwrap();
}