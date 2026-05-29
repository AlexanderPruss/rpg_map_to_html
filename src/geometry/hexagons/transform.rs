use crate::geometry::BoundingPolygon;
use crate::PixelPoint;
use crate::geometry::hexagons::{FilledTopLeftCorner, FlatSides, HexCell, HexCellCoordinate, HexCellMap, HexagonGeometryDefinition, StandardizedHexCellMap, StandardizedHexGeometryDefinition};
use crate::geometry::hexagons::transform::identity::Identity;
use crate::geometry::hexagons::transform::reflect::ReflectOverXAxis;
use crate::geometry::hexagons::transform::rotate::RotateCounterClockwise;

pub mod identity;
pub mod reflect;
pub mod rotate;



/// Standardizes the Hex geometry by rotating and/or reflecting it until the map has flat horizontal sides
/// and the top-left corner is filled. The transforms are kept track of, allowing the standardized
/// geometry to be inverted back onto the original one.
///
/// The [crate::geometry::hexagons] module computes a [CellMap] for this standardized geometry.
pub struct InvertibleStandardizedGeometry {
    pub standardized_geometry: StandardizedHexGeometryDefinition,
    /// The transforms applied to the original map in order to standardize it, in order of application.
    pub transforms_applied: Vec<Box<dyn InvertibleTransform>>,
}

impl InvertibleStandardizedGeometry {
    /// Standardizes the input geometry; see [InvertibleStandardizedGeometry]
    pub fn standardize(
        original_geometry: &HexagonGeometryDefinition,
        geometry_dimensions: PixelPoint
    ) -> InvertibleStandardizedGeometry {
        let transformed_geometry = &mut original_geometry.clone();
        let mut transformed_dimensions = geometry_dimensions;
        let mut transforms_applied : Vec<Box<dyn InvertibleTransform>> = Vec::new();
        transforms_applied.push(Box::new(Identity{}));

        if transformed_geometry.flat_sides == FlatSides::FlatVerticalSides {
            let (rotation, reflected_geometry) = RotateCounterClockwise::rotate(transformed_dimensions, &transformed_geometry);
            *transformed_geometry = reflected_geometry;
            transformed_dimensions = rotation.rotated_map_dimensions;
            transforms_applied.push(Box::new(rotation));
        }
        if transformed_geometry.filled_top_left_corner == FilledTopLeftCorner::EMPTY {
            let (reflection, reflected_geometry) = ReflectOverXAxis::reflect(transformed_dimensions, transformed_geometry);
            *transformed_geometry = reflected_geometry;
            transforms_applied.push(Box::new(reflection));
        }
        InvertibleStandardizedGeometry {
            standardized_geometry: StandardizedHexGeometryDefinition {
                number_of_rows: transformed_geometry.number_of_rows,
                number_of_columns: transformed_geometry.number_of_columns,
                hexagon_height: transformed_geometry.hexagon_height,
                hexagon_width: transformed_geometry.hexagon_width,
                geometry_dimensions : transformed_dimensions,
            },
            transforms_applied,
        }
    }
}

trait Transform {
    fn transform(&self, geometry: &HexagonGeometryDefinition) -> HexagonGeometryDefinition;
}

pub trait InvertibleTransform {
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
            bounding_polygon: BoundingPolygon{points: hex_cell
                .bounding_polygon
                .points
                .into_iter()
                .map(|point| self.inverse_transform_point(point))
                .collect()},
        }
    }
    fn inverse_transform_point(&self, point: PixelPoint) -> PixelPoint;
    fn inverse_transform_coordinate(&self, coordinate: HexCellCoordinate) -> HexCellCoordinate;
}

#[cfg(test)]
mod test {
    use super::*;
    mod standardize_geometry {
        #[test]
        fn standardizes_already_standardized_geometry(){
            unimplemented!()
        }
        #[test]
        fn standardizes_geometry_that_must_be_rotated(){
            unimplemented!()
        }

        #[test]
        fn standardizes_geometry_that_must_be_reflected(){
            unimplemented!()
        }

        #[test]
        fn standardizes_geometry_that_must_be_rotated_and_reflected(){
            unimplemented!()
        }
        
    }
}