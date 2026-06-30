fn main() {
    let build_uid = uuid::Uuid::new_v4().simple().to_string();
    println!("cargo:rustc-env=APP_BUILD_UID={}", build_uid);
    tauri_build::build();
}
