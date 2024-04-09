pub fn main() {
    // println!("cargo::rerun-if-changed=src/icon.bmp");
    println!("cargo::rerun-if-env-changed=OUT_DIR");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    let img = image::io::Reader::open("src/icon.bmp").unwrap().decode().unwrap();
    let rgbaimg = img.as_rgba8().unwrap();
    let path = std::path::Path::new(out_dir.as_str()).join("icon.bin");
    std::fs::write(path.clone(), rgbaimg.as_raw()).unwrap();
}