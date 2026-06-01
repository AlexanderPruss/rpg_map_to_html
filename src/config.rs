use crate::PixelPoint;
use crate::geometry::Geometry;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    //TODO: Figure out what else we need at some point
    /// New files - images and html - will be saved in a child directory of [target_directory].
    pub target_directory: String,
    pub map_image: MapImageConfig,
}

#[derive(Deserialize, Debug)]
pub struct MapImageConfig {
    /// The file containing the map. ex: sweetestMap.png
    pub image_file: String,
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
    /// Defaults to 50%.
    pub polygon_multiplier: f32,
    /// What color is considered empty. Defaults to white.
    pub empty_color: i32,
}

#[derive(Deserialize, Debug)]
pub enum GenerationMode {
    /// (Re)creates a map, discarding the previous one if it already exists
    CreateOverwrite,
    /// Updates the images of an existing map, adding new pages as needed
    Update,
}

pub fn parse_config(args: &[String]) -> Config {
    unimplemented!()
}
