use crate::PixelPoint;
use crate::config::TemplateConfig;
use crate::document::Template::*;
use crate::document::Token::*;
use crate::image_handling::table_of_contents::TableOfContentsMapImage;
use regex::{Captures, Regex};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;
use crate::geometry::CellMap;

pub mod markdown_content;

pub mod html;
/// The templates that are used to generate the final html document.
enum TemplateFiles {
    Styles,
    MapDocs,
    TableOfContents,
    CellPage,
    ExtraPage,
}

/// If the [string] contains the [pattern], replace it by [replace_by]. Otherwise returns
/// the original string without allocating a new one.
fn replace_if_contains(string: String, token: &Token, replace_by: &str) -> String{
    let token_string = token.to_string();
    let token_pattern = token_string.as_str();
    if !string.contains(token_pattern) {
        return string;
    }
    string.replace(token_pattern, replace_by)
}


impl TemplateFiles {
    fn get_template_lines(
        &self,
        template_config: &Option<TemplateConfig>,
    ) -> Box<dyn Iterator<Item = std::io::Result<String>>> {
        if let Some(override_path) = self.get_override_path(template_config) {
            let override_file = File::open(override_path).unwrap();
            return Box::new(BufReader::new(override_file).lines());
        }
        match self {
            TemplateFiles::Styles => {
                Box::new(include_bytes!("document/templates/styles_template.css").lines())
            }
            TemplateFiles::MapDocs => Box::new(
                include_bytes!("document/templates/table_of_contents_template.html").lines(),
            ),
            TemplateFiles::TableOfContents => Box::new(
                include_bytes!("document/templates/table_of_contents_template.html").lines(),
            ),
            TemplateFiles::CellPage => {
                Box::new(include_bytes!("document/templates/cell_page_template.html").lines())
            }
            TemplateFiles::ExtraPage => {
                Box::new(include_bytes!("document/templates/extra_page_template.html").lines())
            }
        }
    }

    fn get_override_path<'config>(
        &self,
        template_config: &'config Option<TemplateConfig>,
    ) -> &'config Option<PathBuf> {
        if template_config.is_none() {
            return &None;
        }
        let config = template_config.as_ref().unwrap();
        match self {
            TemplateFiles::Styles => &config.styles_override,
            TemplateFiles::MapDocs => &config.document_html_override,
            TemplateFiles::TableOfContents => &config.table_of_contents_html_override,
            TemplateFiles::CellPage => &config.cell_page_html_override,
            TemplateFiles::ExtraPage => &config.extra_cell_page_html_override,
        }
    }
}

/// Token values that have a value depending on how far along the document generation is.
struct DocumentContext<'document> {
    target_directory: &'document PathBuf,
    config: &'document Option<TemplateConfig>,
    cell_map: &'document CellMap,
    table_of_contents_images: &'document Vec<TableOfContentsMapImage>,
    
    replace_tokens_regex: Regex,
    identify_template_regex: Regex,

    document_title: String,
    cutout_image_size: PixelPoint,

    current_image_path: Option<String>,
    current_image_size: Option<PixelPoint>,
    current_image_width_height_css: Option<String>,

    current_coordinate: Option<&'document String>,
    current_page_title: Option<String>,
    current_page_description: Option<String>,

    current_section_title: Option<String>,
    current_section_contents: Option<String>,
}

impl<'document> DocumentContext<'document> {
    fn new(document_title: String, cutout_image_size: PixelPoint,
           target_directory: &'document PathBuf,
           config: &'document Option<TemplateConfig>,
           cell_map: &'document CellMap,
           table_of_contents_images: &'document Vec<TableOfContentsMapImage>) -> DocumentContext<'document> {
        let replace_tokens_regex = Regex::new(r"\{\{(?<token>.*)}}").unwrap();
        let identify_template_regex =
            Regex::new(r"(?<whitespace>\s*)\[\[(?<template>.*)]]").unwrap();
        DocumentContext {
            target_directory,
            config,
            cell_map,
            table_of_contents_images,
            replace_tokens_regex,
            identify_template_regex,
            document_title,
            cutout_image_size,
            current_image_path: None,
            current_image_size: None,
            current_image_width_height_css: None,
            current_coordinate: None,
            current_page_title: None,
            current_page_description: None,
            current_section_title: None,
            current_section_contents: None,
        }
    }

    fn replace_tokens(&self, line: &str) -> String {
        self.replace_tokens_regex
            .replace_all(line, |caps: &Captures| {
                let token = Token::from_str(&caps["token"]);
                if token.is_err() {
                    panic!("Found unidentified token {}", &caps["token"]);
                }
                self.value_for_token(&token.unwrap())
            })
            .to_string()
    }

    fn template_matches(&self, line: &str) -> Vec<TemplateMatch> {
        self.identify_template_regex
            .captures_iter(line)
            .map(|capture| {
                let (_, [whitespace, template_str]) = capture.extract();
                let template = Template::from_str(template_str);
                if template.is_err() {
                    panic!("Encountered an unrecognized template [[{}]]", template_str);
                }
                TemplateMatch {
                    leading_whitespace: whitespace.to_string(),
                    template: template.unwrap(),
                }
            })
            .collect()
    }

    fn set_current_table_of_contents_image(&mut self, image: &TableOfContentsMapImage) {
        self.current_image_path = Some(image.filename.clone());
        self.current_image_size = Some(image.size);
    }

    fn start_new_page(&mut self, coordinate: &'document String) {
        self.current_image_path = Some(format!("{}.png", coordinate));
        self.current_coordinate = Some(coordinate);
        self.current_image_size = Some(self.cutout_image_size);

        self.current_page_title = None;
        self.current_page_description = None;
        self.current_section_title = None;
        self.current_section_contents = None;
    }

    fn start_new_section(&mut self, section_title: String, section_contents: String) {
        self.current_section_title = Some(section_title);
        self.current_section_contents = Some(section_contents);
    }

    fn value_for_token(&self, token: &Token) -> &String {
        let option = match token {
            DocumentTitle => return &self.document_title,
            MapImage => self.current_image_path.as_ref(),
            WidthHeightCss => self.current_image_width_height_css.as_ref(),
            Coordinate => self.current_coordinate,
            PageTitle => self.current_page_title.as_ref(),
            PageDescription => self.current_page_description.as_ref(),
            SectionTitle => self.current_section_title.as_ref(),
            SectionContents => self.current_section_contents.as_ref(),
        };
        if option.is_none() {
            panic!(
                "Tried to obtain a value for token {}, but none was present in the context.\n\
            This is a templating error, a value is being requested before it is available.",
                token
            );
        }
        option.unwrap()
    }
}

/// In-line tokens inside of the template files. They are replaced by simple strings.
///
/// Tokens are wrapped by double brackets;
/// e.g. {{DOCUMENT_TITLE}} maps to [DocumentTitle](Token/DocumentTitle).
#[derive(Eq, Hash, PartialEq)]
pub enum Token {
    DocumentTitle,
    MapImage,
    WidthHeightCss,
    Coordinate,
    PageTitle,
    PageDescription,
    SectionTitle,
    SectionContents,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let as_string: &str = match self {
            DocumentTitle => "DOCUMENT_TITLE",
            MapImage => "MAP_IMAGE",
            WidthHeightCss => "WIDTH_HEIGHT",
            Coordinate => "COORDINATE",
            PageTitle => "PAGE_TITLE",
            PageDescription => "PAGE_DESCRIPTION",
            SectionTitle => "SECTION_TITLE",
            SectionContents => "SECTION_CONTENTS"
        };
        f.write_str(as_string)
    }
}

impl FromStr for Token {
    type Err = ();

    fn from_str(enum_name: &str) -> Result<Self, Self::Err> {
        match enum_name {
            "DOCUMENT_TITLE" => Ok(DocumentTitle),
            "MAP_IMAGE" => Ok(MapImage),
            "WIDTH_HEIGHT" => Ok(WidthHeightCss),
            "COORDINATE" => Ok(Coordinate),
            "PAGE_TITLE" => Ok(PageTitle),
            "PAGE_DESCRIPTION" => Ok(PageDescription),
            "SECTION_TITLE" => Ok(SectionTitle),
            "SECTION_CONTENTS" => Ok(SectionContents),
            _ => Err(()),
        }
    }
}

/// In-line templates inside of the template files. Unlike (tokens)[Token], Templates can contain
/// multiple lines, including further tokens and templates.
///
/// Templates are wrapped by double brackets;
/// e.g. \[\[CELL_PAGES]] maps to [CellPages](Template/CellPages).
pub enum Template {
    TableOfContents,
    SvgPolygonLinks,
    CellPages,
    LeftColumn,
    RightColumn,
    Sections,
}

impl FromStr for Template {
    type Err = ();

    fn from_str(enum_name: &str) -> Result<Self, Self::Err> {
        match enum_name {
            "TABLE_OF_CONTENTS" => Ok(TableOfContents),
            "SVG_POLYGON_LINKS" => Ok(SvgPolygonLinks),
            "CELL_PAGES" => Ok(CellPages),
            "LEFT_COLUMN" => Ok(LeftColumn),
            "RIGHT_COLUMN" => Ok(RightColumn),
            "SECTIONS" => Ok(Sections),
            _ => Err(()),
        }
    }
}

struct TemplateMatch {
    leading_whitespace: String,
    template: Template,
}

struct RegexHelper {
    replace_tokens_regex: Regex,
    identify_template_regex: Regex,
}

impl RegexHelper {
    fn new() -> RegexHelper {
        let replace_tokens_regex = Regex::new(r"\{\{(?<token>.*)}}").unwrap();
        let identify_template_regex =
            Regex::new(r"(?<whitespace>\s*)\[\[(?<template>.*)]]").unwrap();
        RegexHelper {
            replace_tokens_regex,
            identify_template_regex,
        }
    }

    fn replace_tokens(&self, line: &str, context: &DocumentContext) -> String {
        self.replace_tokens_regex
            .replace_all(line, |caps: &Captures| {
                let token = Token::from_str(&caps["token"]);
                if token.is_err() {
                    panic!("Found unidentified token {}", &caps["token"]);
                }
                context.value_for_token(&token.unwrap())
            })
            .to_string()
    }

    fn template_matches(&self, line: &str) -> Vec<TemplateMatch> {
        self.identify_template_regex
            .captures_iter(line)
            .map(|capture| {
                let (_, [whitespace, template_str]) = capture.extract();
                let template = Template::from_str(template_str);
                if template.is_err() {
                    panic!("Encountered an unrecognized template [[{}]]", template_str);
                }
                TemplateMatch {
                    leading_whitespace: whitespace.to_string(),
                    template: template.unwrap(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
pub mod test {

    pub(crate) mod fixtures;
}
