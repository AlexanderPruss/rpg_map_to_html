use crate::PixelPoint;
use crate::geometry::{Cell, CellMap};
use crate::image_handling::empty_cell_detection::is_cell_empty;
use crate::image_handling::image_config::{MapCutout, SkipEmptyCells};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct CutoutImageMap {
    cutout_image_paths_by_cell_coordinate: HashMap<String, PathBuf>,
}

/// Cuts out the zoomed-in map images that are displayed on the details page of any given cell.
///
/// The map is padded to a minimum margin if needed in order to make the cutouts for cells on the edge
/// look nicer.
///
/// The bounding polygon is drawn around each cell in its own cutout.
pub fn create_cutout_images(
    cell_map: CellMap,
    original_image: &DynamicImage,
    image_margins: PixelPoint,
    target_directory: PathBuf,
    skip_empty_cells: SkipEmptyCells,
    map_cutout: MapCutout,
) -> CutoutImageMap {
    let empty_color = Rgba::from(skip_empty_cells.empty_color_rgba);
    let (padded_image, padding) = pad_image(
        original_image,
        image_margins,
        map_cutout.minimum_map_margin,
        skip_empty_cells.empty_color_rgba,
    );
    CutoutImageMap {
        cutout_image_paths_by_cell_coordinate: cell_map
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
                        padding,
                        cell,
                        &target_directory,
                        &map_cutout,
                    ),
                )
            })
            .collect(),
    }
}

/// Ensures that the image has a margin of at least [minimum_margin], creating/extending the
/// image's margin and filling it with the [empty_color] if necessary.
///
/// Returns the padded image as well as the padding added.
fn pad_image(
    image: &DynamicImage,
    current_margin: PixelPoint,
    minimum_margin: PixelPoint,
    empty_color: Rgba<u8>,
) -> (DynamicImage, PixelPoint) {
    if current_margin.x >= minimum_margin.x && current_margin.y >= minimum_margin.y {
        return (image.clone(), PixelPoint { x: 0, y: 0 });
    }
    let additional_padding_needed = PixelPoint {
        x: minimum_margin.y - current_margin.x,
        y: minimum_margin.y - current_margin.y,
    };
    let mut padded_buffer = ImageBuffer::from_pixel(
        image.width() + 2 * additional_padding_needed.x as u32,
        image.height() + 2 * additional_padding_needed.y as u32,
        empty_color,
    );
    image::imageops::overlay(
        &mut padded_buffer,
        image,
        additional_padding_needed.x as i64,
        additional_padding_needed.y as i64,
    );
    (DynamicImage::from(padded_buffer), additional_padding_needed)
}

/// Creates a cutout map image for a given cell. The cell is centered in this cutout as much as possible,
/// its polygon is outlined, and the image is saved.
fn create_and_save_cutout_map_image(
    padded_image: &DynamicImage,
    padding: PixelPoint,
    cell: &Cell,
    target_directory: &PathBuf,
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
    offset_polygon.draw(&mut cutout_image, map_cutout.cell_outline_color);

    //Finally, save the thing.
    let mut path = PathBuf::new();
    path.push(target_directory);
    path.push(cell.coordinate.clone() + ".png");
    cutout_image
        .save_with_format(&path, ImageFormat::Png)
        .unwrap();
    path
}

/// Returns the adjustment that must be added to [val] so that it satisfies
///
/// [greater_than_or_equal] <= [val] < [less_than]
fn adjustment_to_fit(val: i32, greater_than_or_equal: i32, less_than: i32) -> i32 {
    if val < greater_than_or_equal {
        return greater_than_or_equal - val;
    } else if val >= less_than {
        return less_than - 1 - val;
    }
    0
}

#[cfg(test)]
mod test {

    mod create_cutout_images {
        use crate::PixelPoint;
        use crate::geometry::hexagons::fixtures::{FourByFour, ToSnapshot};
        use crate::image_handling::image_config::{MapCutout, SkipEmptyCells};
        use crate::image_handling::map_cutout::create_cutout_images;
        use crate::image_handling::test::fixtures::FourByFourImages;
        use image::Rgba;

        #[test]
        fn creates_cutout_images_for_non_empty_cells() {
            let map_cutout = MapCutout {
                zoomed_in_map_image_size: PixelPoint { x: 175, y: 175 },
                minimum_map_margin: PixelPoint { x: 20, y: 20 },
                cell_outline_color: Rgba::from([255u8, 0, 0, 255]),
            };
            let skip_empty_cells = SkipEmptyCells {
                skipping_enabled: true,
                polygon_multiplier: 0.3,
                empty_color_rgba: Rgba::from([255u8, 255, 255, 255]),
            };
            let image = FourByFourImages::Standardized.load_image();
            let snapshot = FourByFour::Standardized.to_snapshot();

            let mut target_directory = FourByFourImages::Standardized.get_test_cases_path();
            target_directory.push("cutout_images/result");
            let mut expected_path = FourByFourImages::Standardized.get_test_cases_path();
            expected_path.push("cutout_images/expected");

            let cutout_image_map = create_cutout_images(
                snapshot.cell_map,
                &image,
                PixelPoint { x: 0, y: 0 },
                target_directory,
                skip_empty_cells,
                map_cutout,
            );

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
            assert_eq!(
                filled_coordinates.len(),
                cutout_image_map.cutout_image_paths_by_cell_coordinate.len()
            );
            filled_coordinates.iter().for_each(|coordinate| {
                let cutout_image_path = cutout_image_map
                    .cutout_image_paths_by_cell_coordinate
                    .get(coordinate)
                    .unwrap();
                let cutout_image = image::ImageReader::open(cutout_image_path)
                    .unwrap()
                    .decode()
                    .unwrap();
                let mut expected_filename = coordinate.clone();
                expected_filename.push_str(".png");
                expected_path.push(expected_filename);
                let expected_image = image::ImageReader::open(expected_path.clone())
                    .unwrap()
                    .decode()
                    .unwrap();
                expected_path.push("..");
                assert_eq!(expected_image, cutout_image);
            });
        }
    }

    mod pad_image {
        use crate::PixelPoint;
        use crate::image_handling::map_cutout::pad_image;
        use crate::image_handling::test::fixtures::FourByFourImages;
        use image::Rgba;

        #[test]
        fn returns_image_unchanged_if_the_margin_already_suffices() {
            let image = FourByFourImages::Standardized.load_image();
            let current_margin = PixelPoint { x: 10, y: 10 };
            let minimum_margin = PixelPoint { x: 10, y: 10 };
            let white = Rgba::from([0u8, 0, 0, 255]);

            let (padded_image, added_margin) =
                pad_image(&image, current_margin, minimum_margin, white);

            assert_eq!(image, padded_image);
            assert_eq!(PixelPoint { x: 0, y: 0 }, added_margin);
        }

        #[test]
        fn pads_the_image_as_much_as_needed() {
            let image = FourByFourImages::Standardized.load_image();
            let current_margin = PixelPoint { x: 10, y: 10 };
            let minimum_margin = PixelPoint { x: 20, y: 20 };
            let green = Rgba::from([0u8, 255, 0, 255]);
            let mut expected_path = FourByFourImages::Standardized.get_test_cases_path();
            expected_path.push("pad_image/expected.png");
            let expected = image::ImageReader::open(expected_path)
                .unwrap()
                .decode()
                .unwrap();

            let (padded_image, added_margin) =
                pad_image(&image, current_margin, minimum_margin, green);

            assert_eq!(expected, padded_image);
            assert_eq!(PixelPoint { x: 10, y: 10 }, added_margin);
        }
    }

    mod create_and_save_cutout_map_image {
        use crate::PixelPoint;
        use crate::geometry::hexagons::fixtures::{FourByFour, ToSnapshot};
        use crate::image_handling::image_config::MapCutout;
        use crate::image_handling::map_cutout::{create_and_save_cutout_map_image, pad_image};
        use crate::image_handling::test::fixtures::FourByFourImages;
        use image::Rgba;

        fn test_config() -> MapCutout {
            MapCutout {
                zoomed_in_map_image_size: PixelPoint { x: 175, y: 175 },
                minimum_map_margin: PixelPoint { x: 20, y: 20 },
                cell_outline_color: Rgba::from([255u8, 0, 0, 255]),
            }
        }

        fn cutout_test_case(coordinate: String, test_case_name: String) {
            let config = test_config();
            let image = FourByFourImages::Standardized.load_image();
            let green = Rgba::from([0u8, 255, 0, 255]);
            let (padded_image, padding) = pad_image(
                &image,
                PixelPoint { x: 0, y: 0 },
                config.minimum_map_margin,
                green,
            );
            let snapshot = FourByFour::Standardized.to_snapshot();
            let cell = snapshot
                .cell_map
                .cells_by_coordinate
                .get(&coordinate)
                .unwrap();
            let mut target_directory = FourByFourImages::Standardized.get_test_cases_path();
            target_directory.push("single_cutout_image");
            target_directory.push(test_case_name);
            let mut expected_path = target_directory.clone();
            target_directory.push("result");
            expected_path.push("expected.png");
            let expected = image::ImageReader::open(expected_path)
                .unwrap()
                .decode()
                .unwrap();

            let cutout_image_path = create_and_save_cutout_map_image(
                &padded_image,
                padding,
                cell,
                &target_directory,
                &config,
            );
            let cutout_image = image::ImageReader::open(cutout_image_path)
                .unwrap()
                .decode()
                .unwrap();

            assert_eq!(expected, cutout_image);
        }

        #[test]
        fn creates_cutouts_centered_on_the_cell_for_cells_in_the_middle() {
            cutout_test_case("001.001".to_string(), "central_cell".to_string());
        }

        #[test]
        fn creates_cutouts_offset_from_the_cell_for_cells_on_the_side() {
            cutout_test_case("000.000".to_string(), "corner_cell".to_string());
        }

        #[test]
        fn creates_cutouts_offset_from_the_cell_for_corners() {
            cutout_test_case("001.003".to_string(), "edge_cell".to_string());
        }
    }

    mod adjustment_to_fit {
        use crate::image_handling::map_cutout::adjustment_to_fit;

        #[test]
        fn returns_the_positive_offset_if_the_value_is_smaller_than_the_lower_bound() {
            let adjustment = adjustment_to_fit(10, 15, 20);
            assert_eq!(5, adjustment);
        }

        #[test]
        fn returns_zero_if_the_value_is_equal_to_the_lower_bound() {
            let adjustment = adjustment_to_fit(15, 15, 20);
            assert_eq!(0, adjustment);
        }

        #[test]
        fn returns_zero_if_the_value_is_between_the_bounds() {
            let adjustment = adjustment_to_fit(17, 15, 20);
            assert_eq!(0, adjustment);
        }

        #[test]
        fn returns_negative_one_if_the_value_is_equal_to_the_upper_bound() {
            let adjustment = adjustment_to_fit(20, 15, 20);
            assert_eq!(-1, adjustment);
        }

        #[test]
        fn returns_the_negative_offset_if_the_value_is_greater_than_the_upper_bound() {
            let adjustment = adjustment_to_fit(25, 15, 20);
            assert_eq!(-6, adjustment);
        }
    }
}
