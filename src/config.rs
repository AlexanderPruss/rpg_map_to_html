use crate::geometry::Geometry;
use crate::{PixelPoint, read_input_with_default, read_input_yes_no_with_default, geometry};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    /// Generated files - images and html - will be saved in a child directory of [target_directory].
    pub target_directory: PathBuf,
    pub title: String,
    pub map_image: MapImageConfig,
    /// Describes the structure of the map.
    pub geometry: Geometry,
    pub image_handling_config: Option<ImageHandlingConfig>,
    pub template: Option<TemplateConfig>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MapImageConfig {
    /// The file containing the map. ex: sweetestMap.png
    pub image_file: PathBuf,
    /// The contentless margin surrounding the map. Hexographer's default export has no margin.
    pub image_margins: Option<PixelPoint>,
    /// Per default, cells that are mostly white around their center are skipped.
    pub skip_empty_cells: Option<SkipEmptyCellsConfig>,
}

/// The program can try to automatically determine which cells have no content.
#[derive(Deserialize, Serialize, Debug)]

pub struct SkipEmptyCellsConfig {
    /// Whether to determine and skip empty cells. Defaults to true.
    pub skipping_enabled: Option<bool>,
    /// The multiplier should be in the range (0, 1.0). It scales the [BoundingPolygon] around the
    /// cell's center point. If the area defined this way is empty, the cell is skipped.
    ///
    /// Defaults to 30%. You may need so scale this up if the middle of filled cells is still blank,
    /// i.e. if your hex map has hexes containing a large white circle.
    pub polygon_multiplier: Option<f32>,
    /// What color is considered empty, as an RGBA value. Defaults to white.
    pub empty_color_rgba: Option<[u8; 4]>,
}

/// Allows customizing how the zoomed-in map cutouts on each cell's page are generated.
#[derive(Deserialize, Serialize, Debug)]
pub struct ImageHandlingConfig {
    /// The size, in pixels, of the generated map cutouts on the cell pages.
    ///
    /// Defaults to 325x340.
    pub zoomed_in_map_image_size: Option<PixelPoint>,
    /// The maximum size of the map images that appear at the start of the generated document in
    /// table of contents-like map that allows you to click and jump to any cell.
    ///
    /// Defaults to 900x1200, which fits an A4 page pretty well.
    pub max_table_of_contents_image_size: Option<PixelPoint>,
    /// Points on the edge of the map look nicer if they're at least somewhat centered, so the map
    /// is padded out to a minimum. This does nothing if the map's margin is already larger.
    ///
    /// Defaults to 50x50.
    pub minimum_map_margin: Option<PixelPoint>,
    /// The cell owning the cutout is outlined to make it clearer which cell is being documented.
    ///
    /// Defaults to red.
    pub cell_outline_color: Option<[u8; 4]>,
}

/// Allows customizing the generated map file by replacing the templates used.
#[derive(Deserialize, Serialize, Debug)]
pub struct TemplateConfig {
    /// A .css file replacing the default styles.
    pub styles_override: Option<PathBuf>,
    /// The overall html document that is rendered, including references to all other templates.
    pub document_html_override: Option<PathBuf>,
    /// An .html file that is rendered at the start of the output document.
    ///
    /// The default introduction is a visual table of contents where all cells are hyperlinked to their
    /// document page.
    pub table_of_contents_html_override: Option<PathBuf>,
    /// An .html file rendered for every cell.
    ///
    /// Defaults to a Dolmenwood-esque two-column description.
    pub cell_page_html_override: Option<PathBuf>,
    /// An .html file rendered for every extra page needed for a cell.
    ///
    /// Defaults to a Dolmenwood-esque two-column description, but without the image.
    pub extra_cell_page_html_override: Option<PathBuf>,
    /// An .html file rendered for every left column on a page.
    ///
    /// Defaults to a section container taking up roughly half a page.
    pub left_column_html_override: Option<PathBuf>,
    /// An .html file rendered for every right column on a page.
    ///
    /// Defaults to the left column, but with a zoomed in cell map.
    pub right_column_html_override: Option<PathBuf>,
    /// An .html file rendered for every right column on an extra page.
    ///
    /// Defaults to the usual right column, but with no cell map.
    pub extra_right_column_html_override: Option<PathBuf>,
    /// An .html file rendered for every section in a column. It is filled by user generated
    /// content from the markdown content file.
    ///
    /// Defaults to a section similar to the ones in the Dolmenwood book.
    pub section_html_override: Option<PathBuf>,
    /// An .html file rendered for every highlighted section in a column. It is filled by user generated
    /// content from the markdown content file.
    ///
    /// "Highlighted" means here that it is set in its own colored box. It draws attention to
    /// individual elements, creatures, treasures, etc.
    ///
    /// Defaults to a highlighted similar to the ones in the Dolmenwood book.
    pub highlighted_section_html_override: Option<PathBuf>,
    /// The size, in pixels, of the generated map cutouts on the cell pages.
    ///
    /// Defaults to 325x340.
    pub zoomed_in_map_image_size: Option<PixelPoint>,
}

/// Reads a [Config] out of the file at [config_path].
pub fn parse_config(config_path: PathBuf) -> Config {
    serde_json::from_reader(BufReader::new(File::open(config_path).unwrap())).unwrap()
}

/// Persists the [config] to the [path] as a pretty-JSON file.
pub fn persist_config(config: &Config, path: &PathBuf) {
    let mut writer = BufWriter::new(File::create(path).unwrap());
    writer
        .write(serde_json::to_string_pretty(config).unwrap().as_bytes())
        .unwrap();
    writer.flush().unwrap();
}

pub fn generate_config_interactively() -> Config {
    println!("Generating a config interactively");
    let mut input = String::new();

    println!("Path to the map image:");
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let image_file = PathBuf::from(input.trim());

    println!("Directory the generated files should be created in: (Default: 'target')");
    let target_directory = PathBuf::from(read_input_with_default(&mut input, "target".to_string()));

    println!("Title of the generated document: (Default: 'Title')");
    let title = read_input_with_default(&mut input, "Title".to_string());

    println!(
        "Does the map have a margin? (Pixels of the image that do not contain map cells.) Y/N (default N)"
    );
    let image_margins = if read_input_yes_no_with_default(&mut input, false) {
        println!("Width of the image margin, in pixels: (Default: 0)");
        let x = read_input_with_default(&mut input, "0".to_string());
        println!("Height of the image margin, in pixels: (Default: 0)");
        let y = read_input_with_default(&mut input, "0".to_string());
        Some(PixelPoint {
            x: x.parse().expect("Could not parse the margin width"),
            y: y.parse().expect("Could not parse the margin height"),
        })
    } else {
        None
    };

    let geometry = geometry::generate_geometry_interactively();
    Config{
        target_directory,
        title,
        map_image: MapImageConfig {
            image_file,
            image_margins,
            skip_empty_cells: None,
        },
        geometry,
        image_handling_config: None,
        template: None,
    }
}
