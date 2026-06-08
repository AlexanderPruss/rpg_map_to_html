use crate::PixelPoint;
use crate::geometry::Geometry;
use image::Rgba;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
pub struct Config {
    /// New files - images and html - will be saved in a child directory of [target_directory].
    pub target_directory: String,
    pub map_image: MapImageConfig,
    pub map_cutout: Option<MapCutoutConfig>,
    pub template: Option<TemplateConfig>,
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
    pub skip_empty_cells: Option<SkipEmptyCellsConfig>,
}

/// The program can try to automatically determine which cells have no content.
#[derive(Deserialize, Debug)]

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
#[derive(Deserialize, Debug)]
pub struct MapCutoutConfig {
    /// The size, in pixels, of the generated map cutouts on the cell pages.
    ///
    /// Defaults to 325x340.
    pub zoomed_in_map_image_size: Option<PixelPoint>,
    /// Points on the edge of the map look nicer if they're at least somewhat centered, so the map
    /// is padded out to a minimum. This does nothing if the map's margin is already larger.
    ///
    /// Defaults to 50x50.
    pub minimum_map_margin: Option<PixelPoint>,
    /// The cell owning the cutout is outlined to make it clearer which cell is being documented.
    ///
    /// Defaults to yellow.
    pub cell_outline_color: Option<[u8; 4]>, //TODO: Maybe also thickness of the outline? Try it out first
}

/// Allows customizing the generated map file by replacing the templates used.
#[derive(Deserialize, Debug)]
pub struct TemplateConfig {
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
    pub cell_page_html_override: Option<PathBuf>,
    /// The size, in pixels, of the generated map cutouts on the cell pages.
    ///
    /// Defaults to 325x340.
    pub zoomed_in_map_image_size: Option<PixelPoint>, //TODO: But also - color, thickness of the outline, buffer size
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
