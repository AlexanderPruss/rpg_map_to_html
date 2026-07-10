use std::path::PathBuf;
use crate::config::Config;
use crate::{config, geometry, image_handling};
use crate::geometry::CellMap;
use crate::image_handling::map_cutout::CutoutImage;
use crate::image_handling::table_of_contents::TableOfContentsMapImage;

struct CachedComputedObjects {
    cell_map: CellMap,
    table_of_contents_map_images: Vec<TableOfContentsMapImage>,
    cutout_images: Vec<CutoutImage>
}

/// Attempts to recover computationally intensive objects. These can only be recovered if
///
/// * The config is unchanged from the previous run
/// * The map image is unchanged from the previous run
/// * The cached objects are present
pub fn get_cached_objects() -> Option<CachedComputedObjects> {
    config::parse_config()
    todo!()
}

pub fn persist_cached_objects(target_directory: &PathBuf, config: &Config, cell_map: CellMap,
                                     table_of_contents_images: &Vec<TableOfContentsMapImage>, cutout_images: &Vec<CutoutImage>) {
    config::persist_config(&config);
    geometry::persist_cell_map_as_geometry(target_directory, cell_map);
    image_handling::persist_image_metadata(target_directory, table_of_contents_images, cutout_images);
}