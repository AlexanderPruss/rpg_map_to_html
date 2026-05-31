use crate::PixelPoint;
use crate::geometry::hexagons::FilledTopLeftCorner::{EMPTY, FILLED};
use crate::geometry::hexagons::FlatSides::FlatVerticalSides;
use crate::geometry::hexagons::standardized::StandardizedHexCellMap;
use crate::geometry::hexagons::transform::InvertibleStandardizedGeometry;
use crate::geometry::{BoundingPolygon, Cell, CellMap, ComputesCellMap};
use serde::Deserialize;
use std::collections::HashMap;
use FlatSides::FlatHorizontalSides;

mod transform;

mod standardized;

/// A hex map. All rows have the same number of hexes, all columns have the same number of hexes.
///
/// It is possible to input
#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct HexagonGeometryDefinition {
    flat_sides: FlatSides,
    number_of_rows: u8,
    number_of_columns: u8,
    /// The units here can be pixels, cm, whatever, so long as they're consistent with [hexagon_width]
    hexagon_height: f32,
    /// The units here can be pixels, cm, whatever, so long as they're consistent with [hexagon_height]
    hexagon_width: f32,
    filled_top_left_corner: FilledTopLeftCorner,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct HexCellCoordinate {
    row: u8,
    column: u8,
}
impl HexCellCoordinate {
    fn to_coordinate_string(&self) -> String {
        format!("{}-{}", self.column, self.row)
    }
}

#[derive(Debug, PartialEq, Clone)]
struct HexCell {
    hex_coordinate: HexCellCoordinate,
    neighbor_coordinates: Vec<HexCellCoordinate>,
    center_point: PixelPoint,
    bounding_polygon: BoundingPolygon,
}

#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum FlatSides {
    FlatVerticalSides,
    FlatHorizontalSides,
}

/// Whether the top left corner of the map is filled in by a hex.
///
/// This is equivalent to asking if hex (0, 0) exists in the map.
#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum FilledTopLeftCorner {
    /// There's an empty hex at the top left of the map. For flat top hexes, the corner looks like this:
    ///
    ///``` picture
    ///                         ••••••••••
    ///                        •          •
    ///                       •            •
    ///             ••••••••••     (0,1)    •
    ///            •          •            •
    ///           •            •          •
    ///          •     (1,0)    ••••••••••
    ///           •            •
    ///            •          •
    ///             ••••••••••
    ///```
    EMPTY,
    /// There's no empty space at the top left of the map. For flat top hexes, the corner looks like this:
    ///
    ///``` picture
    ///             ••••••••••
    ///            •          •
    ///           •            •
    ///          •     (0,0)    ••••••••••
    ///           •            •          •
    ///            •          •            •
    ///             ••••••••••    (0,1)     •
    ///                       •            •
    ///                        •          •
    ///                         ••••••••••
    /// ```
    FILLED,
}

impl FilledTopLeftCorner {
    fn switch(&self) -> FilledTopLeftCorner {
        match self {
            EMPTY => FILLED,
            FILLED => EMPTY,
        }
    }
}

impl FlatSides {
    fn switch(&self) -> FlatSides {
        match self {
            FlatHorizontalSides => FlatVerticalSides,
            FlatVerticalSides => FlatHorizontalSides,
        }
    }
}

type HexCellMap = HashMap<HexCellCoordinate, HexCell>;

fn offset_map(hex_cell_map: HexCellMap, offset: PixelPoint) -> HexCellMap {
    if (offset == PixelPoint { x: 0, y: 0 }) {
        return hex_cell_map;
    }
    hex_cell_map
        .into_iter()
        .map(|(coordinate, cell)| {
            let cell_shifted_by_margin = HexCell {
                hex_coordinate: cell.hex_coordinate,
                neighbor_coordinates: cell.neighbor_coordinates,
                center_point: cell.center_point + offset,
                bounding_polygon: cell.bounding_polygon.offset_by(offset),
            };
            (coordinate, cell_shifted_by_margin)
        })
        .collect()
}
fn to_cell_map(hex_cell_map: HexCellMap) -> CellMap {
    CellMap {
        cells_by_coordinate: hex_cell_map
            .into_iter()
            .map(|(hex_coordinate, hex_cell)| {
                (
                    hex_coordinate.to_coordinate_string(),
                    Cell {
                        coordinate: hex_cell.hex_coordinate.to_coordinate_string(),
                        neighbor_coordinates: hex_cell
                            .neighbor_coordinates
                            .into_iter()
                            .map(|hex_coordinate| hex_coordinate.to_coordinate_string())
                            .collect(),
                        center_point: hex_cell.center_point,
                        bounding_polygon: hex_cell.bounding_polygon,
                    },
                )
            })
            .collect(),
    }
}

impl ComputesCellMap for HexagonGeometryDefinition {
    fn compute_cell_map(&self, map_dimensions: PixelPoint, map_margin: PixelPoint) -> CellMap {
        let map_dimensions_without_margin = PixelPoint {
            x: map_dimensions.x - 2 * map_margin.x,
            y: map_dimensions.y - 2 * map_margin.y,
        };

        let invertible_standardized_geometry =
            InvertibleStandardizedGeometry::standardize(self, map_dimensions_without_margin);
        let standardized_hex_cell_map: StandardizedHexCellMap =
            invertible_standardized_geometry.compute_standardized_cell_map();

        let hex_cell_map = standardized_hex_cell_map.invert_standardization();
        let hex_cell_map = offset_map(hex_cell_map, map_margin);
        to_cell_map(hex_cell_map)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod hex_cell_coordinate {
        #[test]
        fn stringifies_the_coordinate(){
            unimplemented!()
        }
    }
    
    mod offset_map {
        
        #[test]
        fn offsets_all_pixels_of_the_cell_map(){
            unimplemented!()
        }
        
        #[test]
        fn changes_nothing_if_the_offset_is_empty(){
            unimplemented!()
        }
    }
    
    mod to_cell_map {
        #[test]
        fn stringifies_hex_coordinates() {
            unimplemented!()

        }
    }
    
    mod compute_cell_map {
        
        #[test]
        fn computes_a_cell_map_for_standardized_geometries(){
            unimplemented!()
        }

        #[test]
        fn computes_a_cell_map_for_unstandardized_geometries(){
            unimplemented!()
        }
    }
}