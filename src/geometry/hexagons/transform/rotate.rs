use crate::PixelPoint;
use crate::geometry::hexagons::transform::{InvertibleTransform, Transform};
use crate::geometry::hexagons::{HexCellCoordinate, HexagonGeometryDefinition};

#[derive(PartialEq, Debug)]
pub struct RotateCounterClockwise {
    pub rotated_map_dimensions: PixelPoint,
    original_number_of_columns: u8,
}

impl RotateCounterClockwise {
    pub fn rotate(
        geometry_dimensions: PixelPoint,
        geometry: &HexagonGeometryDefinition,
    ) -> (Self, HexagonGeometryDefinition) {
        let rotation = RotateCounterClockwise {
            rotated_map_dimensions: PixelPoint {
                x: geometry_dimensions.y,
                y: geometry_dimensions.x,
            },
            original_number_of_columns: geometry.number_of_columns,
        };
        let rotated_geometry = rotation.transform(geometry);
        (rotation, rotated_geometry)
    }
}

impl Transform for RotateCounterClockwise {
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
}

impl InvertibleTransform for RotateCounterClockwise {
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
            //An extra -1 is needed because our coordinates are 0-indexed.
            column: self.original_number_of_columns - coordinate.row - 1,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::PixelPoint;
    use crate::geometry::hexagons::FlatSides::{FlatHorizontalSides, FlatVerticalSides};
    use crate::geometry::hexagons::transform::InvertibleTransform;
    use crate::geometry::hexagons::transform::rotate::RotateCounterClockwise;
    use crate::geometry::hexagons::{FilledTopLeftCorner, HexagonGeometryDefinition};
    use crate::geometry::hexagons::{HexCell, HexCellCoordinate, HexCellMap};
    use FilledTopLeftCorner::EMPTY;
    use FilledTopLeftCorner::FILLED;
    mod rotate_geometry {
        use super::*;
        #[test]
        fn rotates_geometries_with_even_columns() {
            let geometry_dimensions = PixelPoint { x: 1000, y: 1500 };

            for test_case in 0..4 {
                let flat_sides = match test_case & 1 {
                    0 => FlatHorizontalSides,
                    1 => FlatVerticalSides,
                    _ => panic!(),
                };
                let filled_top_left_corner = match test_case & 2 {
                    0 => FILLED,
                    2 => EMPTY,
                    _ => panic!(),
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
                    original_number_of_columns: 6,
                };

                let (transform, rotated_geometry) =
                    RotateCounterClockwise::rotate(geometry_dimensions, &input_geometry);

                assert_eq!(transform, expected_transform);
                assert_eq!(rotated_geometry, expected_geometry);
            }
        }

        #[test]
        fn rotates_geometries_with_odd_columns() {
            let geometry_dimensions = PixelPoint { x: 1000, y: 1500 };

            for test_case in 0..4 {
                let flat_sides = match test_case & 1 {
                    0 => FlatHorizontalSides,
                    1 => FlatVerticalSides,
                    _ => panic!(),
                };
                let filled_top_left_corner = match test_case & 2 {
                    0 => FILLED,
                    2 => EMPTY,
                    _ => panic!(),
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
                    filled_top_left_corner,
                };
                let expected_transform = RotateCounterClockwise {
                    rotated_map_dimensions: PixelPoint { x: 1500, y: 1000 },
                    original_number_of_columns: 5,
                };

                let (transform, rotated_geometry) =
                    RotateCounterClockwise::rotate(geometry_dimensions, &input_geometry);

                assert_eq!(transform, expected_transform);
                assert_eq!(rotated_geometry, expected_geometry);
            }
        }
    }

    /// Minimal check that we're using the default implementation for transforming maps and cells.
    mod inverse_transform_default_impl {
        use super::*;
        use crate::geometry::hexagons::test::fixtures::{FourByFour, ToSnapshot};

        #[test]
        fn rotates_hex_cell_map_clockwise() {
            let standardized_cell_map = FourByFour::Standardized.to_snapshot().hex_cell_map;
            let needs_rotation = FourByFour::MustBeRotated.to_snapshot();
            let expected_cell_map = needs_rotation.hex_cell_map;

            let (transform, _) = RotateCounterClockwise::rotate(
                needs_rotation.dimensions,
                &needs_rotation.geometry_definition,
            );
            let cells_after_inverse_rotation =
                transform.inverse_transform_map(standardized_cell_map);

            assert_eq!(expected_cell_map, cells_after_inverse_rotation)
        }

        mod inverse_transform_point {
            use super::*;

            #[test]
            fn rotates_points_clockwise() {
                let geometry_dimensions = PixelPoint { x: 100, y: 200 };
                let input_geometry = HexagonGeometryDefinition {
                    flat_sides: FlatVerticalSides,
                    number_of_rows: 1,
                    number_of_columns: 2,
                    hexagon_height: 40.0,
                    hexagon_width: 20.0,
                    filled_top_left_corner: FILLED,
                };
                let (transform, _) =
                    RotateCounterClockwise::rotate(geometry_dimensions, &input_geometry);

                let point_on_y_axis = PixelPoint { x: 0, y: 25 };
                assert_eq!(
                    transform.inverse_transform_point(point_on_y_axis),
                    PixelPoint { x: 75, y: 0 }
                );

                let point_on_x_axis = PixelPoint { x: 25, y: 0 };
                assert_eq!(
                    transform.inverse_transform_point(point_on_x_axis),
                    PixelPoint { x: 100, y: 25 }
                );

                let point_near_middle = PixelPoint { x: 110, y: 60 };
                assert_eq!(
                    transform.inverse_transform_point(point_near_middle),
                    PixelPoint { x: 40, y: 110 }
                )
            }
        }

        mod inverse_transform_coordinate {
            use super::*;

            #[test]
            fn rotates_coordinates_clockwise() {
                let geometry_dimensions = PixelPoint { x: 100, y: 200 };
                let input_geometry = HexagonGeometryDefinition {
                    flat_sides: FlatVerticalSides,
                    number_of_rows: 10,
                    number_of_columns: 20,
                    hexagon_height: 40.0,
                    hexagon_width: 20.0,
                    filled_top_left_corner: FILLED,
                };
                let (transform, _) =
                    RotateCounterClockwise::rotate(geometry_dimensions, &input_geometry);

                let coordinate_on_y_axis = HexCellCoordinate { row: 0, column: 5 };
                assert_eq!(
                    transform.inverse_transform_coordinate(coordinate_on_y_axis),
                    HexCellCoordinate { row: 5, column: 19 }
                );

                let coordinate_on_x_axis = HexCellCoordinate { row: 5, column: 0 };
                assert_eq!(
                    transform.inverse_transform_coordinate(coordinate_on_x_axis),
                    HexCellCoordinate { row: 0, column: 14 }
                );

                let coordinate_near_middle = HexCellCoordinate { row: 4, column: 6 };
                assert_eq!(
                    transform.inverse_transform_coordinate(coordinate_near_middle),
                    HexCellCoordinate { row: 6, column: 15 }
                )
            }
        }
    }
}
