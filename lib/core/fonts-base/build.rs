use basegl_build_utilities::github_download;
use std::{path, env, fs};
use std::io::Write;

pub const PACKAGE_NAME         : &str = "dejavu-fonts-ttf-2.37.zip";
pub const VERSION              : &str = "version_2_37";
pub const PROJECT_URL          : &str =
    "https://github.com/dejavu-fonts/dejavu-fonts/";
pub const PACKAGE_FONTS_PREFIX : &str = "dejavu-fonts-ttf-2.37/ttf";
pub const FONTS_TO_EXTRACT     : &[&str] = &[
    "DejaVuSansMono",
    "DejaVuSansMono-Bold"
];

fn extract_dejavu_font(package_path : &path::Path, font_name : &str) {
    let font_file = format!("{}.ttf", font_name);
    let font_package_path = format!("{}/{}",
        PACKAGE_FONTS_PREFIX,
        font_file
    );

    let mut archive = zip::ZipArchive::new(
        std::fs::File::open(package_path).unwrap()
    ).unwrap();
    let mut input = archive.by_name(
        font_package_path.as_str()
    ).unwrap();
    let mut output = std::fs::File::create(
        package_path.parent().unwrap().join(font_file)
    ).unwrap();
    std::io::copy(&mut input, &mut output).unwrap();
}

fn main() {
    let out = env::var("OUT_DIR").unwrap();
    let out_dir = path::Path::new(&out);
    let package_path = out_dir.join(PACKAGE_NAME);
    let fill_map_rs_path = out_dir.join("fill_map.rs");

    github_download(
        PROJECT_URL,
        VERSION,
        PACKAGE_NAME,
        path::Path::new(&out_dir)
    );

    for font_name in FONTS_TO_EXTRACT {
        extract_dejavu_font(package_path.as_path(), font_name);
    }

    let mut fill_map_rs_file = fs::File::create(fill_map_rs_path).unwrap();
    writeln!(fill_map_rs_file, "{{").unwrap();
    for font_name in FONTS_TO_EXTRACT {
        writeln!(fill_map_rs_file,
            "   fonts_by_name.insert(\"{}\", include_bytes!(\"{}.ttf\"));",
            font_name,
            font_name
        ).unwrap();
    }
    writeln!(fill_map_rs_file, "}}").unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}