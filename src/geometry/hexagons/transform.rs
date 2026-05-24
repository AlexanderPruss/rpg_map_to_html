use std::cmp::PartialEq;
use std::collections::HashMap;
use crate::geometry::Cell;
use crate::geometry::hexagons::{FilledTopLeftCorner, FlatSides, HexCell, HexCellCoordinate, HexCellMap, HexagonGeometryDefinition};
use crate::geometry::hexagons::FilledTopLeftCorner::{FILLED, EMPTY};
use crate::geometry::hexagons::FlatSides::{FlatHorizontalSides, FlatVerticalSides};
use crate::PixelPoint;
/*enum HexGeometryTransform {
    RotateCounterclockwise(),
    HorizontalReflection()
}



impl Transforms for HexGeometryTransform {
    fn transform(&self, geometry: &HexagonGeometryDefinition) -> HexagonGeometryDefinition {
        match self {
            HexGeometryTransform::RotateCounterclockwise() => {}
            HexGeometryTransform::HorizontalReflection() => {}
        }
    }

    fn inverse_transform<'hex>(&self, transformed_geometry: &HexagonGeometryDefinition, cell_map: HexCellMap) -> HexCellMap<'hex> {
        todo!()
    }
}*/

trait Transforms {
    fn transform(&self, geometry: &HexagonGeometryDefinition) -> HexagonGeometryDefinition;
    fn inverse_transform_map(&self, cell_map: HexCellMap) -> HexCellMap;
    fn inverse_transform_point(&self, point: &PixelPoint) -> PixelPoint;
    fn inverse_transform_coordinate(&self, coordinate: &HexCellCoordinate) -> HexCellCoordinate;
    fn inverse_transform_cell(&self, hex_cell: &HexCell) -> HexCell;

}
/// Standardizes the Hex geometry by rotating and/or reflecting it until the map has flat horizontal sides
/// and the top-left corner is filled.
///
/// The [crate::geometry::hexagons] module computes a [CellMap] for this standardized geometry.
pub struct StandardizedHexGeometry {
    standardized_geometry: HexagonGeometryDefinition,
    /// Converts the [HexCellMap] for the standardized geometry back into the input geometry.
    invert_transform: fn(HexCellMap) -> HexCellMap
}

impl StandardizedHexGeometry {
    fn from(original_geometry: &HexagonGeometryDefinition) -> StandardizedHexGeometry {
        let mut standardized_geometry = original_geometry.clone();
        let mut inverse_functions : Vec<fn(HexCellMap) -> HexCellMap> = Vec::new();
        //Start with the identity function.
        inverse_functions.push(|hex_cell_map| hex_cell_map);
        if original_geometry.flat_sides == FlatSides::FlatVerticalSides {
            let mut filled_top_corner = standardized_geometry.filled_top_left_corner;
            if standardized_geometry.number_of_columns %2 == 0 {
                filled_top_corner = filled_top_corner.switch()
            }
            standardized_geometry = HexagonGeometryDefinition{
                flat_sides: FlatHorizontalSides,
                number_of_rows: standardized_geometry.number_of_columns,
                number_of_columns: standardized_geometry.number_of_rows,
                hexagon_height: standardized_geometry.hexagon_width,
                hexagon_width: standardized_geometry.hexagon_height,
                filled_top_left_corner: filled_top_corner,
            };
            inverse_functions.push(|rotated_hex_cell_map| {

               rotated_hex_cell_map
            });
        }
        unimplemented!()
    }

}

struct RotateCounterClockwise {
    original_map_dimensions: PixelPoint,
    rotated_map_dimensions: PixelPoint
}

impl RotateCounterClockwise {

    fn from(original_map_dimensions: PixelPoint) -> Self {
        RotateCounterClockwise{
            original_map_dimensions,
            rotated_map_dimensions: PixelPoint{
                x: original_map_dimensions.y,
                y: original_map_dimensions.x
            }
        }
    }
}

impl Transforms for RotateCounterClockwise {
    fn transform(&self, geometry: &HexagonGeometryDefinition) -> HexagonGeometryDefinition {
        let mut filled_top_corner = geometry.filled_top_left_corner;
        if geometry.number_of_columns %2 == 0 {
            filled_top_corner = filled_top_corner.switch()
        }
        HexagonGeometryDefinition{
            flat_sides: geometry.flat_sides.switch(),
            number_of_rows: geometry.number_of_columns,
            number_of_columns: geometry.number_of_rows,
            hexagon_height: geometry.hexagon_width,
            hexagon_width: geometry.hexagon_height,
            filled_top_left_corner: filled_top_corner,
        }
    }

    fn inverse_transform_map(&self, cell_map: HexCellMap) -> HexCellMap {
        let mut cells_by_coordinate: HashMap<&HexCellCoordinate, HexCell> = HashMap::new();
        for(coordinate, cell) in cell_map.cells_by_coordinate {
            cells_by_coordinate.insert(
                self.inverse_transform_coordinate(coordinate), //TODO: Ownership fun times
                self.inverse_transform_cell(&cell));
        }
        HexCellMap{
            cells_by_coordinate: cells_by_coordinate,
            neighbors_by_coordinate: Default::default(),
        }
    }

    fn inverse_transform_point(&self, point: &PixelPoint) -> PixelPoint {
        PixelPoint{
            x: self.rotated_map_dimensions.y - point.y,
            y: point.x
        }
    }

    fn inverse_transform_coordinate(&self, coordinate: &HexCellCoordinate) -> HexCellCoordinate {
        HexCellCoordinate{
            row: coordinate.column,
            column: coordinate.row
        }
    }

    fn inverse_transform_cell(&self, hex_cell: &HexCell) -> HexCell {
        let coordinate = self.inverse_transform_coordinate(&hex_cell.coordinate);
        HexCell {
            coordinate,
            cell: Cell{
                coordinate: coordinate.to_coordinate_string(),
                center_point: self.inverse_transform_point(&hex_cell.cell.center_point),
                bounding_polygon: hex_cell.cell.bounding_polygon.iter().map(
                    |polygon_point| self.inverse_transform_point(polygon_point)).collect(),
            }
        }
    }

}

