use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::string::ToString;
use pulldown_cmark::{Event, Parser, TextMergeStream};
use crate::config::TemplateConfig;
use crate::document::html::TemplateFiles;
use crate::document::markdown_content::get_markdown_content_read_file;

static PAGE_PREFIX: &str = "# Cell ";
static EXTRA_PAGE_PREFIX: &str = "# Extra ";
static PAGE_TITLE_PREFIX: &str = "## Title - ";
static LEFT_COLUMN_PREFIX: &str = "## Left Column";
static RIGHT_COLUMN_PREFIX: &str = "## Right Column";
static SECTION_PREFIX : &str = "### ";
static HIGHLIGHTED_SECTION_PREFIX : &str = "#### ";
enum HeaderType {
    NotAHeader,
    Page,
    ExtraPage,
    PageTitle,
    LeftColumn,
    RightColumn,
    Section,
    HighlightedSection
}

fn fill_cell_page_templates(target_directory: &PathBuf, writer: &mut BufWriter<File>, config: &Option<TemplateConfig>) {
    let markdown_content = get_markdown_content_read_file(target_directory);
    let mut md_lines = BufReader::new(markdown_content).lines();
    loop {
        let line = md_lines.next();
        if line.is_none() {
            break;
        }
        let line = line.unwrap().unwrap();
        let header_type = identify_header_type(&line);
    }

        //if header - 1
        //identify which template we're using, normal or extra. If neither, just panic.
        // say we're writing a cell page.
        //first, need COORDINATE and DESCRIPTION. Once we have them, start writing the template.
        //if we encounter LEFT_COLUMN or RIGHT_COLUMN, get it from MD.

    todo!()
}

struct CurrentPage {
    /// What token we need to continue unpacking this page, if any.
    next_expected_token: Option<Token>,
    template_iterator : Box<dyn Iterator<Item = std::io::Result<String>>>,

    title: String,
    description: String,
    image: Option<String>,
    coordinate: Option<String>,
}

impl CurrentPage {
    fn cell_page_from(target_directory: &PathBuf,writer: &mut BufWriter<File>, title: String, description: String, image: String, coordinate: String, config: &Option<TemplateConfig>) -> CurrentPage{
        CurrentPage{
            next_expected_token: None,
            title,
            description,
            image: Some(image),
            coordinate: Some(coordinate),
            template_iterator: TemplateFiles::CellPage.get_template_lines(config)
        }
    }
    fn extra_page_from(target_directory: &PathBuf, writer: &mut BufWriter<File>,title: String, description: String, config: &Option<TemplateConfig>) -> CurrentPage{
        CurrentPage{
            next_expected_token: None,
            title,
            description,
            image : None,
            coordinate: None,
            template_iterator: TemplateFiles::ExtraPage.get_template_lines(config)
        }
    }
    
    fn process(&mut self) -> ProcessResult {
        //TODO: Rust regex
        unimplemented!()
    }
    
}

enum ProcessResult {
    WaitingForToken,
    Finished
}

enum Token {
    Title,
    Description,
    Coordinate,
    LeftColumn,
    RightColumn,
    SvgLinks,
    SectionTitle, //TODO: Probably these need to be templates as well, as does the svg (un-uglify that code)
    SectionContents
}

fn identify_header_type(line: &String) -> HeaderType {
    if !line.starts_with('#') {
        return HeaderType::NotAHeader;
    }
    if line.starts_with(SECTION_PREFIX) {
        return HeaderType::Section;
    }
    if line.starts_with(HIGHLIGHTED_SECTION_PREFIX) {
        return HeaderType::HighlightedSection;
    }
    if line.starts_with(PAGE_TITLE_PREFIX) {
        return HeaderType::PageTitle;
    }
    if line.starts_with(PAGE_PREFIX) {
        return HeaderType::Page;
    }
    if line.starts_with(EXTRA_PAGE_PREFIX) {
        return HeaderType::ExtraPage;
    }
    if line.starts_with(LEFT_COLUMN_PREFIX) {
        return HeaderType::LeftColumn;
    }
    if line.starts_with(RIGHT_COLUMN_PREFIX) {
        return HeaderType::RightColumn;
    }
    panic!("Did not recognize the header for line:\n{}", line);
}


//fn: get heading level

//         let md_iterator = TextMergeStream::new(Parser::new(&md_line.as_str()));
// for md_event in md_iterator {
//     match md_event{
//         Event::Start(tag) => {}
//         Event::End(end_tag) => {}
//         _ => {
//             md_event;
//         }
//     }
// }