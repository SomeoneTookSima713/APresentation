use std::fs::{ read_to_string, write as write_to_file };
use std::path::PathBuf;
use serde::Deserialize;

const CONFIG_FILE: &'static str = "build_config.hjson";

const FONT_FILE: &'static str = "OpenSans.ttf";

#[derive(Deserialize)]
struct Config {
    app_version: [u8;3],
    include_default_font: bool,
    enable_debugging_features: bool,
}

fn main() {
    println!("cargo:rerun-if-changed=src/version");

    // The path to the current working directory - or rather the root directory of the project.
    // Errors if the environment variable doesn't exist or contains an invalid path.
    let cwd: PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap().parse().expect("invalid path in environment: CARGO_MANIFEST_DIR");

    let config_path = cwd.join(CONFIG_FILE);
    // The parsed config file.
    // Errors if the file can't be read or can't be parsed.
    let config: Config = deser_hjson::from_str( read_to_string(config_path).expect("error reading config file").as_str() ).expect("invalid config format");

    // The current build type; either 'release' or 'debug'.
    // Errors if the environment variable doesn't exist.
    let build_type = std::env::var("PROFILE").unwrap();

    let font_path = cwd.join("src").join(FONT_FILE);
    // Download and include the default font if specified in the build config.
    // It should only get downloaded if the file doesn't exist though.
    if config.include_default_font {
        if !font_path.exists() {
            let font_url = "http://fonts.gstatic.com/s/opensans/v10/cJZKeOuBrn4kERxqtaUH3SZ2oysoEQEeKwjgmXLRnTc.ttf";
            let font_bytes = {
                let response = reqwest::blocking::get(font_url).expect("couldn't reach url of default font");

                response.bytes().expect("couldn't download default font")
            };
            write_to_file(font_path, font_bytes.to_vec()).expect("couldn't write data of default font to file");
        }
        println!("cargo:rustc-cfg=default_font");
    }

    if config.enable_debugging_features {
        println!("cargo:rustc-cfg=debug_features")
    }

    let version_string = match build_type.as_str() {
        // building in debug-mode
        "debug" => format!("{}.{}.{}-DEBUG",config.app_version[0],config.app_version[1],config.app_version[2]),
        // building in release mode
        _ => format!("{}.{}.{}",config.app_version[0],config.app_version[1],config.app_version[2]),
    };

    let version_path = cwd.join("src/version");
    write_to_file(version_path, version_string).expect("couldn't write file for version string");
}