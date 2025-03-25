use tauri_winres::VersionInfo;

fn main() {
    println!("cargo:rerun-if-env-changed=VERSION_NUMBER");
    let mut res = tauri_winres::WindowsResource::new();

    res.set_icon("src/app.ico");

    res.set("OriginalFilename", "hyper-key.exe");
    res.set("ProductName", "Hyper Key");
    res.set("FileDescription", "Hyper Key");

    let version_result = std::env::var("VERSION_NUMBER");
    let version = match version_result {
        Ok(ver) => ver,
        Err(_) => "0.0.0".to_string(),
    };

    let version_parts = version
        .split('.')
        .take(3)
        .map(|part| part.parse().unwrap_or(0))
        .collect::<Vec<u16>>();

    let [major, minor, patch] = <[u16; 3]>::try_from(version_parts).unwrap_or([0, 0, 0]);

    let version_str = format!("{major}.{minor}.{patch}.0");
    res.set("FileVersion", &version_str);
    res.set("ProductVersion", &version_str);

    let version_u64 =
        (u64::from(major) << 48) | (u64::from(minor) << 32) | (u64::from(patch) << 16);

    res.set_version_info(VersionInfo::FILEVERSION, version_u64);
    res.set_version_info(VersionInfo::PRODUCTVERSION, version_u64);

    res.compile().unwrap();
}
