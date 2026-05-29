use crate::PixelPoint;
use crate::geometry::Cell;
use crate::geometry::hexagons::FlatSides::FlatHorizontalSides;
use crate::geometry::hexagons::{
    FlatSides, HexCell, HexCellCoordinate, HexCellMap, HexagonGeometryDefinition,
    StandardizedHexGeometryDefinition,
};
use std::collections::HashMap;

mod identity;
mod reflect;
mod rotate;

pub trait InvertibleTransform {
    fn transform(&self, geometry: &HexagonGeometryDefinition) -> HexagonGeometryDefinition;

    fn inverse_transform_map(&self, cell_map: HexCellMap) -> HexCellMap {
         cell_map
                .into_iter()
                .map(|(coordinate, cell)| {
                    (
                        self.inverse_transform_coordinate(coordinate),
                        self.inverse_transform_cell(cell),
                    )
                })
                .collect()
    }

    fn inverse_transform_cell(&self, hex_cell: HexCell) -> HexCell {
        let hex_coordinate = self.inverse_transform_coordinate(hex_cell.hex_coordinate);
        HexCell {
            hex_coordinate,
            neighbor_coordinates: hex_cell
                .neighbor_coordinates
                .into_iter()
                .map(|coordinate| self.inverse_transform_coordinate(coordinate))
                .collect(),
            center_point: self.inverse_transform_point(hex_cell.center_point),
            bounding_polygon: hex_cell
                .bounding_polygon
                .into_iter()
                .map(|point| self.inverse_transform_point(point))
                .collect(),
        }
    }
    fn inverse_transform_point(&self, point: PixelPoint) -> PixelPoint;
    fn inverse_transform_coordinate(&self, coordinate: HexCellCoordinate) -> HexCellCoordinate;
}

/// Standardizes the Hex geometry by rotating and/or reflecting it until the map has flat horizontal sides
/// and the top-left corner is filled. The transforms are kept track of, allowing the standardized
/// geometry to be inverted back onto the original one.
///
/// The [crate::geometry::hexagons] module computes a [CellMap] for this standardized geometry.
pub struct InvertibleStandardizedGeometry {
    standardized_geometry: StandardizedHexGeometryDefinition,
    /// The transforms applied to the original map in order to standardize it, in order of application.
    transforms_applied: Vec<Box<dyn InvertibleTransform>>,
}

impl InvertibleStandardizedGeometry {
    pub fn standardize(
        original_geometry: &HexagonGeometryDefinition,
    ) -> InvertibleStandardizedGeometry {
        let mut standardized_geometry = original_geometry.clone();
        let mut inverse_functions: Vec<fn(HexCellMap) -> HexCellMap> = Vec::new();
        //Start with the identity function.
        inverse_functions.push(|hex_cell_map| hex_cell_map);
        if original_geometry.flat_sides == FlatSides::FlatVerticalSides {
            let mut filled_top_corner = standardized_geometry.filled_top_left_corner;
            if standardized_geometry.number_of_columns % 2 == 0 {
                filled_top_corner = filled_top_corner.switch()
            }
            standardized_geometry = HexagonGeometryDefinition {
                flat_sides: FlatHorizontalSides,
                number_of_rows: standardized_geometry.number_of_columns,
                number_of_columns: standardized_geometry.number_of_rows,
                hexagon_height: standardized_geometry.hexagon_width,
                hexagon_width: standardized_geometry.hexagon_height,
                filled_top_left_corner: filled_top_corner,
            };
            inverse_functions.push(|rotated_hex_cell_map| rotated_hex_cell_map);
        }
        unimplemented!()
    }
}
