use crate::PixelPoint;
use crate::config::TemplateConfig;
use crate::document::Template::*;
use crate::document::Token::*;
use crate::geometry::CellMap;
use crate::image_handling::IMAGE_SUBDIRECTORY;
use crate::image_handling::map_cutout::CutoutImage;
use crate::image_handling::table_of_contents::TableOfContentsMapImage;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;

pub mod markdown_content;

pub mod html;
/// The templates that are used to generate the final html document.
enum TemplateFiles {
    Styles,
    MapDocs,
    TableOfContents,
    CellPage,
    ExtraPage,
    LeftColumn,
    RightColumn,
    ExtraRightColumn,
    Section,
    HighlightedSection,
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
            TemplateFiles::MapDocs => {
                Box::new(include_bytes!("document/templates/map_docs_template.html").lines())
            }
            TemplateFiles::TableOfContents => Box::new(
                include_bytes!("document/templates/table_of_contents_template.html").lines(),
            ),
            TemplateFiles::CellPage => {
                Box::new(include_bytes!("document/templates/cell_page_template.html").lines())
            }
            TemplateFiles::ExtraPage => {
                Box::new(include_bytes!("document/templates/extra_page_template.html").lines())
            }
            TemplateFiles::LeftColumn => {
                Box::new(include_bytes!("document/templates/left_column_template.html").lines())
            }
            TemplateFiles::RightColumn => {
                Box::new(include_bytes!("document/templates/right_column_template.html").lines())
            }
            TemplateFiles::ExtraRightColumn => Box::new(
                include_bytes!("document/templates/extra_right_column_template.html").lines(),
            ),
            TemplateFiles::Section => {
                Box::new(include_bytes!("document/templates/section_template.html").lines())
            }
            TemplateFiles::HighlightedSection => Box::new(
                include_bytes!("document/templates/highlighted-section-template.html").lines(),
            ),
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
            TemplateFiles::LeftColumn => &config.left_column_html_override,
            TemplateFiles::RightColumn => &config.right_column_html_override,
            TemplateFiles::ExtraRightColumn => &config.extra_right_column_html_override,
            TemplateFiles::Section => &config.section_html_override,
            TemplateFiles::HighlightedSection => &config.highlighted_section_html_override,
        }
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
    ViewBox,
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
            SectionContents => "SECTION_CONTENTS",
            ViewBox => "VIEW_BOX",
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
            "VIEW_BOX" => Ok(ViewBox),
            _ => Err(()),
        }
    }
}

/// In-line templates inside of the template files. Unlike (tokens)[Token], Templates can contain
/// multiple lines, including further tokens and templates.
///
/// Templates are wrapped by double brackets;
/// e.g. \[\[CELL_PAGES]] maps to [CellPages](Template/CellPages).
#[derive(Debug, PartialEq)]
pub enum Template {
    TableOfContents,
    TableOfContentsPolygonLinks,
    ZoomedInMapPolygonLinks,
    CellPages,
    LeftColumn,
    RightColumn,
    ExtraRightColumn,
    Sections,
}

impl FromStr for Template {
    type Err = ();

    fn from_str(enum_name: &str) -> Result<Self, Self::Err> {
        match enum_name {
            "TABLE_OF_CONTENTS" => Ok(TableOfContents),
            "TABLE_OF_CONTENTS_POLYGON_LINKS" => Ok(TableOfContentsPolygonLinks),
            "ZOOMED_IN_MAP_POLYGON_LINKS" => Ok(ZoomedInMapPolygonLinks),
            "CELL_PAGES" => Ok(CellPages),
            "LEFT_COLUMN" => Ok(LeftColumn),
            "RIGHT_COLUMN" => Ok(RightColumn),
            "SECTIONS" => Ok(Sections),
            "EXTRA_RIGHT_COLUMN" => Ok(ExtraRightColumn),
            _ => Err(()),
        }
    }
}

impl Display for Template {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let as_string: &str = match self {
            TableOfContents => "TABLE_OF_CONTENTS",
            TableOfContentsPolygonLinks => "TABLE_OF_CONTENTS_POLYGON_LINKS",
            ZoomedInMapPolygonLinks => "ZOOMED_IN_MAP_POLYGON_LINKS",
            CellPages => "CELL_PAGES",
            LeftColumn => "LEFT_COLUMN",
            RightColumn => "RIGHT_COLUMN",
            Sections => "SECTIONS",
            ExtraRightColumn => "EXTRA_RIGHT_COLUMN",
        };
        f.write_str(as_string)
    }
}

/// If the [string] contains the [token], replace it by [replace_by]. Otherwise returns
/// the original string without allocating a new one.
fn replace_if_contains(string: String, token: &Token, replace_by: &str) -> String {
    //This squiggly mess is equivalent to {{token}} after the character is escaped
    let token_string = format!("{{{{{}}}}}", token.to_string());
    let token_pattern = token_string.as_str();
    if !string.contains(token_pattern) {
        return string;
    }
    string.replace(token_pattern, replace_by)
}

/// Token values that have a value depending on how far along the document generation is.
struct DocumentContext<'document> {
    target_directory: &'document PathBuf,
    config: &'document Option<TemplateConfig>,
    cell_map: &'document CellMap,
    table_of_contents_images: &'document Vec<TableOfContentsMapImage>,
    cutout_images_by_coordinate: HashMap<String, &'document CutoutImage>,

    replace_tokens_regex: Regex,
    identify_template_regex: Regex,

    document_title: &'document String,
    cutout_image_size: PixelPoint,

    current_table_of_contents_image: Option<&'document TableOfContentsMapImage>,

    current_image_path: Option<String>,
    current_image_size: Option<PixelPoint>,
    current_image_width_height_css: Option<String>,
    current_svg_viewbox: Option<String>,

    current_coordinate: Option<String>,
    current_page_title: Option<String>,
    current_page_description: Option<String>,

    current_section_title: Option<String>,
    current_section_contents: Option<String>,
}
impl<'document> DocumentContext<'document> {
    fn new(
        document_title: &'document String,
        cutout_image_size: PixelPoint,
        target_directory: &'document PathBuf,
        config: &'document Option<TemplateConfig>,
        cell_map: &'document CellMap,
        table_of_contents_images: &'document Vec<TableOfContentsMapImage>,
        cutout_images_by_coordinate: HashMap<String, &'document CutoutImage>,
    ) -> DocumentContext<'document> {
        let replace_tokens_regex = Regex::new(r"\{\{(?<token>.*?)}}").unwrap();
        let identify_template_regex =
            Regex::new(r"(?<whitespace>\s*)\[\[(?<template>.*?)]]").unwrap();
        DocumentContext {
            target_directory,
            config,
            cell_map,
            table_of_contents_images,
            cutout_images_by_coordinate,
            replace_tokens_regex,
            identify_template_regex,
            document_title,
            cutout_image_size,
            current_image_path: None,
            current_image_size: None,
            current_image_width_height_css: None,
            current_svg_viewbox: None,
            current_coordinate: None,
            current_page_title: None,
            current_page_description: None,
            current_section_title: None,
            current_section_contents: None,
            current_table_of_contents_image: None,
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

    fn set_current_table_of_contents_image(&mut self, image: &'document TableOfContentsMapImage) {
        self.current_image_path =
            Some(format!("{}/{}", IMAGE_SUBDIRECTORY, image.filename.clone()));
        self.current_image_size = Some(image.size);
        self.current_svg_viewbox = Some(format!("0 0 {} {}", image.size.x, image.size.y));
        self.current_table_of_contents_image = Some(image);
        self.current_image_width_height_css = Some(format!(
            "width:{}px; height:{}px;",
            image.size.x, image.size.y
        ))
    }

    fn start_new_cell_page(&mut self, coordinate: String, title: String, description: String) {
        let mut image_size = self.cutout_image_size;
        if let Some(cutout_image) = self.cutout_images_by_coordinate.get(&coordinate) {
            if cutout_image.image_size.x < image_size.x || cutout_image.image_size.y < image_size.y
            {
                image_size = cutout_image.image_size;
            }
        };
        self.current_image_path = Some(format!("{}/{}.png", IMAGE_SUBDIRECTORY, coordinate));
        self.current_coordinate = Some(coordinate);
        self.current_image_size = Some(self.cutout_image_size);
        self.current_svg_viewbox = Some(format!("0 0 {} {}", image_size.x, image_size.y));

        self.current_image_width_height_css = Some(format!(
            "width:{}px; height:{}px;",
            image_size.x, image_size.y
        ));

        self.current_page_title = Some(title);
        self.current_page_description = Some(description);

        self.current_section_title = None;
        self.current_section_contents = None;
    }

    fn start_new_extra_page(
        &mut self,
        coordinate: Option<String>,
        title: String,
        description: String,
    ) {
        self.current_image_path = None;
        self.current_coordinate = coordinate;
        self.current_image_size = Some(self.cutout_image_size);
        self.current_svg_viewbox = Some(format!(
            "0 0 {} {}",
            self.cutout_image_size.x, self.cutout_image_size.y
        ));
        self.current_image_width_height_css = Some(format!(
            "width:{}px; height:{}px;",
            self.cutout_image_size.x, self.cutout_image_size.y
        ));

        self.current_page_title = Some(title);
        self.current_page_description = Some(description);

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
            Coordinate => self.current_coordinate.as_ref(),
            PageTitle => self.current_page_title.as_ref(),
            PageDescription => self.current_page_description.as_ref(),
            SectionTitle => self.current_section_title.as_ref(),
            SectionContents => self.current_section_contents.as_ref(),
            ViewBox => self.current_svg_viewbox.as_ref(),
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

#[derive(Debug, PartialEq)]
struct TemplateMatch {
    leading_whitespace: String,
    template: Template,
}

#[cfg(test)]
pub mod test {
    pub(crate) mod fixtures;

    mod context {
        mod replace_tokens {
            use crate::PixelPoint;
            use crate::document::DocumentContext;
            use crate::geometry::hexagons::fixtures::{FourByFour, ToSnapshot};
            use crate::image_handling::table_of_contents::TableOfContentsMapImage;
            use std::collections::HashMap;
            use std::path::PathBuf;

            #[test]
            fn replaces_all_tokens_in_the_string_with_values_from_the_context() {
                let all_tokens = r"
1. {{DOCUMENT_TITLE}} 2. {{MAP_IMAGE}}
3. {{WIDTH_HEIGHT}}
4. {{COORDINATE}}
5. {{PAGE_TITLE}}
6. {{PAGE_DESCRIPTION}}
7. {{SECTION_TITLE}}
8. {{SECTION_CONTENTS}}
9. {{VIEW_BOX}}
                ";
                let expected = r"
1. abc 2. generated-images/jkl.png
3. width:100px; height:200px;
4. jkl
5. mno
6. pqr
7. stu
8. vwx
9. 0 0 100 200
                ";
                let title = "abc".to_string();
                let target = PathBuf::from("target");
                let cell_map = FourByFour::Standardized.to_snapshot().cell_map;
                let toc_images = vec![];
                let cutout_images = HashMap::new();
                let mut context = DocumentContext::new(
                    &title,
                    PixelPoint { x: 100, y: 200 },
                    &target,
                    &None,
                    &cell_map,
                    &toc_images,
                    cutout_images,
                );
                let toc_image = TableOfContentsMapImage {
                    filename: "def".to_string(),
                    size: PixelPoint { x: 150, y: 250 },
                    offset: PixelPoint { x: 0, y: 0 },
                    coordinates_contained: Default::default(),
                };
                context.set_current_table_of_contents_image(&toc_image);
                context.start_new_cell_page(
                    "jkl".to_string(),
                    "mno".to_string(),
                    "pqr".to_string(),
                );
                context.start_new_section("stu".to_string(), "vwx".to_string());

                assert_eq!(expected, context.replace_tokens(all_tokens));
            }

            #[test]
            fn returns_the_unchanged_string_if_no_tokens_match() {
                let title = "abc".to_string();
                let target = PathBuf::from("target");
                let cell_map = FourByFour::Standardized.to_snapshot().cell_map;
                let toc_images = vec![];
                let cutout_images = HashMap::new();
                let context = DocumentContext::new(
                    &title,
                    PixelPoint { x: 100, y: 200 },
                    &target,
                    &None,
                    &cell_map,
                    &toc_images,
                    cutout_images,
                );
                let line_without_tokens = "Just vibing in a dungeon";
                assert_eq!(
                    line_without_tokens.to_string(),
                    context.replace_tokens(line_without_tokens)
                );
            }

            #[test]
            #[should_panic]
            fn panics_if_the_token_is_unknown() {
                let title = "abc".to_string();
                let target = PathBuf::from("target");
                let cell_map = FourByFour::Standardized.to_snapshot().cell_map;
                let toc_images = vec![];
                let cutout_images = HashMap::new();
                let context = DocumentContext::new(
                    &title,
                    PixelPoint { x: 100, y: 200 },
                    &target,
                    &None,
                    &cell_map,
                    &toc_images,
                    cutout_images,
                );
                let line_without_tokens = "Just vibing in a dungeon when suddenly {{PANIC}}";
                context.replace_tokens(line_without_tokens);
            }
        }
    }
    mod template_matches {
        use crate::PixelPoint;
        use crate::document::{DocumentContext, Template, TemplateMatch};
        use crate::geometry::hexagons::fixtures::{FourByFour, ToSnapshot};
        use std::collections::HashMap;
        use std::path::PathBuf;

        #[test]
        fn identifies_all_templates_and_their_whitespace_prefixes() {
            let all_templates = r"
1. [[TABLE_OF_CONTENTS]]
2.  [[TABLE_OF_CONTENTS_POLYGON_LINKS]]
3.   [[ZOOMED_IN_MAP_POLYGON_LINKS]]
4.    [[CELL_PAGES]]
5.     [[LEFT_COLUMN]]
6.      [[RIGHT_COLUMN]]
7.       [[SECTIONS]]
8.        [[EXTRA_RIGHT_COLUMN]]

                ";
            let expected = vec![
                TemplateMatch {
                    leading_whitespace: " ".to_string(),
                    template: Template::TableOfContents,
                },
                TemplateMatch {
                    leading_whitespace: "  ".to_string(),
                    template: Template::TableOfContentsPolygonLinks,
                },
                TemplateMatch {
                    leading_whitespace: "   ".to_string(),
                    template: Template::ZoomedInMapPolygonLinks,
                },
                TemplateMatch {
                    leading_whitespace: "    ".to_string(),
                    template: Template::CellPages,
                },
                TemplateMatch {
                    leading_whitespace: "     ".to_string(),
                    template: Template::LeftColumn,
                },
                TemplateMatch {
                    leading_whitespace: "      ".to_string(),
                    template: Template::RightColumn,
                },
                TemplateMatch {
                    leading_whitespace: "       ".to_string(),
                    template: Template::Sections,
                },
                TemplateMatch {
                    leading_whitespace: "        ".to_string(),
                    template: Template::ExtraRightColumn,
                },
            ];
            let title = "abc".to_string();
            let target = PathBuf::from("target");
            let cell_map = FourByFour::Standardized.to_snapshot().cell_map;
            let toc_images = vec![];
            let cutout_images = HashMap::new();
            let context = DocumentContext::new(
                &title,
                PixelPoint { x: 100, y: 200 },
                &target,
                &None,
                &cell_map,
                &toc_images,
                cutout_images,
            );
            assert_eq!(expected, context.template_matches(all_templates));
        }

        #[test]
        fn returns_an_empty_vector_if_no_templates_were_found() {
            let title = "abc".to_string();
            let target = PathBuf::from("target");
            let cell_map = FourByFour::Standardized.to_snapshot().cell_map;
            let toc_images = vec![];
            let cutout_images = HashMap::new();
            let context = DocumentContext::new(
                &title,
                PixelPoint { x: 100, y: 200 },
                &target,
                &None,
                &cell_map,
                &toc_images,
                cutout_images,
            );
            let line_without_templates = "Just vibing in a dungeon";
            assert_eq!(0, context.template_matches(line_without_templates).len());
        }

        #[test]
        #[should_panic]
        fn panics_if_the_template_is_unknown() {
            let title = "abc".to_string();
            let target = PathBuf::from("target");
            let cell_map = FourByFour::Standardized.to_snapshot().cell_map;
            let toc_images = vec![];
            let cutout_images = HashMap::new();
            let context = DocumentContext::new(
                &title,
                PixelPoint { x: 100, y: 200 },
                &target,
                &None,
                &cell_map,
                &toc_images,
                cutout_images,
            );
            let line_without_tokens = "Just vibing in a dungeon when suddenly [[PANIC]]";
            context.template_matches(line_without_tokens);
        }
    }
}
