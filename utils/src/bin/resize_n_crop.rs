use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use image::{Pixel, RgbaImage};
use walkdir::WalkDir;

const RESIZE_DIVISOR: u32 = 1;

fn main() {
    let exe_path = std::env::current_exe().expect("exe path");
    let search_dir = exe_path.parent().expect("exe parent dir");
    let output_dir = search_dir.join("./output");

    println!("Searching for webps in: {search_dir:?}");

    // Create output metadata file
    fs::create_dir_all(&output_dir).expect("can create output dir");
    let mut metadata_file =
        File::create(output_dir.join("metadata.txt")).expect("created metadata file");

    for entry in WalkDir::new(search_dir)
        .into_iter()
        .filter_map(Result::ok) // Gracefully skip unreadable folders
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "webp"))
        .filter(|e| !e.path().starts_with(&output_dir))
    // Only webp
    {
        let path = entry.path();
        println!("Found: {}", path.display());

        process_image(path, &search_dir, &output_dir, &mut metadata_file);

        println!("Processed: {}", path.display());
    }
}

fn process_image(path: &Path, search_dir: &Path, output_dir: &Path, metadata_file: &mut File) {
    // Open the image
    let img = image::open(path).expect("image opens").to_rgba8();
    let orig_dims = img.dimensions();

    // Figure out how much to crop
    let (x1, y1, x2, y2) = figure_out_how_much_to_crop(&img);
    let new_dims = (x2 - x1, y2 - y1);

    // Crop
    let cropped_img = image::imageops::crop_imm(&img, x1, y1, new_dims.0, new_dims.1).to_image();

    // Resize
    let resized_img = image::imageops::resize(
        &cropped_img,
        new_dims.0 / RESIZE_DIVISOR,
        new_dims.1 / RESIZE_DIVISOR,
        image::imageops::FilterType::Triangle,
    );

    // Write to output dir
    let write_path = output_dir.join(path.strip_prefix(search_dir).expect("strip prefix"));
    if let Some(parent) = write_path.parent() {
        fs::create_dir_all(parent).expect("create output dir");
    }
    resized_img.save(write_path).expect("save processed img");

    if new_dims != orig_dims {
        // Write to metadata file (crop + resize!)
        let name = path
            .file_stem()
            .expect("webp has a name")
            .to_string_lossy()
            .to_string();
        let x_off = x1 / RESIZE_DIVISOR;
        let y_off = y1 / RESIZE_DIVISOR;
        writeln!(metadata_file, "{name} | offset({x_off}, {y_off})").expect("write metadata");
    }
}

/// Returns the (x1,y1,x2,y2) of the inner box to crop to
fn figure_out_how_much_to_crop(img: &RgbaImage) -> (u32, u32, u32, u32) {
    let dims = img.dimensions();
    let (mut x1, mut y1, mut x2, mut y2) = (dims.0, dims.1, 0, 0);

    for (px, py, pixel) in img.enumerate_pixels() {
        if pixel.alpha() > 0 {
            if px < x1 {
                x1 = px - px % 2;
            }
            if py < y1 {
                y1 = py - py % 2;
            }
            if px > x2 {
                x2 = px + px % 2;
            }
            if py > y2 {
                y2 = py + py % 2;
            }
        }
    }

    // Trim symmetrically for x
    match x1 < dims.0 - x2 {
        true => x2 = dims.0 - x1,
        false => x1 = dims.0 - x2,
    }

    return (x1, y1, x2, y2);
}
