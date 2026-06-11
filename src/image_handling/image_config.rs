use crate::PixelPoint;
use crate::config::{MapCutoutConfig, SkipEmptyCellsConfig};
use image::Rgba;

/// Resolve the input config into a config filled with values, using default values where needed.
pub fn resolve_config(
    skip_config: Option<SkipEmptyCellsConfig>,
    cutout_config: Option<MapCutoutConfig>,
) -> (SkipEmptyCells, MapCutout) {
    let mut skip_empty_cells = SkipEmptyCells {
        skipping_enabled: true,
        polygon_multiplier: 0.3,
        //White
        empty_color_rgba: Rgba::from([255, 255, 255, 255]),
    };
    let mut map_cutout = MapCutout {
        zoomed_in_map_image_size: PixelPoint { x: 325, y: 340 },
        minimum_map_margin: PixelPoint { x: 50, y: 50 },
        //Red
        cell_outline_color: Rgba::from([255, 0, 0, 255]),
    };
    if let Some(input_config) = skip_config {
        let input_color = match input_config.empty_color_rgba {
            None => None,
            Some(rgba_array) => Some(Rgba::from(rgba_array)),
        };
        skip_empty_cells = SkipEmptyCells {
            skipping_enabled: input_config
                .skipping_enabled
                .unwrap_or(skip_empty_cells.skipping_enabled),
            polygon_multiplier: input_config
                .polygon_multiplier
                .unwrap_or(skip_empty_cells.polygon_multiplier),
            empty_color_rgba: input_color.unwrap_or(skip_empty_cells.empty_color_rgba),
        }
    }
    if let Some(input_config) = cutout_config {
        let input_color = match input_config.cell_outline_color {
            None => None,
            Some(rgba_array) => Some(Rgba::from(rgba_array)),
        };
        map_cutout = MapCutout {
            zoomed_in_map_image_size: input_config
                .zoomed_in_map_image_size
                .unwrap_or(map_cutout.zoomed_in_map_image_size),
            minimum_map_margin: input_config
                .minimum_map_margin
                .unwrap_or(map_cutout.minimum_map_margin),
            cell_outline_color: input_color.unwrap_or(map_cutout.cell_outline_color),
        }
    }
    (skip_empty_cells, map_cutout)
}

/// A [SkipEmptyCellsConfig] realized with default values where needed.
pub struct SkipEmptyCells {
    pub skipping_enabled: bool,
    pub polygon_multiplier: f32,
    pub empty_color_rgba: Rgba<u8>,
}

/// A [MapCutoutConfig] realized with default values where needed.
pub struct MapCutout {
    pub zoomed_in_map_image_size: PixelPoint,
    pub minimum_map_margin: PixelPoint,
    pub cell_outline_color: Rgba<u8>,
}
