use crate::PixelPoint;
use crate::config::TemplateConfig;
use crate::document::markdown::add_md_content_for_missing_cells;
use crate::geometry::CellMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

static HTML_DOC_FILENAME: &str = "rpg_map_doc.html";
static STYLES_FILENAME: &str = "styles.css";

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
    let mut ordered_cells: Vec<&String> = cell_map
        .cells_by_coordinate
        .iter()
        .map(|(coordinate, _cell)| coordinate)
        .collect();
    ordered_cells.sort();
    add_md_content_for_missing_cells(target_directory, ordered_cells);

    generate_styles(target_directory, &cutout_width_height);
    generate_html_doc(
        target_directory,
        title,
        cell_map,
        cutout_width_height,
        config,
    );
}

fn generate_styles(target_directory: &PathBuf, cutout_width_height: &PixelPoint) {
    todo!()
}

fn generate_html_doc(
    target_directory: &PathBuf,
    title: String,
    cell_map: &CellMap,
    cutout_width_height: PixelPoint,
    config: Option<TemplateConfig>,
) {
    let mut map_docs_path = PathBuf::from(target_directory);
    map_docs_path.push(HTML_DOC_FILENAME);
    let mut result_writer = BufWriter::new(File::create(map_docs_path).unwrap());

    for line in TemplateFiles::MapDocs
        .get_template_lines(&config)
        .map_while(Result::ok)
    {
        let templates_filled = fill_map_docs_templates(&line, &mut result_writer, &config);
        if templates_filled {
            continue;
        }
        let line = line.replace("{{TITLE}}", &title);
        result_writer.write(line.as_bytes()).unwrap();
        result_writer.write("\n".as_bytes()).unwrap();
    }
}

fn fill_map_docs_templates(
    line: &String,
    writer: &mut BufWriter<File>,
    config: &Option<TemplateConfig>,
) -> bool {
    if line.as_str() == "{{TABLE_OF_CONTENTS_TEMPLATES}}" {
        fill_table_of_contents_templates(writer, config);
        return true;
    }
    if line.as_str() == "{{CELL_PAGE_TEMPLATES}}" {
        fill_cell_page_templates(writer, config);
        return true;
    }
    false
}

fn fill_table_of_contents_templates(writer: &mut BufWriter<File>, config: &Option<TemplateConfig>) {
    todo!()
}

fn fill_cell_page_templates(writer: &mut BufWriter<File>, config: &Option<TemplateConfig>) {
    todo!()
}