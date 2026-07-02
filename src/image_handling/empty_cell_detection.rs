use crate::geometry::{BoundingPolygon, Cell};
use crate::image_handling::empty_cell_detection::IsOnLine::{
    InvalidLine, LeftOfLine, OnLine, RightOfLine,
};
use crate::{PixelBox, PixelPoint};
use image::{DynamicImage, GenericImageView, Pixel, Rgba};

/// Checks whether a cell is empty by checking some subset of its bounding polygon to see whether
/// it is empty. We check pixels for emptiness by comparing them against the [empty_color].
pub fn is_cell_empty(
    image: &DynamicImage,
    cell: &Cell,
    polygon_multiplier: f32,
    empty_color: &Rgba<u8>,
) -> bool {
    let scaled_bounding_polygon = cell.scale_bounding_polygon(polygon_multiplier);
    let bounding_box = scaled_bounding_polygon.get_bounding_box();
    let (x_min, y_min) = (
        bounding_box.top_left_corner.x,
        bounding_box.top_left_corner.y,
    );
    let (x_max, y_max) = (
        bounding_box.bottom_right_corner.x,
        bounding_box.bottom_right_corner.y,
    );

    for x in x_min..x_max {
        for y in y_min..y_max {
            let point = PixelPoint { x, y };
            if !scaled_bounding_polygon.contains_point(point) {
                continue;
            }
            if image.get_pixel(x as u32, y as u32).to_rgba() != *empty_color {
                return false;
            }
        }
    }
    true
}

impl Cell {
    fn scale_bounding_polygon(&self, polygon_multiplier: f32) -> BoundingPolygon {
        BoundingPolygon {
            points: self
                .bounding_polygon
                .points
                .iter()
                .map(|point| {
                    let distance_from_center = *point - self.center_point;
                    let scaled_distance_from_center = distance_from_center * polygon_multiplier;
                    scaled_distance_from_center + self.center_point
                })
                .collect(),
        }
    }
}

impl BoundingPolygon {
    fn get_bounding_box(&self) -> PixelBox {
        let mut top_left = *self.points.first().unwrap();
        let mut bottom_right = top_left;
        self.points.iter().for_each(|point| {
            top_left.x = top_left.x.min(point.x);
            top_left.y = top_left.y.min(point.y);
            bottom_right.x = bottom_right.x.max(point.x);
            bottom_right.y = bottom_right.y.max(point.y);
        });
        PixelBox {
            top_left_corner: top_left,
            bottom_right_corner: bottom_right,
        }
    }

    ///Checks whether a point is contained in the polygon with the
    ///[Winding Number Algorithm](https://en.wikipedia.org/wiki/Point_in_polygon#Winding_number_algorithm).
    fn contains_point(&self, point: PixelPoint) -> bool {
        let mut winding_number = 0;
        for index in 0..self.points.len() {
            let first_point = &self.points[index];
            let second_point = if index == self.points.len() - 1 {
                &self.points[0]
            } else {
                &self.points[index + 1]
            };
            if *first_point == point {
                return true;
            }
            //We need special handling for horizontal segments because we're doing a horizontal raycast.
            if first_point.y == point.y && second_point.y == point.y {
                if first_point.x <= point.x && second_point.x >= point.x {
                    return true;
                }
                if second_point.x <= point.x && first_point.x >= point.x {
                    return true;
                }

                continue;
            }
            //We only care about crossings of the horizontal raycast.
            if first_point.y >= point.y && second_point.y >= point.y {
                continue;
            }
            if first_point.y <= point.y && second_point.y <= point.y {
                continue;
            }
            let going_up = first_point.y < point.y;
            let on_line = point.is_on_line(first_point, second_point);
            if on_line == OnLine {
                return true;
            }
            winding_number += match (going_up, on_line) {
                (true, LeftOfLine) => 1,
                (false, RightOfLine) => -1,
                (_, _) => 0,
            };
        }
        winding_number != 0
    }
}

#[derive(Debug, PartialEq)]
enum IsOnLine {
    OnLine,
    LeftOfLine,
    RightOfLine,
    InvalidLine,
}

impl PixelPoint {
    ///The proof is left as an exercise to the reader. (Basic trig)
    fn is_on_line(&self, start_of_line: &PixelPoint, end_of_line: &PixelPoint) -> IsOnLine {
        if *start_of_line == *end_of_line {
            return InvalidLine;
        }
        let trig = (start_of_line.x - self.x) * (end_of_line.y - self.y)
            - (end_of_line.x - self.x) * (start_of_line.y - self.y);
        if trig < 0 {
            return LeftOfLine;
        }
        if trig > 0 {
            return RightOfLine;
        }
        OnLine
    }
}

#[cfg(test)]
mod test {

    mod is_cell_empty {
        use crate::geometry::hexagons::fixtures::{FourByFour, ToSnapshot};
        use crate::image_handling::empty_cell_detection::is_cell_empty;
        use crate::image_handling::test::fixtures::FourByFourImages;
        use image::Rgba;

        #[test]
        fn detects_filled_cells() {
            let map_image = FourByFourImages::Standardized.load_image();
            let snapshot = FourByFour::Standardized.to_snapshot();
            let cell_map = snapshot.cell_map;
            let white = Rgba::from([255u8, 255, 255, 255]);

            let filled_coordinates: Vec<String> = vec![
                "000.002".to_string(),
                "000.003".to_string(),
                "001.001".to_string(),
                "001.002".to_string(),
                "001.003".to_string(),
                "002.000".to_string(),
                "002.001".to_string(),
                "002.002".to_string(),
                "003.000".to_string(),
            ];
            filled_coordinates.iter().for_each(|coordinate| {
                let cell = cell_map.cells_by_coordinate.get(coordinate).unwrap();
                assert_eq!(
                    false,
                    is_cell_empty(&map_image, cell, 0.3, &white),
                    "Expected coordinate {} to be detected as filled",
                    coordinate
                );
            })
        }

        #[test]
        fn detects_empty_cells_despite_the_coordinate_being_present() {
            let map_image = FourByFourImages::Standardized.load_image();
            let snapshot = FourByFour::Standardized.to_snapshot();
            let cell_map = snapshot.cell_map;
            let white = Rgba::from([255u8, 255, 255, 255]);

            let empty_coordinates: Vec<String> = vec![
                "000.000".to_string(),
                "000.001".to_string(),
                "001.000".to_string(),
                "002.003".to_string(),
                "003.001".to_string(),
                "003.002".to_string(),
                "003.003".to_string(),
            ];
            empty_coordinates.iter().for_each(|coordinate| {
                let cell = cell_map.cells_by_coordinate.get(coordinate).unwrap();
                assert_eq!(
                    true,
                    is_cell_empty(&map_image, cell, 0.3, &white),
                    "Expected coordinate {} to be detected as empty",
                    coordinate
                );
            })
        }
    }

    mod cell_scale_bounding_polygon {
        use std::default::Default;
        use crate::PixelPoint;
        use crate::geometry::{BoundingPolygon, Cell};

        fn octagon_around_origin() -> BoundingPolygon {
            BoundingPolygon {
                points: vec![
                    PixelPoint { x: -100, y: 0 },
                    PixelPoint { x: -50, y: 50 },
                    PixelPoint { x: 0, y: 100 },
                    PixelPoint { x: 50, y: 50 },
                    PixelPoint { x: 100, y: 0 },
                    PixelPoint { x: 50, y: -50 },
                    PixelPoint { x: 0, y: -100 },
                    PixelPoint { x: -50, y: -50 },
                ],
            }
        }

        #[test]
        fn can_shrink_polygons_in_towards_the_origin() {
            let center_on_origin = PixelPoint { x: 0, y: 0 };
            let octagon = octagon_around_origin();
            let expected = BoundingPolygon {
                points: octagon.points.iter().map(|point| *point * 0.5).collect(),
            };

            let cell = Cell {
                coordinate: "".to_string(),
                neighbor_coordinates: Default::default(),
                center_point: center_on_origin,
                bounding_polygon: octagon,
                inscribed_rectangle: Default::default()
            };

            assert_eq!(expected, cell.scale_bounding_polygon(0.5));
        }

        #[test]
        fn can_shrink_polygons_in_towards_arbitrary_centers() {
            let center = PixelPoint { x: 50, y: 40 };
            let octagon = octagon_around_origin();
            let octagon_around_center = BoundingPolygon {
                points: octagon.points.iter().map(|point| *point + center).collect(),
            };
            let expected = BoundingPolygon {
                points: octagon
                    .points
                    .iter()
                    .map(|point| (*point * 0.5) + center)
                    .collect(),
            };

            let cell = Cell {
                coordinate: "".to_string(),
                neighbor_coordinates: Default::default(),
                center_point: center,
                bounding_polygon: octagon_around_center,
                inscribed_rectangle: Default::default()
            };

            assert_eq!(expected, cell.scale_bounding_polygon(0.5));
        }

        #[test]
        fn can_extend_polygons_away_from_the_origin() {
            let center_on_origin = PixelPoint { x: 0, y: 0 };
            let octagon = octagon_around_origin();
            let expected = BoundingPolygon {
                points: octagon.points.iter().map(|point| *point * 2).collect(),
            };

            let cell = Cell {
                coordinate: "".to_string(),
                neighbor_coordinates: Default::default(),
                center_point: center_on_origin,
                bounding_polygon: octagon,
                inscribed_rectangle: Default::default()
            };

            assert_eq!(expected, cell.scale_bounding_polygon(2.0));
        }

        #[test]
        fn can_extend_polygons_away_from_arbitrary_centers() {
            let center = PixelPoint { x: 50, y: 40 };
            let octagon = octagon_around_origin();
            let octagon_around_center = BoundingPolygon {
                points: octagon.points.iter().map(|point| *point + center).collect(),
            };
            let expected = BoundingPolygon {
                points: octagon
                    .points
                    .iter()
                    .map(|point| (*point * 2.0) + center)
                    .collect(),
            };

            let cell = Cell {
                coordinate: "".to_string(),
                neighbor_coordinates: Default::default(),
                center_point: center,
                bounding_polygon: octagon_around_center,
                inscribed_rectangle: Default::default()
            };

            assert_eq!(expected, cell.scale_bounding_polygon(2.0));
        }
    }

    mod bounding_polygon_bounding_box {
        use crate::geometry::BoundingPolygon;
        use crate::{PixelBox, PixelPoint};

        #[test]
        fn finds_the_smallest_box_containing_the_polygon() {
            let slanted_diamond = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 0, y: 0 },
                    PixelPoint { x: 25, y: 100 },
                    PixelPoint { x: 90, y: 70 },
                    PixelPoint { x: 60, y: 20 },
                ],
            };
            let expected = PixelBox {
                top_left_corner: PixelPoint { x: 0, y: 0 },
                bottom_right_corner: PixelPoint { x: 90, y: 100 },
            };

            assert_eq!(expected, slanted_diamond.get_bounding_box());
        }

        #[test]
        #[should_panic]
        fn panics_if_the_polygon_is_empty() {
            let empty_polygon = BoundingPolygon { points: vec![] };

            empty_polygon.get_bounding_box();
        }
    }

    mod bounding_polygon_contains_point {
        use super::super::*;

        #[test]
        fn square_identifies_points_in_and_outside() {
            let square = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 0, y: 0 },
                    PixelPoint { x: 0, y: 10 },
                    PixelPoint { x: 10, y: 10 },
                    PixelPoint { x: 10, y: 0 },
                ],
            };
            for x in -5..15 {
                for y in -5..15 {
                    let point = PixelPoint { x, y };
                    let expect_to_be_contained = x >= 0 && x <= 10 && y >= 0 && y <= 10;
                    assert_eq!(
                        expect_to_be_contained,
                        square.contains_point(point),
                        "Expected Square.contains {:?} to be {}",
                        point,
                        expect_to_be_contained
                    )
                }
            }
        }

        #[test]
        fn triangle_identifies_points_in_and_outside() {
            let triangle = BoundingPolygon {
                points: vec![
                    PixelPoint { x: 0, y: 0 },
                    PixelPoint { x: 0, y: 10 },
                    PixelPoint { x: 10, y: 10 },
                ],
            };
            for x in -5..15 {
                for y in -5..15 {
                    let point = PixelPoint { x, y };
                    let expect_to_be_contained = x >= 0 && x <= 10 && y >= 0 && y <= 10 && y >= x;
                    assert_eq!(
                        expect_to_be_contained,
                        triangle.contains_point(point),
                        "Expected Triangle.contains {:?} to be {}",
                        point,
                        expect_to_be_contained
                    )
                }
            }
        }
    }
    mod pixel_point_is_on_line {
        use super::super::*;

        #[test]
        fn detects_points_on_lines() {
            let origin = PixelPoint { x: 0, y: 0 };
            for x in 0..10 {
                for y in 1..10 {
                    let left_point = PixelPoint { x: -x, y: -y };
                    let right_point = PixelPoint { x, y };
                    assert_eq!(OnLine, origin.is_on_line(&left_point, &right_point))
                }
            }
        }
        #[test]
        fn detects_points_left_or_right_of_lines() {
            for line_x in 1..10 {
                for line_y in 1..10 {
                    for x in line_x + 1..10 {
                        for y in line_y + 1..10 {
                            let left_point = PixelPoint {
                                x: -line_x,
                                y: -line_y,
                            };
                            let right_point = PixelPoint {
                                x: line_x,
                                y: line_y,
                            };
                            let point = PixelPoint { x, y: -y };
                            assert_eq!(
                                LeftOfLine,
                                point.is_on_line(&left_point, &right_point),
                                "Point ${:?} should have been left of the line from ${:?}, ${:?}",
                                point,
                                left_point,
                                right_point
                            );
                            assert_eq!(RightOfLine, point.is_on_line(&right_point, &left_point));
                        }
                    }
                }
            }
        }

        #[test]
        fn detects_invalid_lines() {
            for line_x in -10..10 {
                for line_y in -10..10 {
                    for x in -10..10 {
                        for y in -10..10 {
                            let line_point = PixelPoint {
                                x: -line_x,
                                y: -line_y,
                            };
                            let point = PixelPoint { x, y };
                            assert_eq!(InvalidLine, point.is_on_line(&line_point, &line_point))
                        }
                    }
                }
            }
        }
    }
}
