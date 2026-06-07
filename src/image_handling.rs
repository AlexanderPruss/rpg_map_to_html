use crate::config::SkipEmptyCells;
use crate::geometry::{BoundingPolygon, Cell, CellMap};
use crate::image_handling::IsOnLine::{LeftOfLine, OnLine, RightOfLine};
use crate::{PixelBox, PixelPoint};
use image::{DynamicImage, GenericImage, GenericImageView, Pixel, Rgba};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct CroppedImageMap {
    cropped_image_paths_by_cell_coordinate: HashMap<String, String>,
}

//TODO: Tests
pub fn crop_images(
    cell_map: CellMap,
    image: &DynamicImage,
    skip_empty_cells: SkipEmptyCells,
    target_directory: String,
) -> CroppedImageMap {
    let empty_color = Rgba::from(skip_empty_cells.empty_color_rgba);
    CroppedImageMap {
        cropped_image_paths_by_cell_coordinate: cell_map
            .cells_by_coordinate
            .iter()
            .filter(|(coordinate, cell)| {
                !skip_empty_cells.skip
                    || !is_cell_empty(
                    image,
                    &cell,
                    skip_empty_cells.polygon_multiplier,
                    &empty_color,
                )
            })
            .map(|(coordinate, cell)|
                (coordinate.clone(), crop_image(image, cell, &target_directory)))
            .collect()
    }
}

fn crop_image(image: &DynamicImage, cell: &Cell, target_directory: &String) -> String {
    //TODO: Need to pre-configure how large these things are. And don't need the cell, just need its center point.
    //TODO: Probably want to pad the map with the empty_color out to 50ish pixels
    //TODO: Also probably want to draw a line around the bounding_polygon, this should also be a config thing
    //TODO: see https://docs.rs/imageproc/latest/imageproc/drawing/fn.draw_polygon.html; but maybe need to thicken it too
    let mut test_image = image.crop_imm(1,1,1,1);
    todo!()
}

//TODO: Tests
/// Checks whether a cell is empty by checking some subset of its bounding polygon to see whether
/// it is empty. We check pixels for emptiness by comparing them against the [empty_color].
fn is_cell_empty(
    image: &DynamicImage,
    cell: &Cell,
    polygon_multiplier: f32,
    empty_color: &Rgba<u8>,
) -> bool {
    let scaled_bounding_polygon = cell.scale_bounding_polygon(polygon_multiplier);
    let bounding_box = cell.bounding_polygon.get_bounding_box();
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
    //TODO: Missing tests
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
    //TODO: Missing tests
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
}

impl PixelPoint {
    ///The proof is left as an exercise to the reader. (Basic trig)
    fn is_on_line(&self, start_of_line: &PixelPoint, end_of_line: &PixelPoint) -> IsOnLine {
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

    mod bounding_polygon_contains_point {
        use crate::PixelPoint;
        use crate::geometry::BoundingPolygon;

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
        use crate::PixelPoint;
        use crate::image_handling::IsOnLine::{LeftOfLine, OnLine, RightOfLine};

        #[test]
        fn detects_points_on_lines() {
            let origin = PixelPoint { x: 0, y: 0 };
            for x in 0..10 {
                for y in 0..10 {
                    let left_point = PixelPoint { x: -x, y: -y };
                    let right_point = PixelPoint { x, y };
                    assert_eq!(OnLine, origin.is_on_line(&left_point, &right_point))
                }
            }
        }
        #[test]
        fn detects_points_left_or_right_of_lines() {
            for line_x in 0..10 {
                for line_y in 0..10 {
                    for x in 1..10 {
                        for y in 1..10 {
                            let left_point = PixelPoint {
                                x: -line_x,
                                y: -line_y,
                            };
                            let right_point = PixelPoint {
                                x: line_x,
                                y: line_y,
                            };
                            let point = PixelPoint { x, y: -y };
                            assert_eq!(LeftOfLine, point.is_on_line(&left_point, &right_point));
                            assert_eq!(RightOfLine, point.is_on_line(&right_point, &left_point));
                        }
                    }
                }
            }
        }
    }
}
