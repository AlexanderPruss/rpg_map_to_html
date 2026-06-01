use crate::PixelPoint;
use crate::geometry::hexagons::transform::{InvertibleTransform, Transform};
use crate::geometry::hexagons::{HexCellCoordinate, HexagonGeometryDefinition};

#[derive(PartialEq, Debug)]
pub struct ReflectOverXAxis {
    map_dimensions: PixelPoint,
    number_of_columns: u8,
}

impl ReflectOverXAxis {
    pub fn reflect(
        geometry_dimensions: PixelPoint,
        geometry: &HexagonGeometryDefinition,
    ) -> (Self, HexagonGeometryDefinition) {
        let reflection = ReflectOverXAxis {
            map_dimensions: geometry_dimensions,
            number_of_columns: geometry.number_of_columns,
        };
        let rotated_geometry = reflection.transform(geometry);
        (reflection, rotated_geometry)
    }
}

impl Transform for ReflectOverXAxis {
    fn transform(&self, geometry: &HexagonGeometryDefinition) -> HexagonGeometryDefinition {
        let mut filled_top_corner = geometry.filled_top_left_corner;
        if geometry.number_of_columns % 2 == 0 {
            filled_top_corner = filled_top_corner.switch()
        }
        HexagonGeometryDefinition {
            flat_sides: geometry.flat_sides,
            number_of_rows: geometry.number_of_rows,
            number_of_columns: geometry.number_of_columns,
            hexagon_height: geometry.hexagon_height,
            hexagon_width: geometry.hexagon_width,
            filled_top_left_corner: filled_top_corner,
        }
    }
}
impl InvertibleTransform for ReflectOverXAxis {
    fn inverse_transform_point(&self, point: PixelPoint) -> PixelPoint {
        PixelPoint {
            x: self.map_dimensions.x - point.x,
            y: point.y,
        }
    }

    fn inverse_transform_coordinate(&self, coordinate: HexCellCoordinate) -> HexCellCoordinate {
        HexCellCoordinate {
            row: coordinate.row,
            column: self.number_of_columns - coordinate.column - 1,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::PixelPoint;
    use crate::geometry::hexagons::FlatSides::{FlatHorizontalSides, FlatVerticalSides};
    use crate::geometry::hexagons::transform::InvertibleTransform;
    use crate::geometry::hexagons::{FilledTopLeftCorner, HexagonGeometryDefinition};
    use crate::geometry::hexagons::{HexCell, HexCellCoordinate};
    use FilledTopLeftCorner::EMPTY;
    use FilledTopLeftCorner::FILLED;

    mod reflect_geometry {
        use super::*;
        use crate::geometry::hexagons::transform::reflect::ReflectOverXAxis;
        #[test]
        fn reflects_geometries_with_even_columns() {
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
                    flat_sides,
                    number_of_rows: 5,
                    number_of_columns: 6,
                    hexagon_height: 7.0,
                    hexagon_width: 8.0,
                    filled_top_left_corner: filled_top_left_corner.switch(),
                };
                let expected_transform = ReflectOverXAxis {
                    map_dimensions: geometry_dimensions,
                    number_of_columns: 6,
                };

                let (transform, rotated_geometry) =
                    ReflectOverXAxis::reflect(geometry_dimensions, &input_geometry);

                assert_eq!(transform, expected_transform);
                assert_eq!(rotated_geometry, expected_geometry);
            }
        }

        #[test]
        fn reflects_geometries_with_odd_columns() {
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
                    number_of_columns: 7,
                    hexagon_height: 7.0,
                    hexagon_width: 8.0,
                    filled_top_left_corner,
                };
                let expected_geometry = HexagonGeometryDefinition {
                    flat_sides,
                    number_of_rows: 5,
                    number_of_columns: 7,
                    hexagon_height: 7.0,
                    hexagon_width: 8.0,
                    filled_top_left_corner,
                };
                let expected_transform = ReflectOverXAxis {
                    map_dimensions: geometry_dimensions,
                    number_of_columns: 7,
                };

                let (transform, rotated_geometry) =
                    ReflectOverXAxis::reflect(geometry_dimensions, &input_geometry);

                assert_eq!(transform, expected_transform);
                assert_eq!(rotated_geometry, expected_geometry);
            }
        }
    }

    /// Minimal check that we're using the default implementation for transforming maps and cells.
    mod inverse_transform_default_impl {
        use super::*;
        use crate::geometry::hexagons::transform::reflect::ReflectOverXAxis;

        use crate::geometry::hexagons::test::fixtures::{FourByFour, ToSnapshot};

        #[test]
        fn reflects_hex_cell_map_over_x_axis() {
            let standardized_cell_map = FourByFour::Standardized.to_snapshot().hex_cell_map;
            let needs_reflection = FourByFour::MustBeReflected.to_snapshot();
            let expected_cell_map = needs_reflection.hex_cell_map;

            let (transform, _) = ReflectOverXAxis::reflect(
                needs_reflection.dimensions,
                &needs_reflection.geometry_definition,
            );
            let cells_after_inverse_rotation =
                transform.inverse_transform_map(standardized_cell_map);

            assert_eq!(expected_cell_map, cells_after_inverse_rotation)
        }
    }

    mod inverse_transform_point {
        use crate::PixelPoint;
        use crate::geometry::hexagons::FilledTopLeftCorner::FILLED;
        use crate::geometry::hexagons::FlatSides::FlatVerticalSides;
        use crate::geometry::hexagons::HexagonGeometryDefinition;
        use crate::geometry::hexagons::transform::InvertibleTransform;
        use crate::geometry::hexagons::transform::reflect::ReflectOverXAxis;

        #[test]
        fn reflects_points_over_x_axis() {
            let geometry_dimensions = PixelPoint { x: 100, y: 200 };
            let input_geometry = HexagonGeometryDefinition {
                flat_sides: FlatVerticalSides,
                number_of_rows: 1,
                number_of_columns: 2,
                hexagon_height: 40.0,
                hexagon_width: 20.0,
                filled_top_left_corner: FILLED,
            };
            let (transform, _) = ReflectOverXAxis::reflect(geometry_dimensions, &input_geometry);

            let point_on_y_axis = PixelPoint { x: 0, y: 25 };
            assert_eq!(
                transform.inverse_transform_point(point_on_y_axis),
                PixelPoint { x: 100, y: 25 }
            );

            let point_on_x_axis = PixelPoint { x: 25, y: 0 };
            assert_eq!(
                transform.inverse_transform_point(point_on_x_axis),
                PixelPoint { x: 75, y: 0 }
            );

            let point_near_middle = PixelPoint { x: 45, y: 90 };
            assert_eq!(
                transform.inverse_transform_point(point_near_middle),
                PixelPoint { x: 55, y: 90 }
            )
        }
    }

    mod inverse_transform_coordinate {
        use super::*;
        use crate::geometry::hexagons::transform::reflect::ReflectOverXAxis;

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
            let (transform, _) = ReflectOverXAxis::reflect(geometry_dimensions, &input_geometry);

            let coordinate_on_y_axis = HexCellCoordinate { row: 0, column: 5 };
            assert_eq!(
                transform.inverse_transform_coordinate(coordinate_on_y_axis),
                HexCellCoordinate { row: 0, column: 14 }
            );

            let coordinate_on_x_axis = HexCellCoordinate { row: 5, column: 0 };
            assert_eq!(
                transform.inverse_transform_coordinate(coordinate_on_x_axis),
                HexCellCoordinate { row: 5, column: 19 }
            );

            let coordinate_near_middle = HexCellCoordinate { row: 4, column: 6 };
            assert_eq!(
                transform.inverse_transform_coordinate(coordinate_near_middle),
                HexCellCoordinate { row: 4, column: 13 }
            )
        }
    }
}
