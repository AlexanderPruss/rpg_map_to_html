use std::path::{Path, PathBuf};
use crate::PixelPoint;
use crate::geometry::Geometry;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    //TODO: Figure out what else we need at some point
    /// New files - images and html - will be saved in a child directory of [target_directory].
    pub target_directory: String,
    pub map_image: MapImageConfig,
    pub output_config: OutputConfig
}

#[derive(Deserialize, Debug)]
pub struct MapImageConfig {
    /// The file containing the map. ex: sweetestMap.png
    pub image_file: PathBuf,
    /// The contentless margin surrounding the map. Hexographer's default export has no margin.
    pub image_margins: Option<PixelPoint>,
    /// Describes the structure of the map.
    pub geometry: Geometry,
    /// Per default, cells that are mostly white around their center are skipped.
    pub skip_empty_cells: Option<SkipEmptyCells>,
}

/// The program can try to automatically determine which cells have no content.
#[derive(Deserialize, Debug)]

pub struct SkipEmptyCells {
    /// Whether to determine and skip empty cells.
    pub skip: bool,
    /// The multiplier should be in the range (0, 1.0). It scales the [BoundingPolygon] around the
    /// cell's center point. If the area defined this way is empty, the cell is skipped.
    ///
    /// Defaults to 30%. You may need so scale this up if the middle of filled cells is still blank,
    /// i.e. if your hex map has hexes containing a large white circle.
    pub polygon_multiplier: f32,
    /// What color is considered empty, as an RGBA value. Defaults to white.
    pub empty_color_rgba: [u8; 4],
}

#[derive(Deserialize, Debug)]
pub struct OutputConfig {
    /// The directory that all generated files will land in.
    pub target_directory: PathBuf,
    /// A .css file replacing the default styles.
    pub styles_override: Option<PathBuf>,
    /// An .html file that is rendered at the start of the output document.
    ///
    /// The default introduction is a two-page map where all cells are hyperlinked to their
    /// document page.
    pub introduction_html_override: Option<PathBuf>,
    /// An .html file rendered for every cell.
    ///
    /// Defaults to a Dolmenwood-esque two-column description.
    ///
    /// Any custom overrides should have a div with the class //TODO not yet specified
    /// which is where the generated zoomed-in map is placed.
    pub cell_page_html_override : Option<PathBuf>,
    /// The size, in pixels, of the generated map cutouts on the cell pages.
    ///
    /// Defaults to 325x340.
    pub zoomed_in_map_image_size: Option<PixelPoint>
}

#[derive(Deserialize, Debug)]
pub enum GenerationMode {
    /// (Re)creates a map, discarding the previous one if it already exists
    CreateOverwrite,
    /// Updates the images of an existing map, adding new pages as needed
    Update,
}

pub fn parse_config(_args: &[String]) -> Config {
    unimplemented!()
}
