use crate::config::{MapCutoutConfig, SkipEmptyCellsConfig};
use crate::geometry::{BoundingPolygon, Cell, CellMap};
use crate::image_handling::IsOnLine::{InvalidLine, LeftOfLine, OnLine, RightOfLine};
use crate::{PixelBox, PixelPoint};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Pixel, Rgba};
use imageproc::drawing::draw_line_segment_mut;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct CroppedImageMap {
    cropped_image_paths_by_cell_coordinate: HashMap<String, PathBuf>,
}

//TODO: Tests
/// Cuts out the zoomed-in map images that are displayed on the details page of any given cell.
///
/// To
pub fn create_cutout_images(
    cell_map: CellMap,
    original_image: &DynamicImage,
    image_margins: PixelPoint,
    target_directory: String,
    skip_empty_cells_config: Option<SkipEmptyCellsConfig>,
    map_cutout_config: Option<MapCutoutConfig>,
) -> CroppedImageMap {
    let (skip_empty_cells, map_cutout) = resolve_config(skip_empty_cells_config, map_cutout_config);
    let empty_color = Rgba::from(skip_empty_cells.empty_color_rgba);
    let (padded_image, margin_offset) = pad_image(
        original_image,
        image_margins,
        map_cutout.minimum_map_margin,
        skip_empty_cells.empty_color_rgba,
    );
    CroppedImageMap {
        cropped_image_paths_by_cell_coordinate: cell_map
            .cells_by_coordinate
            .iter()
            .filter(|(_coordinate, cell)| {
                !skip_empty_cells.skipping_enabled
                    || !is_cell_empty(
                        original_image,
                        cell,
                        skip_empty_cells.polygon_multiplier,
                        &empty_color,
                    )
            })
            .map(|(coordinate, cell)| {
                (
                    coordinate.clone(),
                    create_and_save_cutout_map_image(
                        &padded_image,
                        margin_offset,
                        cell,
                        &target_directory,
                        &map_cutout,
                    ),
                )
            })
            .collect(),
    }
}

//TODO: Test
/// Ensures that the image has a margin of at least [minimum_margin], creating/extending the
/// image's margin and filling it with the [empty_color] if necessary.
///
/// Returns the padded image as well as the resulting effective margin.
fn pad_image(
    image: &DynamicImage,
    current_margin: PixelPoint,
    minimum_margin: PixelPoint,
    empty_color: Rgba<u8>,
) -> (DynamicImage, PixelPoint) {
    if current_margin.x >= minimum_margin.x && current_margin.y >= minimum_margin.y {
        return (image.clone(), PixelPoint { x: 0, y: 0 });
    }
    let additional_buffer_needed = PixelPoint {
        x: minimum_margin.y - current_margin.x,
        y: minimum_margin.y - current_margin.y,
    };
    let mut padded_buffer = ImageBuffer::from_pixel(
        image.width() + 2 * additional_buffer_needed.x as u32,
        image.height() + 2 * additional_buffer_needed.y as u32,
        empty_color,
    );
    image::imageops::overlay(&mut padded_buffer, image, 0, 0);
    (DynamicImage::from(padded_buffer), additional_buffer_needed)
}

/// Creates a cutout map image for a given cell. The cell is centered in this cutout as much as possible,
/// its polygon is outlined, and the image is saved.
fn create_and_save_cutout_map_image(
    padded_image: &DynamicImage,
    padding: PixelPoint,
    cell: &Cell,
    target_directory: &String,
    map_cutout: &MapCutout,
) -> PathBuf {
    //The cell was created for an image that wasn't yet padded, so the padding offsets the cell.
    //The cell is centered by default, which means half the map cutout's size separates the cell from
    //the cutout's top-left corner.
    let cell_center_offset = padding - map_cutout.zoomed_in_map_image_size * 0.5;
    let mut cutout_top_left_corner = cell.center_point + cell_center_offset;
    //Cutouts for cells on or near the boundary might extend past the bounds of the padded image
    //if the cell stays centered. If this is the case, adjust cutout so that it fits inside the image,
    //which has the effect of no longer centering the cutout on the cell.
    let fit_in_bounds_adjustment = PixelPoint {
        x: adjustment_to_fit(cutout_top_left_corner.x, 0, padded_image.width() as i32),
        y: adjustment_to_fit(cutout_top_left_corner.y, 0, padded_image.height() as i32),
    };
    cutout_top_left_corner = cutout_top_left_corner + fit_in_bounds_adjustment;
    let mut cutout_image = padded_image.crop_imm(
        cutout_top_left_corner.x as u32,
        cutout_top_left_corner.y as u32,
        map_cutout.zoomed_in_map_image_size.x as u32,
        map_cutout.zoomed_in_map_image_size.y as u32,
    );

    //We highlight the cell owning the cutout by tracing its bounding polygon.
    //The coordinates of the cutout are not the coordinates of the cell anymore. The cell is now
    //centered on the center of the cutout minus the fit_in_bounds_adjustment.
    let center_of_cutout = map_cutout.zoomed_in_map_image_size * 0.5;
    let new_cell_center = center_of_cutout - fit_in_bounds_adjustment;
    let offset_from_old_cell_coordinates = new_cell_center - cell.center_point;
    let offset_polygon = cell
        .bounding_polygon
        .offset_by(offset_from_old_cell_coordinates);
    let mut previous_point = offset_polygon.points.last().unwrap();
    offset_polygon.points.iter().for_each(|pixel_point| {
        draw_line_segment_mut(
            &mut cutout_image,
            (previous_point.x as f32, previous_point.y as f32),
            (pixel_point.x as f32, pixel_point.y as f32),
            map_cutout.cell_outline_color,
        );
        previous_point = pixel_point;
    });

    //Finally, save the thing.
    let mut path = PathBuf::new();
    path.push(target_directory);
    path.push(cell.coordinate.clone() + ".png");
    cutout_image
        .save_with_format(&path, ImageFormat::Png)
        .unwrap();
    path
}

//TODO: and tests, as usual
/// Returns the adjustment that must be added to [val] so that it satisfies
///
/// [greater_than_or_equal] <= [val] < [less_than]
fn adjustment_to_fit(val: i32, greater_than_or_equal: i32, less_than: i32) -> i32 {
    if val < greater_than_or_equal {
        return greater_than_or_equal - val;
    } else if val >= less_than {
        return val - less_than - 1;
    }
    0
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

fn resolve_config(
    skip_config: Option<SkipEmptyCellsConfig>,
    cutout_config: Option<MapCutoutConfig>,
) -> (SkipEmptyCells, MapCutout) {
    let mut skip_empty_cells = SkipEmptyCells {
        skipping_enabled: true,
        polygon_multiplier: 0.3,
        //White
        empty_color_rgba: Rgba::from([255, 255, 255, 1]),
    };
    let mut map_cutout = MapCutout {
        zoomed_in_map_image_size: PixelPoint { x: 325, y: 340 },
        minimum_map_margin: PixelPoint { x: 50, y: 50 },
        //Bright yellow
        cell_outline_color: Rgba::from([255, 255, 0, 1]),
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
struct SkipEmptyCells {
    skipping_enabled: bool,
    polygon_multiplier: f32,
    empty_color_rgba: Rgba<u8>,
}

/// A [MapCutoutConfig] realized with default values where needed.
struct MapCutout {
    zoomed_in_map_image_size: PixelPoint,
    minimum_map_margin: PixelPoint,
    cell_outline_color: Rgba<u8>,
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
        use crate::image_handling::IsOnLine::{InvalidLine, LeftOfLine, OnLine, RightOfLine};

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
