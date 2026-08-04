use crate::geometry::CellMap;
use crate::image_handling::IMAGE_SUBDIRECTORY;
use crate::{PixelBox, PixelPoint};
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TableOfContentsMapImage {
    pub filename: String,
    pub size: PixelPoint,
    pub offset: PixelPoint,
    pub coordinates_contained: HashSet<String>,
}

/// As [save_table_of_contents_map_images_with_cache], but without caching.
pub fn save_table_of_contents_map_images(
    target_directory: &PathBuf,
    original_image: &DynamicImage,
    max_image_size: &PixelPoint,
    cell_map: &CellMap,
) -> Vec<TableOfContentsMapImage> {
    save_table_of_contents_map_images_with_cache(
        target_directory,
        original_image,
        max_image_size,
        cell_map,
        &HashMap::new(),
    )
}

/// The start of the HTML document contains a table-of-contents in image form. The original map
/// is presented, and all of its non-empty cells can be clicked on to navigate to that cell's page.
///
/// However, the original image might be too large to fit onto one page. If this is the case, the image
/// is split up into smaller, overlapping images of identical size, and a page is drawn for each of these images.
///
/// If cached values are provided, the calculation prefers cached values when they are plausible.
pub fn save_table_of_contents_map_images_with_cache(
    target_directory: &PathBuf,
    original_image: &DynamicImage,
    max_image_size: &PixelPoint,
    cell_map: &CellMap,
    cached_by_filename: &HashMap<String, TableOfContentsMapImage>,
) -> Vec<TableOfContentsMapImage> {
    if max_image_size.x <= 1 || max_image_size.y <= 1 {
        panic!("Max image sizes for table of contents images have to be larger than 1x1.")
    }
    let minimum_overlap = PixelPoint {
        x: 100.min(max_image_size.x / 2),
        y: 100.min(max_image_size.y / 2),
    };
    let offset_per_shift = PixelPoint {
        x: max_image_size.x - minimum_overlap.x,
        y: max_image_size.y - minimum_overlap.y,
    };
    let mut current_offset = PixelPoint { x: 0, y: 0 };
    let saved_image_size = PixelPoint {
        x: max_image_size.x.min(original_image.width() as i32),
        y: max_image_size.y.min(original_image.height() as i32),
    };
    let mut image_directory = PathBuf::from(target_directory);
    image_directory.push(IMAGE_SUBDIRECTORY);
    let mut table_of_contents_images: Vec<TableOfContentsMapImage> = vec![];
    loop {
        let max_drawable = current_offset + *max_image_size;
        let x_maxed = max_drawable.x >= original_image.width() as i32;
        let y_maxed = max_drawable.y >= original_image.height() as i32;
        let mut effective_offset = current_offset;
        //If we reach the end of the image, just display as much of the image as we can
        if x_maxed {
            effective_offset.x = original_image.width() as i32 - saved_image_size.x;
        }
        if y_maxed {
            effective_offset.y = original_image.height() as i32 - saved_image_size.y;
        }

        let coordinates_contained = filter_coordinates_contained(
            cell_map,
            PixelBox {
                top_left_corner: effective_offset,
                bottom_right_corner: effective_offset + *max_image_size,
            },
        );
        let filename = format!(
            "table_of_contents_{}_{}.png",
            effective_offset.x, effective_offset.y
        );
        let cache_hit = cached_by_filename.get(&filename);
        let table_of_contents_image = match cache_hit {
            None => save(
                &image_directory,
                original_image,
                filename,
                effective_offset,
                saved_image_size,
                coordinates_contained,
            ),
            Some(cached_image) => cached_image.clone(),
        };
        table_of_contents_images.push(table_of_contents_image);

        //We're done once we reach the bottom-right corner.
        if x_maxed && y_maxed {
            break;
        }
        //Shift to the right, or shift down to the next row if we've reached the horizontal end of the image
        if x_maxed {
            current_offset = PixelPoint {
                x: 0,
                y: current_offset.y + offset_per_shift.y,
            }
        } else {
            current_offset = PixelPoint {
                x: current_offset.x + offset_per_shift.x,
                y: current_offset.y,
            }
        }
    }
    table_of_contents_images
}

fn save(
    target_directory: &PathBuf,
    image: &DynamicImage,
    filename: String,
    offset: PixelPoint,
    size: PixelPoint,
    coordinates_contained: HashSet<String>,
) -> TableOfContentsMapImage {
    let mut path = PathBuf::from(target_directory);
    path.push(&filename);

    let cropped = &image.crop_imm(
        offset.x as u32,
        offset.y as u32,
        size.x as u32,
        size.y as u32,
    );
    cropped.save_with_format(&path, ImageFormat::Png).unwrap();

    TableOfContentsMapImage {
        filename,
        size,
        offset,
        coordinates_contained,
    }
}

fn filter_coordinates_contained(cell_map: &CellMap, pixel_box: PixelBox) -> HashSet<String> {
    cell_map
        .cells_by_coordinate
        .iter()
        .filter(|(_coordinate, cell)| {
            cell.center_point.x <= pixel_box.bottom_right_corner.x
                && cell.center_point.x >= pixel_box.top_left_corner.x
                && cell.center_point.y <= pixel_box.bottom_right_corner.y
                && cell.center_point.y >= pixel_box.top_left_corner.y
        })
        .map(|(coordinate, _cell)| coordinate.clone())
        .collect()
}

#[cfg(test)]
mod test {
    mod save_table_of_contents_map_images {
        use crate::geometry::{BoundingPolygon, Cell, CellMap};
        use crate::image_handling::IMAGE_SUBDIRECTORY;
        use crate::image_handling::table_of_contents::{
            TableOfContentsMapImage, filter_coordinates_contained,
            save_table_of_contents_map_images,
        };
        use crate::image_handling::test::fixtures::get_test_resources_path;
        use crate::{PixelBox, PixelPoint};
        use std::collections::HashMap;
        use std::path::PathBuf;

        fn two_fifty_px_by_two_fifty_px_cell_map(upper_bound: PixelPoint) -> CellMap {
            let mut cells_by_coordinate: HashMap<String, Cell> = HashMap::new();
            for x in 1..=upper_bound.x / 250 {
                for y in 1..=upper_bound.y / 250 {
                    let coordinate = format!("{}.{}", x * 250, y * 250);
                    cells_by_coordinate.insert(
                        coordinate.clone(),
                        Cell {
                            coordinate,
                            neighbor_coordinates: Default::default(),
                            center_point: PixelPoint { x, y },
                            bounding_polygon: BoundingPolygon { points: vec![] },
                            inscribed_rectangle: Default::default(),
                        },
                    );
                }
            }
            CellMap {
                cells_by_coordinate,
            }
        }

        fn create_table_of_contents_test_case(
            test_case: PathBuf,
            original_filename: String,
            expected_size: PixelPoint,
            expected_offsets: Vec<PixelPoint>,
        ) {
            let mut setup_file_path = PathBuf::from(&test_case);
            setup_file_path.push("setup");
            setup_file_path.push(original_filename);
            let original_map_image = image::open(setup_file_path).unwrap();

            let cell_map = two_fifty_px_by_two_fifty_px_cell_map(PixelPoint {
                x: original_map_image.width() as i32,
                y: original_map_image.height() as i32,
            });
            let mut target_directory = PathBuf::from(&test_case);
            target_directory.push("result");

            let toc_images = save_table_of_contents_map_images(
                &target_directory,
                &original_map_image,
                &PixelPoint { x: 900, y: 1200 },
                &cell_map,
            );

            let mut expected_directory = PathBuf::from(&test_case);
            expected_directory.push("expected");
            let expected_toc_images: Vec<TableOfContentsMapImage> = expected_offsets
                .into_iter()
                .map(|offset| TableOfContentsMapImage {
                    size: expected_size,
                    filename: format!("table_of_contents_{}_{}.png", offset.x, offset.y),
                    offset,
                    coordinates_contained: filter_coordinates_contained(
                        &cell_map,
                        PixelBox {
                            top_left_corner: offset,
                            bottom_right_corner: offset + expected_size,
                        },
                    ),
                })
                .collect();
            assert_eq!(toc_images.len(), expected_toc_images.len());
            let mut expected_toc_image_iter = expected_toc_images.iter();
            for toc_image in toc_images {
                let mut result_filepath = PathBuf::from(&target_directory);
                result_filepath.push(IMAGE_SUBDIRECTORY);
                result_filepath.push(&toc_image.filename);
                let mut expected_filepath = PathBuf::from(&expected_directory);
                expected_filepath.push(&toc_image.filename);
                let resulting_image = image::open(result_filepath).unwrap();
                let expected_image = image::open(expected_filepath).unwrap();
                assert_eq!(expected_image, resulting_image);
                assert_eq!(*expected_toc_image_iter.next().unwrap(), toc_image);
            }
        }

        #[test]
        fn creates_a_single_image_for_small_maps() {
            let mut test_case = get_test_resources_path();
            test_case.push("table_of_contents_images/small_map");
            create_table_of_contents_test_case(
                test_case,
                "map_600_600.png".to_string(),
                PixelPoint { x: 600, y: 600 },
                vec![PixelPoint { x: 0, y: 0 }],
            )
        }

        #[test]
        fn creates_multiple_images_ordered_by_row() {
            let mut test_case = get_test_resources_path();
            test_case.push("table_of_contents_images/three_by_three_map");
            create_table_of_contents_test_case(
                test_case,
                "map_2400_3000.png".to_string(),
                PixelPoint { x: 900, y: 1200 },
                vec![
                    PixelPoint { x: 0, y: 0 },
                    PixelPoint { x: 800, y: 0 },
                    PixelPoint { x: 1500, y: 0 },
                    PixelPoint { x: 0, y: 1100 },
                    PixelPoint { x: 800, y: 1100 },
                    PixelPoint { x: 1500, y: 1100 },
                    PixelPoint { x: 0, y: 1800 },
                    PixelPoint { x: 800, y: 1800 },
                    PixelPoint { x: 1500, y: 1800 },
                ],
            )
        }

        #[test]
        fn handles_maps_that_are_exactly_the_max_size_correctly() {
            let mut test_case = get_test_resources_path();
            test_case.push("table_of_contents_images/two_by_one_exact_fit");
            create_table_of_contents_test_case(
                test_case,
                "map_1700_1200.png".to_string(),
                PixelPoint { x: 900, y: 1200 },
                vec![PixelPoint { x: 0, y: 0 }, PixelPoint { x: 800, y: 0 }],
            )
        }

        mod caching {
            use crate::PixelPoint;
            use crate::image_handling::IMAGE_SUBDIRECTORY;
            use crate::image_handling::table_of_contents::test::save_table_of_contents_map_images::two_fifty_px_by_two_fifty_px_cell_map;
            use crate::image_handling::table_of_contents::{
                TableOfContentsMapImage, save_table_of_contents_map_images_with_cache,
            };
            use crate::image_handling::test::fixtures::get_test_resources_path;
            use std::collections::{HashMap, HashSet};
            use std::fs;
            use std::path::PathBuf;

            #[test]
            fn only_saves_images_if_no_cached_image_is_present() {
                let mut test_case = get_test_resources_path();
                test_case.push("table_of_contents_images/three_by_three_map");
                let original_filename = "map_2400_3000.png";
                let mut setup_file_path = PathBuf::from(&test_case);
                setup_file_path.push("setup");
                setup_file_path.push(original_filename);
                let original_map_image = image::open(setup_file_path).unwrap();
                let expected_filenames: Vec<String> = vec![
                    PixelPoint { x: 0, y: 0 },
                    PixelPoint { x: 800, y: 0 },
                    PixelPoint { x: 1500, y: 0 },
                    PixelPoint { x: 0, y: 1100 },
                    PixelPoint { x: 800, y: 1100 },
                    PixelPoint { x: 1500, y: 1100 },
                    PixelPoint { x: 0, y: 1800 },
                    PixelPoint { x: 800, y: 1800 },
                    PixelPoint { x: 1500, y: 1800 },
                ]
                .iter()
                .map(|point| format!("table_of_contents_{}_{}.png", point.x, point.y))
                .collect();

                let cell_map = two_fifty_px_by_two_fifty_px_cell_map(PixelPoint {
                    x: original_map_image.width() as i32,
                    y: original_map_image.height() as i32,
                });
                let mut target_directory = PathBuf::from(&test_case);
                target_directory.push("cached_result");
                //Delete any pngs floating around the target directory
                fs::read_dir(&target_directory)
                    .unwrap()
                    .filter(|file| {
                        file.as_ref()
                            .unwrap()
                            .file_name()
                            .to_str()
                            .unwrap()
                            .ends_with(".png")
                    })
                    .for_each(|file| fs::remove_file(file.unwrap().path()).unwrap());

                let cached_by_filename = HashMap::from([
                    (
                        "table_of_contents_0_1800.png".to_string(),
                        TableOfContentsMapImage {
                            filename: "table_of_contents_0_1800.png".to_string(),
                            size: PixelPoint { x: 1, y: 2 },
                            offset: PixelPoint { x: 3, y: 4 },
                            coordinates_contained: Default::default(),
                        },
                    ),
                    (
                        "table_of_contents_800_0.png".to_string(),
                        TableOfContentsMapImage {
                            filename: "table_of_contents_800_0.png".to_string(),
                            size: PixelPoint { x: 5, y: 6 },
                            offset: PixelPoint { x: 7, y: 8 },
                            coordinates_contained: Default::default(),
                        },
                    ),
                ]);
                let cached_filenames = HashSet::from([
                    "table_of_contents_0_1800.png".to_string(),
                    "table_of_contents_800_0.png".to_string(),
                ]);

                let toc_images = save_table_of_contents_map_images_with_cache(
                    &target_directory,
                    &original_map_image,
                    &PixelPoint { x: 900, y: 1200 },
                    &cell_map,
                    &cached_by_filename,
                );

                //Ensure no cached files were created.
                fs::read_dir(&target_directory)
                    .unwrap()
                    .filter(|file| {
                        let filename = file.as_ref().unwrap().file_name();
                        let filename_option = filename.to_str().unwrap().strip_suffix(".png");
                        filename_option.is_some()
                            && cached_filenames.contains(filename_option.unwrap())
                    })
                    .for_each(|file| {
                        panic!(
                            "The file {} should not exist, it should have been cached",
                            file.unwrap().file_name().to_str().unwrap()
                        )
                    });

                let mut expected_directory = PathBuf::from(&test_case);
                expected_directory.push("expected");
                assert_eq!(toc_images.len(), 9);
                let mut expected_path = PathBuf::from(&target_directory);
                expected_path.push("../expected");
                for filename in expected_filenames {
                    if cached_filenames.contains(&filename) {
                        let computed = toc_images
                            .iter()
                            .find(|toc_image| toc_image.filename == filename)
                            .unwrap();
                        let expected = cached_by_filename.get(&filename).unwrap();
                        assert_eq!(expected, computed)
                    } else {
                        let mut toc_image_path = PathBuf::from(&target_directory);
                        toc_image_path.push(IMAGE_SUBDIRECTORY);
                        toc_image_path.push(&filename);
                        let cutout_image = image::ImageReader::open(toc_image_path)
                            .unwrap()
                            .decode()
                            .unwrap();

                        expected_path.push(&filename);
                        let expected_image = image::ImageReader::open(expected_path.clone())
                            .unwrap()
                            .decode()
                            .unwrap();
                        expected_path.push("..");
                        assert_eq!(expected_image, cutout_image);
                    }
                }
            }
        }
    }

    mod filter_coordinates_contained {
        use crate::geometry::{BoundingPolygon, Cell, CellMap};
        use crate::image_handling::table_of_contents::filter_coordinates_contained;
        use crate::{PixelBox, PixelPoint};
        use std::collections::{HashMap, HashSet};

        #[test]
        fn filters_coordinates_that_are_not_contained_in_the_box() {
            let container = PixelBox {
                top_left_corner: PixelPoint { x: 6, y: 6 },
                bottom_right_corner: PixelPoint { x: 10, y: 10 },
            };
            let mut cells_by_coordinate: HashMap<String, Cell> = HashMap::new();
            for x in 0..=15 {
                for y in 0..=15 {
                    let coordinate = format!("{}.{}", x, y);
                    cells_by_coordinate.insert(
                        coordinate.clone(),
                        Cell {
                            coordinate,
                            neighbor_coordinates: Default::default(),
                            center_point: PixelPoint { x, y },
                            bounding_polygon: BoundingPolygon { points: vec![] },
                            inscribed_rectangle: Default::default(),
                        },
                    );
                }
            }
            let mut expected: HashSet<String> = HashSet::new();
            for x in 6..=10 {
                for y in 6..=10 {
                    expected.insert(format!("{}.{}", x, y));
                }
            }

            let coordinates = filter_coordinates_contained(
                &CellMap {
                    cells_by_coordinate,
                },
                container,
            );
            assert_eq!(expected, coordinates);
        }
    }
}
