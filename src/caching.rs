use crate::caching::ChangedImage::{AllNew, NoChanges};
use crate::config::{Config, LAST_USED_CONFIG};
use crate::geometry::CellMap;
use crate::image_handling::map_cutout::CutoutImage;
use crate::image_handling::table_of_contents::TableOfContentsMapImage;
use crate::{PixelBox, PixelPoint, config, geometry, image_handling};
use image::{DynamicImage, GenericImageView};
use std::collections::HashMap;
use std::path::PathBuf;

/// Contains valid cached objects; valid meaning that the caching mechanism has already verified
/// that these objects can be safely used without being recomputed.
#[derive(Debug, PartialEq)]
pub struct CachedComputedObjects {
    pub cell_map: CellMap,
    pub table_of_contents_map_images_by_filename: HashMap<String, TableOfContentsMapImage>,
    pub cutout_images_by_coordinate: HashMap<String, CutoutImage>,
}

/// Whether pixels of the map image have changed between two runs of the generator.
#[derive(Debug, PartialEq)]
enum ChangedImage {
    NoChanges,
    Changes { bounding_box: PixelBox },
    AllNew,
}

/// Attempts to recover computationally intensive objects. These can only be recovered if
///
/// * The config is unchanged from the previous run
/// * The map image is the same size as before
/// * The cached objects are present
pub fn get_cached_objects(
    config: &Config,
    map_image: &DynamicImage,
) -> Option<CachedComputedObjects> {
    let previous_config = config::parse_config(PathBuf::from(LAST_USED_CONFIG))?;
    //If the config has changed, everything could be different. Don't use the cache.
    if previous_config != *config {
        return None;
    }
    let image_filename = config
        .map_image
        .image_file
        .file_name()?
        .to_str()?
        .to_string();
    let (table_of_contents_map_images, cutout_images, previous_image) =
        image_handling::load_persisted_image_metadata(&config.target_directory, &image_filename)?;
    let changed_pixel_box = find_changed_pixels(map_image, &previous_image);
    if changed_pixel_box == AllNew {
        return None;
    }

    let cell_map = geometry::load_persisted_cell_map(&previous_config.target_directory)?;
    Some(CachedComputedObjects {
        cell_map,
        table_of_contents_map_images_by_filename: table_of_contents_map_images
            .into_iter()
            .filter(|cached| match &changed_pixel_box {
                NoChanges => true,
                ChangedImage::Changes { bounding_box } => !bounding_box.intersects(&PixelBox {
                    top_left_corner: cached.offset,
                    bottom_right_corner: cached.offset + cached.size,
                }),
                AllNew => panic!("Should never return cached values if the image is all new."),
            })
            .map(|cached| (cached.filename.clone(), cached))
            .collect(),
        cutout_images_by_coordinate: cutout_images
            .into_iter()
            .filter(|cached| match &changed_pixel_box {
                NoChanges => true,
                ChangedImage::Changes { bounding_box } => !bounding_box.intersects(&PixelBox {
                    top_left_corner: cached.offset_from_original_image,
                    bottom_right_corner: cached.offset_from_original_image + cached.image_size,
                }),
                AllNew => panic!("Should never return cached values if the image is all new."),
            })
            .map(|cached| (cached.coordinate.clone(), cached))
            .collect(),
    })
}

pub fn persist_cached_objects(
    target_directory: &PathBuf,
    config: &Config,
    image: &DynamicImage,
    cell_map: CellMap,
    table_of_contents_images: &Vec<TableOfContentsMapImage>,
    cutout_images: &Vec<CutoutImage>,
) {
    config::persist_config(&config);
    geometry::persist_cell_map_as_geometry(target_directory, cell_map);

    let image_filename = config
        .map_image
        .image_file
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    image_handling::persist_image_metadata(
        target_directory,
        &image_filename,
        image,
        table_of_contents_images,
        cutout_images,
    );
}

/// Returns a box bounding all pixels that have changed between the last two runs.
/// TODO: This should have terrible performance, can we do this with basically a matrix op instead?
/// TODO: Still worth it as-is
fn find_changed_pixels(map_image: &DynamicImage, previous_image: &DynamicImage) -> ChangedImage {
    if map_image.width() != previous_image.width() || map_image.height() != previous_image.height()
    {
        return AllNew;
    }
    let mut new_pixels = map_image.pixels();
    let mut previous_pixels = previous_image.pixels();
    let mut changed_pixel_bounding_box: Option<PixelBox> = None;
    loop {
        let new_pixel = new_pixels.next();
        let previous_pixel = previous_pixels.next();
        if new_pixel.is_none() && previous_pixel.is_none() {
            break;
        }
        if new_pixel.is_none() || previous_pixel.is_none() {
            return AllNew;
        }
        let new_pixel = new_pixel.unwrap();
        if new_pixel != previous_pixel.unwrap() {
            let x = new_pixel.0 as i32;
            let y = new_pixel.1 as i32;
            if changed_pixel_bounding_box.is_none() {
                let _ = changed_pixel_bounding_box.insert(PixelBox {
                    top_left_corner: PixelPoint { x, y },
                    bottom_right_corner: PixelPoint { x, y },
                });
            } else {
                let _ = changed_pixel_bounding_box.insert(PixelBox {
                    top_left_corner: PixelPoint::min(
                        &PixelPoint { x, y },
                        &changed_pixel_bounding_box.as_ref().unwrap().top_left_corner,
                    ),
                    bottom_right_corner: PixelPoint::max(
                        &PixelPoint { x, y },
                        &changed_pixel_bounding_box
                            .as_ref()
                            .unwrap()
                            .bottom_right_corner,
                    ),
                });
            };
        }
    }
    match changed_pixel_bounding_box {
        None => NoChanges,
        Some(bounding_box) => {
            if (bounding_box
                == PixelBox {
                    top_left_corner: PixelPoint { x: 0, y: 0 },
                    bottom_right_corner: PixelPoint {
                        x: map_image.width() as i32 - 1,
                        y: map_image.height() as i32 - 1,
                    },
                })
            {
                AllNew
            } else {
                ChangedImage::Changes { bounding_box }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::PixelPoint;
    use crate::config::{Config, MapImageConfig, LAST_USED_CONFIG};
    use crate::geometry::CellMap;
    use crate::geometry::Geometry::Hexagons;
    use crate::geometry::hexagons::fixtures::{FourByFour, ToSnapshot};
    use crate::image_handling::map_cutout::CutoutImage;
    use crate::image_handling::table_of_contents::TableOfContentsMapImage;
    use crate::image_handling::test::fixtures::FourByFourImages;
    use image::DynamicImage;
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::path::{ PathBuf};

    struct CachingTestCase {
        target_directory: PathBuf,
        config: Config,
        image: DynamicImage,
        original_image_path: PathBuf,
        cell_map: CellMap,
        table_of_contents_images: Vec<TableOfContentsMapImage>,
        table_of_contents_images_by_filename: HashMap<String, TableOfContentsMapImage>,
        cutout_images: Vec<CutoutImage>,
        cutout_images_by_coordinate: HashMap<String, CutoutImage>,
    }

    fn standard_test_case(test_case: String) -> CachingTestCase {
        let last_used_config = PathBuf::from(LAST_USED_CONFIG);
        if fs::exists(&last_used_config).unwrap() {
            fs::remove_file(last_used_config).unwrap();
        }

        let mut test_case_path = PathBuf::new();
        test_case_path.push(env!("CARGO_MANIFEST_DIR"));
        test_case_path.push("test_resources");
        test_case_path.push("caching");
        test_case_path.push(&test_case);
        let mut target_directory = PathBuf::from(&test_case_path);
        target_directory.push("cached");
        //Remove any previous cached entries.
        fs::read_dir(&target_directory)
            .unwrap()
            .filter(|file| {
                !file
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .to_str()
                    .unwrap()
                    .starts_with(".")
            })
            .for_each(|file| fs::remove_file(file.unwrap().path()).unwrap());

        let image_filename = "four_by_four.png".to_string();
        let mut original_image_path = PathBuf::from(&test_case_path);
        original_image_path.push("original");
        original_image_path.push(&image_filename);

        let standard_snapshot = FourByFour::Standardized.to_snapshot();
        let config = Config {
            target_directory: PathBuf::from(&target_directory),
            title: "title".to_string(),
            map_image: MapImageConfig {
                image_file: original_image_path.clone(),
                image_margins: None,
                skip_empty_cells: None,
            },
            geometry: Hexagons {
                definition: standard_snapshot.geometry_definition,
            },
            image_handling_config: None,
            template: None,
        };
        let table_of_contents_images: Vec<TableOfContentsMapImage> = vec![
            TableOfContentsMapImage {
                filename: "first_toc.jpg".to_string(),
                size: PixelPoint { x: 1, y: 2 },
                offset: PixelPoint { x: 3, y: 4 },
                coordinates_contained: HashSet::from(["foo".to_string(), "bar".to_string()]),
            },
            TableOfContentsMapImage {
                filename: "second_toc.jpg".to_string(),
                size: PixelPoint { x: 4, y: 5 },
                offset: PixelPoint { x: 6, y: 7 },
                coordinates_contained: HashSet::from(["baz".to_string(), "dag".to_string()]),
            },
        ];
        let table_of_contents_images_by_filename: HashMap<String, TableOfContentsMapImage> =
            table_of_contents_images
                .iter()
                .map(|toc| (toc.filename.clone(), toc.clone()))
                .collect();
        let cutout_images: Vec<CutoutImage> = vec![
            CutoutImage {
                coordinate: "first_cutout.png".to_string(),
                offset_from_original_image: PixelPoint { x: 10, y: 20 },
                image_size: PixelPoint { x: 30, y: 40 },
            },
            CutoutImage {
                coordinate: "second_cutout.png".to_string(),
                offset_from_original_image: PixelPoint { x: 100, y: 200 },
                image_size: PixelPoint { x: 300, y: 400 },
            },
        ];
        let cutout_images_by_coordinate: HashMap<String, CutoutImage> = cutout_images
            .iter()
            .map(|cutout| (cutout.coordinate.clone(), cutout.clone()))
            .collect();
        let image = FourByFourImages::Standardized.load_image();

        CachingTestCase {
            target_directory,
            config,
            image,
            cell_map: standard_snapshot.cell_map,
            table_of_contents_images,
            cutout_images,
            table_of_contents_images_by_filename,
            cutout_images_by_coordinate,
            original_image_path,
        }
    }

    mod get_cached_objects {
        use crate::caching::test::standard_test_case;
        use crate::caching::{CachedComputedObjects, get_cached_objects, persist_cached_objects};

        ///These are a bit silly because they all access the last_used_config.json at the root directory.
        ///For now we'll just run them in sequence.
        #[test]
        fn sequential_caching_tests() {
            returns_none_if_the_config_has_changed();
            returns_none_if_the_image_has_changed_completely();
            returns_all_cached_values_if_the_image_is_unchanged();
            filters_out_cached_values_intersecting_changed_pixels();
        }


        fn returns_none_if_the_config_has_changed() {
            let test_case_name = "changed_config".to_string();
            let test_case = standard_test_case(test_case_name);
            persist_cached_objects(
                &test_case.target_directory,
                &test_case.config,
                &test_case.image,
                test_case.cell_map.clone(),
                &test_case.table_of_contents_images,
                &test_case.cutout_images,
            );

            let mut changed_config = test_case.config;
            changed_config.title = "Changed title".to_string();
            let cached_objects = get_cached_objects(&changed_config, &test_case.image);

            assert_eq!(None, cached_objects);
        }

        fn returns_none_if_the_image_has_changed_completely() {
            let test_case_name = "new_image".to_string();
            let test_case = standard_test_case(test_case_name);
            persist_cached_objects(
                &test_case.target_directory,
                &test_case.config,
                &test_case.image,
                test_case.cell_map.clone(),
                &test_case.table_of_contents_images,
                &test_case.cutout_images,
            );

            let new_image = image::open(test_case.original_image_path).unwrap();
            let cached_objects = get_cached_objects(&test_case.config, &new_image);

            assert_eq!(None, cached_objects);
        }
        fn returns_all_cached_values_if_the_image_is_unchanged() {
            let test_case_name = "basic_caching".to_string();
            let test_case = standard_test_case(test_case_name);

            persist_cached_objects(
                &test_case.target_directory,
                &test_case.config,
                &test_case.image,
                test_case.cell_map.clone(),
                &test_case.table_of_contents_images,
                &test_case.cutout_images,
            );
            let cached_objects = get_cached_objects(&test_case.config, &test_case.image);

            let expected = Some(CachedComputedObjects {
                cell_map: test_case.cell_map,
                table_of_contents_map_images_by_filename: test_case
                    .table_of_contents_images_by_filename,
                cutout_images_by_coordinate: test_case.cutout_images_by_coordinate,
            });
            assert_eq!(expected, cached_objects); 
        }

        fn filters_out_cached_values_intersecting_changed_pixels() {
            let test_case_name = "updated_image".to_string();
            let test_case = standard_test_case(test_case_name);

            persist_cached_objects(
                &test_case.target_directory,
                &test_case.config,
                &test_case.image,
                test_case.cell_map.clone(),
                &test_case.table_of_contents_images,
                &test_case.cutout_images,
            );
            let changed_image = image::open(test_case.original_image_path).unwrap();
            let cached_objects = get_cached_objects(&test_case.config, &changed_image);

            //The original image here has its top-left corner changed, which eliminates
            //the first cutout and the first toc image.
            let expected = Some(CachedComputedObjects {
                cell_map: test_case.cell_map,
                table_of_contents_map_images_by_filename: test_case
                    .table_of_contents_images_by_filename
                    .into_iter()
                    .filter(|(_filename, toc)| toc.offset.x > 99)
                    .collect(),
                cutout_images_by_coordinate: test_case.cutout_images_by_coordinate
                    .into_iter()
                    .filter(|(_filename, cutout)| cutout.offset_from_original_image.x > 99)
                    .collect(),
            });

            assert_eq!(expected, cached_objects);
        }
    }

    mod find_changed_pixels {
        use image::DynamicImage::ImageRgb8;
        use image::{GenericImage, Rgba};
        use crate::caching::ChangedImage::{AllNew, Changes, NoChanges};
        use crate::caching::find_changed_pixels;
        use crate::{PixelBox, PixelPoint};

        #[test]
        fn identifies_an_image_as_new_if_its_dimensions_have_changed() {
            let baseline_image = ImageRgb8(image::RgbImage::new(32, 32));
            let wrong_width = ImageRgb8(image::RgbImage::new(30, 32));
            let wrong_height = ImageRgb8(image::RgbImage::new(32, 35));

            assert_eq!(AllNew, find_changed_pixels(&baseline_image, &wrong_width));
            assert_eq!(AllNew, find_changed_pixels(&baseline_image, &wrong_height));
        }

        #[test]
        fn identifies_an_image_as_new_if_its_changed_bounding_box_is_the_whole_image() {
            let baseline_image = ImageRgb8(image::RgbImage::new(32, 32));
            let mut changed_corners = ImageRgb8(image::RgbImage::new(32, 32));
            changed_corners.put_pixel(0, 0, Rgba([1, 1, 1, 1]));
            changed_corners.put_pixel(31, 31, Rgba([1, 1, 1, 1]));

            assert_eq!(AllNew, find_changed_pixels(&baseline_image, &changed_corners));
        }

        #[test]
        fn identifies_when_images_have_not_changed() {
            let baseline_image = ImageRgb8(image::RgbImage::new(32, 32));
            let same_image = ImageRgb8(image::RgbImage::new(32, 32));

            assert_eq!(NoChanges, find_changed_pixels(&baseline_image, &same_image));
        }

        #[test]
        fn returns_a_bounding_box_containing_all_changed_pixels_for_partially_changed_images() {
            let baseline_image = ImageRgb8(image::RgbImage::new(32, 32));
            let mut changed_diagonal = ImageRgb8(image::RgbImage::new(32, 32));
            for x in 10..=20 {
                for y in 12..=18 {
                    changed_diagonal.put_pixel(x, y, Rgba([1, 1, 1, 1]));
                }
            }

            let expected = Changes {bounding_box: PixelBox{
                top_left_corner: PixelPoint {x: 10, y:12},
                bottom_right_corner: PixelPoint {x:20, y:18} }};
            assert_eq!(expected, find_changed_pixels(&baseline_image, &changed_diagonal));
        }
    }
}
