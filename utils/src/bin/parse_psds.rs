use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use heck::{ToPascalCase, ToSnakeCase};
use psd::Psd;
use walkdir::WalkDir;

struct LayerGroup {
    name: String,
    parent_id: Option<u32>,
    sub_groups: Vec<u32>, // IDs of child groups
}

impl Default for LayerGroup {
    fn default() -> Self {
        Self {
            name: String::new(),
            parent_id: None,
            sub_groups: Vec::new(),
        }
    }
}

const NO_GROUP_ID: i32 = -1;
const NO_GROUP_NAME: &str = "no_group";

fn main() {
    let search_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../game/assets");

    println!("Searching for PSDs in: {:?}", search_dir);

    for entry in WalkDir::new(search_dir.clone())
        .into_iter()
        .filter_map(Result::ok) // Gracefully skip unreadable folders
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "psd"))
    {
        let path = entry.path();
        let directory = path.parent().expect("PSD is in a directory");
        println!("Found: {}", path.display());

        // Load each
        let bytes = std::fs::read(path).unwrap();
        let psd = Psd::from_bytes(&bytes).unwrap();

        // println!("Groups: {:#?}", psd.groups());
        // println!("Layers: {:#?}", psd.layers());

        // Export each group as webp
        let exported_images = export_layers(&psd, directory);

        let poses = find_poses(&psd, &exported_images);
        println!("Poses: {:#?}", poses);

        // Generate layered images
        let name = path
            .file_stem()
            .expect("PSD has a name")
            .to_string_lossy()
            .to_string();

        let relative_path = directory
            .strip_prefix(&search_dir)
            .expect("Failed to strip prefix")
            .to_string_lossy()
            .to_string()
            .replace("\\", "/");
        generate_layered_image(&name, &relative_path, &poses);
    }
}

/// Exports all layers by group as webp in same directory as the PSD.
fn export_layers(psd: &Psd, directory: &Path) -> Vec<i32> {
    let mut exported_images = Vec::new();
    // 1. Export layers not in a group
    let pixels: Vec<u8> = psd
        .flatten_layers_rgba(&|(_idx, layer)| {
            !layer.name().starts_with("_") && layer.parent_id().is_none()
        })
        .unwrap();
    // TODO: Better name?
    if export_pixels(pixels, psd.width(), psd.height(), directory, NO_GROUP_NAME) {
        exported_images.push(NO_GROUP_ID);
    }

    // 2. Export all layers in each group as a webp
    for (group_id, group) in psd.groups() {
        // Filter by name and group id
        let pixels: Vec<u8> = psd
            .flatten_layers_rgba(&|(_idx, layer)| {
                !layer.name().starts_with("_") && layer.parent_id() == Some(*group_id)
            })
            .unwrap();
        if export_pixels(pixels, psd.width(), psd.height(), directory, &group.name()) {
            exported_images.push(*group_id as i32);
        }
    }

    exported_images
}

/// Exports the pixels as a webp if non-empty in the given directory.
fn export_pixels(pixels: Vec<u8>, width: u32, height: u32, directory: &Path, name: &str) -> bool {
    // Check if completely transparent
    if !pixels.chunks(4).all(|pixel| pixel[3] == 0) {
        let img = image::RgbaImage::from_raw(width, height, pixels).unwrap();
        img.save_with_format(
            directory.join(format!("{}.webp", name.strip_prefix("*").unwrap_or(name))),
            image::ImageFormat::WebP,
        )
        .expect(&format!("Failed to export {}.webp", name));
        true
    } else {
        false
    }
}

/// Returns a list of all possible poses (ordered combinations of groups)
fn find_poses(psd: &Psd, exported_images: &Vec<i32>) -> Vec<Vec<String>> {
    // Organize layers by groups
    // group id -> LayerGroup
    let mut groups = HashMap::new();
    for (group_id, group) in psd.groups() {
        let entry: &mut LayerGroup = groups.entry(*group_id).or_insert(Default::default());
        entry.name = group.name().to_string();

        // Add sub groups to parent group
        if let Some(parent_id) = group.parent_id() {
            entry.parent_id = Some(parent_id);
            let parent_entry: &mut LayerGroup =
                groups.entry(parent_id).or_insert(Default::default());
            parent_entry.sub_groups.push(*group_id);
        }
    }

    // Find all root groups
    let mut mut_excl_groups = Vec::new();
    let mut always_incl_groups = Vec::new();
    groups.iter().for_each(|(id, group)| {
        if group.parent_id.is_none() {
            println!("Root group found: {}", group.name);
            if group.name.starts_with("*") {
                always_incl_groups.push(*id);
            } else {
                mut_excl_groups.push(*id);
            }
        }
    });

    let mut poses = Vec::new();

    // 1. Mutually exclusive root groups
    mut_excl_groups.iter().for_each(|id| {
        if exported_images.contains(&(*id as i32)) {
            poses.extend(recurse_helper(&groups, *id, exported_images));
        }
    });

    // 2. Always included root groups
    always_incl_groups.iter().for_each(|id| {
        let results = recurse_helper(&groups, *id, exported_images);
        if poses.is_empty() {
            poses = results;
        } else {
            poses = poses
                .iter()
                .flat_map(|pose| {
                    results
                        .iter()
                        .map(move |result| [pose.as_slice(), result.as_slice()].concat())
                })
                .collect();
        }
    });

    // 3. Add the no group image to each pose
    if exported_images.contains(&NO_GROUP_ID) {
        poses.iter_mut().for_each(|pose| {
            pose.push(NO_GROUP_NAME.to_string());
        });
    }

    poses
}

/// Recursively travels from root to each leaf.
/// Combines groups where "*" prefix indicates always included.
/// Groups that begin with "_" are ignored.
/// Groups that were not exported (contains no images) are ignored.
/// Groups otherwise named are mutually exclusive within the same level.
fn recurse_helper(
    groups: &HashMap<u32, LayerGroup>,
    group_id: u32,
    exported_images: &Vec<i32>,
) -> Vec<Vec<String>> {
    let group = &groups[&group_id];

    // Base case (leaf)
    if group.sub_groups.is_empty() {
        return vec![vec![group.name.clone()]];
    }

    // Recursive case (concat sub results)
    let mut results = Vec::new();

    // First pass over mutually exclusive groups
    for sub_group_id in group.sub_groups.iter().copied() {
        let sub_group = &groups[&sub_group_id];
        if !sub_group.name.starts_with("*") && !sub_group.name.starts_with("_") {
            let sub_results = recurse_helper(groups, sub_group_id, exported_images);
            if exported_images.contains(&(group_id as i32)) {
                results.extend(
                    sub_results
                        .into_iter()
                        .map(|result| [vec![group.name.clone()], result].concat()),
                )
            } else {
                results.extend(sub_results);
            }
        }
    }

    // Second pass over always included groups
    for sub_group_id in group.sub_groups.iter().copied() {
        let sub_group = &groups[&sub_group_id];
        if sub_group.name.starts_with("*") && !sub_group.name.starts_with("_") {
            let sub_results = recurse_helper(groups, sub_group_id, exported_images);
            // Combine each result with each sub result
            results = results
                .into_iter()
                .flat_map(|result| {
                    sub_results
                        .iter()
                        .map(move |sub_result| [result.as_slice(), sub_result.as_slice()].concat())
                })
                .collect::<Vec<Vec<String>>>();
        }
    }
    results
}

/// Generates a layered image for the given pose.
fn generate_layered_image(name: &str, path: &str, poses: &Vec<Vec<String>>) {
    let layered_image_name = name.to_pascal_case();

    let mut enum_variant_names = Vec::new();
    for pose in poses {
        // Filter out the * prefixed and no group groups
        let pose_name = pose
            .iter()
            .filter(|s| !s.starts_with("*") && *s != NO_GROUP_NAME)
            .map(|s| s.to_pascal_case())
            .collect::<Vec<String>>()
            .join("");
        enum_variant_names.push(pose_name);
    }

    let mut enum_variants = String::new();
    for variant_name in &enum_variant_names {
        enum_variants.push_str(&format!("    {},\n", variant_name));
    }

    let mut match_variants = String::new();
    for (i, variant_name) in enum_variant_names.clone().iter().enumerate() {
        match_variants.push_str(&format!(
            "\n            {layered_image_name}::{variant_name} => vec!["
        ));
        for image in &poses[i] {
            let image_name = image.strip_prefix("*").unwrap_or(image);
            let asset_path = if path.is_empty() {
                format!("{image_name}.webp")
            } else {
                format!("{path}/{image_name}.webp")
            };
            match_variants.push_str(&format!(
                r#"
                ImageSchema::new("{asset_path}"),"#
            ));
        }
        match_variants.push_str("\n            ],");
    }

    let file = format!(
        r#"use script::{{layered_image::LayeredImage, state::schema::image_schema::ImageSchema}};

pub enum {layered_image_name} {{
{enum_variants}}}

impl LayeredImage for {layered_image_name} {{
    const ID: &'static str = "{layered_image_name}";

    fn images(&self) -> Vec<ImageSchema> {{
        match self {{{match_variants}
        }}
    }}
}}
"#,
        layered_image_name = layered_image_name,
        enum_variants = enum_variants,
    );

    // TODO: Consolidate with existing file (copy over transforms/animations?)
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../game/definitions/images")
        .join(format!("{}.rs", name.to_snake_case()));
    fs::write(output_path, file).expect("Write succeeds");
}
