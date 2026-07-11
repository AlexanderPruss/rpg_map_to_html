use std::collections::HashMap;
use crate::{PixelBox, PixelPoint};
use crate::geometry::{Cell, CellMap};
use crate::image_handling::empty_cell_detection::is_cell_empty;
use crate::image_handling::image_config::{ImageHandling, SkipEmptyCells};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::caching::ChangedImage;
use crate::image_handling::IMAGE_SUBDIRECTORY;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CutoutImage {
    pub coordinate: String,
    pub offset_from_original_image: PixelPoint,
    pub image_size: PixelPoint,
}

pub fn save_cutout_images(
    target_directory: &PathBuf,
    cell_map: &CellMap,
    original_image: &DynamicImage,
    image_margins: PixelPoint,
    skip_empty_cells: &SkipEmptyCells,
    image_handling: &ImageHandling,
) -> Vec<CutoutImage> {
    save_cutout_images_with_cache(
        target_directory,
        cell_map,
        original_image,
        image_margins,
        skip_empty_cells,
        image_handling,
        vec![],
        &ChangedImage::AllNew
    )
}

/// Cuts out the zoomed-in map images that are displayed on the details page of any given cell.
///
/// The map is padded to a minimum margin if needed in order to make the cutouts for cells on the edge
/// look nicer.
///
/// The bounding polygon is drawn around each cell in its own cutout.
pub fn save_cutout_images_with_cache(
    target_directory: &PathBuf,
    cell_map: &CellMap,
    original_image: &DynamicImage,
    image_margins: PixelPoint,
    skip_empty_cells: &SkipEmptyCells,
    image_handling: &ImageHandling,
    cached_cutout_images: Vec<CutoutImage>,
    changed_image: &ChangedImage
) -> Vec<CutoutImage> {
    let cached_images_by_coordinate : HashMap<String, CutoutImage> = cached_cutout_images.into_iter().map(|image| (image.coordinate.clone(), image)).collect();

    let empty_color = Rgba::from(skip_empty_cells.empty_color_rgba);
    let (padded_image, padding) = pad_image(
        original_image,
        image_margins,
        image_handling.minimum_map_margin,
        skip_empty_cells.empty_color_rgba,
    );
    cell_map
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
            match check_cache_hit(coordinate, &cached_images_by_coordinate, changed_image) {
                None => save_cutout_map_image(
                    &padded_image,
                    padding,
                    cell,
                    target_directory,
                    &image_handling,
                ),
                Some(cached) => cached
            }
        })
        .collect()
}

/// Checks if we can use an image from the cache.
///
/// We use a cached image if
/// * A cached image exists
/// * The image is not brand new
/// * If the image has changes, none of the change pixels overlap with the cached image
fn check_cache_hit(
    coordinate: &String,
    cached_images_by_coordinate : &HashMap<String, CutoutImage>,
    changed_image: &ChangedImage,
) -> Option<CutoutImage> {
    let cached_cutout_image = cached_images_by_coordinate.get(coordinate)?;
    let changed_pixels =  match changed_image{
        ChangedImage::NoChanges => return Some(cached_cutout_image.clone()),
        ChangedImage::AllNew => return None,
        ChangedImage::Changes { bounding_box } => bounding_box
    };
    let pixels_of_image = PixelBox{
        top_left_corner: cached_cutout_image.offset_from_original_image,
        bottom_right_corner: cached_cutout_image.offset_from_original_image + cached_cutout_image.image_size
    };
    if pixels_of_image.intersects(changed_pixels) {
        return None
    }
    Some(cached_cutout_image.clone())
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
fn save_cutout_map_image(
    padded_image: &DynamicImage,
    padding: PixelPoint,
    cell: &Cell,
    target_directory: &PathBuf,
    image_handling: &ImageHandling,
) -> CutoutImage {
    let mut path = PathBuf::new();
    path.push(target_directory);
    path.push(IMAGE_SUBDIRECTORY);
    path.push(cell.coordinate.clone() + ".png");
    let padded_image_size = PixelPoint {
        x: padded_image.width() as i32,
        y: padded_image.height() as i32,
    };
    if padded_image_size.x <= image_handling.zoomed_in_map_image_size.x
        && padded_image_size.y <= image_handling.zoomed_in_map_image_size.y
    {
        padded_image
            .save_with_format(&path, ImageFormat::Png)
            .unwrap();
        return CutoutImage {
            coordinate: cell.coordinate.clone(),
            offset_from_original_image: padding,
            image_size: padded_image_size,
        };
    }

    //The cell was created for an image that wasn't yet padded, so the padding offsets the cell.
    //The cell is centered by default, which means half the map cutout's size separates the cell from
    //the cutout's top-left corner.
    let cell_center_offset = padding - image_handling.zoomed_in_map_image_size * 0.5;
    let mut cutout_top_left_corner = cell.center_point + cell_center_offset;
    //Cutouts for cells on or near the boundary might extend past the bounds of the padded image
    //if the cell stays centered. If this is the case, adjust cutout so that it fits inside the image,
    //which has the effect of no longer centering the cutout on the cell.
    let original_top_left_corner = cutout_top_left_corner;
    cutout_top_left_corner = fit_to_bounds(
        cutout_top_left_corner,
        &image_handling.zoomed_in_map_image_size,
        &padded_image_size,
    );
    let fit_in_bounds_adjustment = cutout_top_left_corner - original_top_left_corner;
    let mut cutout_image = padded_image.crop_imm(
        cutout_top_left_corner.x as u32,
        cutout_top_left_corner.y as u32,
        image_handling.zoomed_in_map_image_size.x as u32,
        image_handling.zoomed_in_map_image_size.y as u32,
    );

    //We highlight the cell owning the cutout by tracing its bounding polygon.
    //The coordinates of the cutout are not the coordinates of the cell anymore. The cell is now
    //centered on the center of the cutout minus the fit_in_bounds_adjustment.
    let center_of_cutout = image_handling.zoomed_in_map_image_size * 0.5;
    let new_cell_center = center_of_cutout - fit_in_bounds_adjustment;
    let offset_from_old_cell_coordinates = new_cell_center - cell.center_point;
    let offset_polygon = cell
        .bounding_polygon
        .offset_by(offset_from_old_cell_coordinates);
    offset_polygon.draw(&mut cutout_image, image_handling.cell_outline_color);

    //Finally, save the thing.
    cutout_image
        .save_with_format(&path, ImageFormat::Png)
        .unwrap();
    CutoutImage {
        coordinate: cell.coordinate.clone(),
        offset_from_original_image: offset_from_old_cell_coordinates,
        image_size: image_handling.zoomed_in_map_image_size,
    }
}

/// Moves the point so that the cutout image rectangle with this point as its top-left corner
/// fits inside the containing image.
///
/// Panics if fitting to bounds is not possible.
fn fit_to_bounds(
    mut point: PixelPoint,
    cutout_image_size: &PixelPoint,
    padded_image_size: &PixelPoint,
) -> PixelPoint {
    if point.x < 0 {
        point.x = 0;
    }
    if point.y < 0 {
        point.y = 0;
    }
    let distance_to_positive_boundary = *padded_image_size - *cutout_image_size - point;
    if distance_to_positive_boundary.x < 0 {
        point.x += distance_to_positive_boundary.x;
    }
    if distance_to_positive_boundary.y < 0 {
        point.y += distance_to_positive_boundary.y;
    }
    if point.x < 0 || point.y < 0 || point.x > padded_image_size.x || point.y > padded_image_size.y
    {
        panic!("The map was too small, it was not possible to fit a cutout image into it.")
    }
    point
}

#[cfg(test)]
mod test {

    mod save_cutout_images {
        use crate::PixelPoint;
        use crate::geometry::hexagons::fixtures::{FourByFour, ToSnapshot};
        use crate::image_handling::image_config::{ImageHandling, SkipEmptyCells};
        use crate::image_handling::map_cutout::save_cutout_images;
        use crate::image_handling::test::fixtures::FourByFourImages;
        use image::Rgba;
        use std::path::PathBuf;
        use crate::image_handling::IMAGE_SUBDIRECTORY;

        #[test]
        fn creates_cutout_images_for_non_empty_cells() {
            let image_handling = ImageHandling {
                zoomed_in_map_image_size: PixelPoint { x: 175, y: 175 },
                max_table_of_contents_map_image_size: PixelPoint { x: 0, y: 0 },
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

            let cutout_images = save_cutout_images(
                &target_directory,
                &snapshot.cell_map,
                &image,
                PixelPoint { x: 0, y: 0 },
                &skip_empty_cells,
                &image_handling,
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
            assert_eq!(filled_coordinates.len(), cutout_images.len());
            filled_coordinates.iter().for_each(|coordinate| {
                let mut cutout_image_path = PathBuf::from(&target_directory);
                cutout_image_path.push(IMAGE_SUBDIRECTORY);
                cutout_image_path.push(coordinate.clone() + ".png");
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

    mod save_cutout_image {
        use crate::PixelPoint;
        use crate::geometry::hexagons::fixtures::{FourByFour, ToSnapshot};
        use crate::image_handling::image_config::ImageHandling;
        use crate::image_handling::map_cutout::{CutoutImage, pad_image, save_cutout_map_image};
        use crate::image_handling::test::fixtures::FourByFourImages;
        use image::Rgba;
        use std::path::PathBuf;
        use crate::image_handling::IMAGE_SUBDIRECTORY;

        fn test_config() -> ImageHandling {
            ImageHandling {
                zoomed_in_map_image_size: PixelPoint { x: 175, y: 175 },
                max_table_of_contents_map_image_size: PixelPoint { x: 0, y: 0 },
                minimum_map_margin: PixelPoint { x: 20, y: 20 },
                cell_outline_color: Rgba::from([255u8, 0, 0, 255]),
            }
        }

        fn cutout_test_case(
            coordinate: String,
            test_case_name: &String,
            expected_cutout_image: CutoutImage,
        ) {
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
            expected_path.push(format!("expected-{coordinate}.png"));
            let expected = image::ImageReader::open(expected_path)
                .unwrap()
                .decode()
                .unwrap();

            let cutout_image =
                save_cutout_map_image(&padded_image, padding, cell, &target_directory, &config);
            let mut cutout_image_path = PathBuf::from(target_directory);
            cutout_image_path.push(IMAGE_SUBDIRECTORY);
            cutout_image_path.push(cell.coordinate.clone() + ".png");
            let actual_cutout_image = image::ImageReader::open(cutout_image_path)
                .unwrap()
                .decode()
                .unwrap();

            assert_eq!(expected_cutout_image, cutout_image);
            assert_eq!(expected, actual_cutout_image);
        }

        #[test]
        fn creates_cutouts_centered_on_the_cell_for_cells_in_the_middle() {
            let expected_size = test_config().zoomed_in_map_image_size;
            let test_case = "central_cell".to_string();
            cutout_test_case(
                "001.001".to_string(),
                &test_case,
                CutoutImage {
                    coordinate: "001.001".to_string(),
                    offset_from_original_image: PixelPoint { x: -38, y: -13 },
                    image_size: expected_size,
                },
            );
            cutout_test_case(
                "002.002".to_string(),
                &test_case,
                CutoutImage {
                    coordinate: "002.002".to_string(),
                    offset_from_original_image: PixelPoint { x: -113, y: -38 },
                    image_size: expected_size,
                },
            );
        }

        #[test]
        fn creates_cutouts_offset_from_the_cell_for_corners() {
            let expected_size = test_config().zoomed_in_map_image_size;
            let test_case = "corner_cell".to_string();
            //Top-left
            cutout_test_case(
                "000.000".to_string(),
                &test_case,
                CutoutImage {
                    coordinate: "000.000".to_string(),
                    offset_from_original_image: PixelPoint { x: 20, y: 20 },
                    image_size: expected_size,
                },
            );
            //Bottom-right
            cutout_test_case(
                "003.003".to_string(),
                &test_case,
                CutoutImage {
                    coordinate: "003.003".to_string(),
                    offset_from_original_image: PixelPoint { x: -170, y: -70 },
                    image_size: expected_size,
                },
            );
        }

        #[test]
        fn creates_cutouts_offset_from_the_cell_for_cells_on_the_edges() {
            let expected_size = test_config().zoomed_in_map_image_size;
            let test_case = "edge_cell".to_string();
            //top
            cutout_test_case(
                "002.000".to_string(),
                &test_case,
                CutoutImage {
                    coordinate: "002.000".to_string(),
                    offset_from_original_image: PixelPoint { x: -113, y: 20 },
                    image_size: expected_size,
                },
            );
            //left
            cutout_test_case(
                "000.002".to_string(),
                &test_case,
                CutoutImage {
                    coordinate: "000.002".to_string(),
                    offset_from_original_image: PixelPoint { x: 20, y: -38 },
                    image_size: expected_size,
                },
            );
            //bottom
            cutout_test_case(
                "003.001".to_string(),
                &test_case,
                CutoutImage {
                    coordinate: "003.001".to_string(),
                    offset_from_original_image: PixelPoint { x: -170, y: -13 },
                    image_size: expected_size,
                },
            );
            //right
            cutout_test_case(
                "001.003".to_string(),
                &test_case,
                CutoutImage {
                    coordinate: "001.003".to_string(),
                    offset_from_original_image: PixelPoint { x: -38, y: -70 },
                    image_size: expected_size,
                },
            );
        }
    }

    mod fit_to_bounds {

        //TODO - new tests here
    }
}
