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
                        x: map_image.width() as i32,
                        y: map_image.height() as i32,
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
    use crate::caching::persist_cached_objects;
    use crate::config::{Config, MapImageConfig};
    use crate::geometry::CellMap;
    use crate::geometry::Geometry::Hexagons;
    use crate::geometry::hexagons::fixtures::{FourByFour, ToSnapshot};
    use crate::image_handling::map_cutout::CutoutImage;
    use crate::image_handling::table_of_contents::TableOfContentsMapImage;
    use crate::image_handling::test::fixtures::FourByFourImages;
    use image::DynamicImage;
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    struct CachingTestCase {
        test_case: String,
        target_directory: PathBuf,
        config: Config,
        image: DynamicImage,
        image_filename: String,
        cell_map: CellMap,
        table_of_contents_images: Vec<TableOfContentsMapImage>,
        table_of_contents_images_by_filename: HashMap<String, TableOfContentsMapImage>,
        cutout_images: Vec<CutoutImage>,
        cutout_images_by_coordinate: HashMap<String, CutoutImage>
    }

    fn standard_test_case(test_case: String) -> CachingTestCase {
        let mut test_case_path = PathBuf::new();
        test_case_path.push(env!("CARGO_MANIFEST_DIR"));
        test_case_path.push("test_resources/caching");
        test_case_path.push(&test_case);
        let mut target_directory = PathBuf::from(&test_case_path);
        target_directory.push("cached");
        let image_filename = "four-by-four.png".to_string();
        let mut original_image_path = PathBuf::from(&test_case_path);
        original_image_path.push("original");
        original_image_path.push(&image_filename);

        let standard_snapshot = FourByFour::Standardized.to_snapshot();
        let config = Config {
            target_directory: PathBuf::from(&target_directory),
            title: "title".to_string(),
            map_image: MapImageConfig {
                image_file: original_image_path,
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
        let table_of_contents_images_by_filename : HashMap<String, TableOfContentsMapImage> = table_of_contents_images.iter().map(|toc|
            (toc.filename.clone(), toc.clone())
        ).collect();
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
        let cutout_images_by_coordinate : HashMap<String, CutoutImage> = cutout_images.iter().map(|cutout|
            (cutout.coordinate.clone(), cutout.clone())
        ).collect();
        let image = FourByFourImages::Standardized.load_image();

        CachingTestCase {
            test_case,
            target_directory,
            config,
            image,
            image_filename,
            cell_map: standard_snapshot.cell_map,
            table_of_contents_images,
            cutout_images,
            table_of_contents_images_by_filename,
            cutout_images_by_coordinate
        }
    }

    mod get_cached_objects {
        use crate::caching::{get_cached_objects, persist_cached_objects, CachedComputedObjects};
        use crate::caching::test::standard_test_case;

        #[test]
        fn returns_none_if_the_config_has_changed() {
            todo!()
        }

        #[test]
        fn returns_none_if_the_image_has_changed_completely() {
            todo!()
        }
        #[test]
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
                table_of_contents_map_images_by_filename: test_case.table_of_contents_images_by_filename,
                cutout_images_by_coordinate: test_case.cutout_images_by_coordinate,
            });
            assert_eq!(expected, cached_objects);
        }

        #[test]
        fn filters_out_cached_values_intersecting_changed_pixels() {
            todo!()
        }
    }

    mod find_changed_pixels {

        #[test]
        fn identifies_an_image_as_new_if_its_dimensions_have_changed() {
            todo!()
        }

        #[test]
        fn identifies_an_image_as_new_if_its_changed_bounding_box_is_the_whole_image() {
            todo!()
        }

        #[test]
        fn identifies_when_images_have_not_changed() {
            todo!()
        }

        #[test]
        fn returns_a_bounding_box_containing_all_changed_pixels_for_partially_changed_images() {
            todo!()
        }
    }
}
