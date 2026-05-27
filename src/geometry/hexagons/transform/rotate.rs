use crate::geometry::hexagons::{HexCellCoordinate, HexagonGeometryDefinition};
use crate::geometry::hexagons::transform::InvertibleTransform;
use crate::PixelPoint;

#[derive(PartialEq, Debug)]
struct RotateCounterClockwise {
    rotated_map_dimensions: PixelPoint,
    rotated_number_of_columns: i8,
}

impl RotateCounterClockwise {
    pub fn rotate(geometry_dimensions: PixelPoint, geometry: &HexagonGeometryDefinition) -> (Self, HexagonGeometryDefinition) {
        let rotation = RotateCounterClockwise {
            rotated_map_dimensions: PixelPoint {
                x: geometry_dimensions.y,
                y: geometry_dimensions.x,
            },
            rotated_number_of_columns: geometry.number_of_rows,
        };
        let rotated_geometry = rotation.transform(geometry);
        (rotation, rotated_geometry)
    }
}

impl InvertibleTransform for RotateCounterClockwise {
    fn transform(&self, geometry: &HexagonGeometryDefinition) -> HexagonGeometryDefinition {
        let mut filled_top_corner = geometry.filled_top_left_corner;
        if geometry.number_of_columns % 2 == 0 {
            filled_top_corner = filled_top_corner.switch()
        }
        HexagonGeometryDefinition {
            flat_sides: geometry.flat_sides.switch(),
            number_of_rows: geometry.number_of_columns,
            number_of_columns: geometry.number_of_rows,
            hexagon_height: geometry.hexagon_width,
            hexagon_width: geometry.hexagon_height,
            filled_top_left_corner: filled_top_corner,
        }
    }

    fn inverse_transform_point(&self, point: PixelPoint) -> PixelPoint {
        PixelPoint {
            x: self.rotated_map_dimensions.y - point.y,
            y: point.x,
        }
    }

    /// Rotates the coordinate clockwise.
    fn inverse_transform_coordinate(&self, coordinate: HexCellCoordinate) -> HexCellCoordinate {
        HexCellCoordinate {
            row: coordinate.column,
            column: self.rotated_number_of_columns - coordinate.row,
        }
    }
}

#[cfg(test)]
mod test {
    use FilledTopLeftCorner::FILLED;
    use crate::geometry::hexagons::{FilledTopLeftCorner, HexagonGeometryDefinition};
    use crate::geometry::hexagons::FilledTopLeftCorner::EMPTY;
    use crate::geometry::hexagons::FlatSides::{FlatHorizontalSides, FlatVerticalSides};
    use crate::geometry::hexagons::transform::rotate::RotateCounterClockwise;
    use crate::PixelPoint;
    mod rotate_geometry {
        use super::*;
        #[test]
        fn rotates_geometries_with_even_columns() {
            let geometry_dimensions = PixelPoint { x: 1000, y: 1500 };

            for test_case in 0..4 {
                let flat_sides = match test_case & 1 {
                    0 => { FlatHorizontalSides }
                    1 => { FlatVerticalSides }
                    _ => panic!()
                };
                let filled_top_left_corner = match test_case & 2 {
                    0 => { FILLED }
                    2 => { EMPTY }
                    _ => panic!()
                };
                let input_geometry = HexagonGeometryDefinition {
                    flat_sides,
                    number_of_rows: 5,
                    number_of_columns: 6,
                    hexagon_height: 7.0,
                    hexagon_width: 8.0,
                    filled_top_left_corner,
                };
                let expected_geometry = HexagonGeometryDefinition {
                    flat_sides: flat_sides.switch(),
                    number_of_rows: 6,
                    number_of_columns: 5,
                    hexagon_height: 8.0,
                    hexagon_width: 7.0,
                    filled_top_left_corner: filled_top_left_corner.switch(),
                };
                let expected_transform = RotateCounterClockwise {
                    rotated_map_dimensions: PixelPoint { x: 1500, y: 1000 },
                    rotated_number_of_columns: 5,
                };
                let (transform, rotated_geometry) = RotateCounterClockwise::rotate(geometry_dimensions, &input_geometry);
                assert_eq!(transform, expected_transform);
                assert_eq!(rotated_geometry, expected_geometry);
            }
        }

        #[test]
        fn rotates_geometries_with_odd_columns() {
            let geometry_dimensions = PixelPoint { x: 1000, y: 1500 };

            for test_case in 0..4 {
                let flat_sides = match test_case & 1 {
                    0 => { FlatHorizontalSides }
                    1 => { FlatVerticalSides }
                    _ => panic!()
                };
                let filled_top_left_corner = match test_case & 2 {
                    0 => { FILLED }
                    2 => { EMPTY }
                    _ => panic!()
                };
                let input_geometry = HexagonGeometryDefinition {
                    flat_sides,
                    number_of_rows: 6,
                    number_of_columns: 5,
                    hexagon_height: 7.0,
                    hexagon_width: 8.0,
                    filled_top_left_corner,
                };
                let expected_geometry = HexagonGeometryDefinition {
                    flat_sides: flat_sides.switch(),
                    number_of_rows: 5,
                    number_of_columns: 6,
                    hexagon_height: 8.0,
                    hexagon_width: 7.0,
                    filled_top_left_corner
                };
                let expected_transform = RotateCounterClockwise {
                    rotated_map_dimensions: PixelPoint { x: 1500, y: 1000 },
                    rotated_number_of_columns: 6,
                };
                let (transform, rotated_geometry) = RotateCounterClockwise::rotate(geometry_dimensions, &input_geometry);
                assert_eq!(transform, expected_transform);
                assert_eq!(rotated_geometry, expected_geometry);
            }
        }
    }

    mod inverse_transform_map {}

    mod inverse_transform_cell {}

    mod inverse_transform_point {}

    mod inverse_transform_coordinate {}
}