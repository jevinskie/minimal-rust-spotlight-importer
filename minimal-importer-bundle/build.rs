use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bundle_dir = out_dir.join("minimal-importer.mdimporter");
    let contents_dir = bundle_dir.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    fs::create_dir_all(&macos_dir).unwrap();

    let plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>English</string>
	<key>CFBundleDocumentTypes</key>
	<array>
		<dict>
			<key>CFBundleTypeRole</key>
			<string>MDImporter</string>
			<key>LSItemContentTypes</key>
			<array>
				<string>vin.je.great</string>
			</array>
		</dict>
	</array>
	<key>CFBundleExecutable</key>
	<string>minimal-importer</string>
	<key>CFBundleIconFile</key>
	<string></string>
	<key>CFBundleIdentifier</key>
	<string>vin.je.minimal-importer</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundlePackageType</key>
	<string>BNDL</string>
	<key>CFBundleSignature</key>
	<string>????</string>
	<key>CFBundleVersion</key>
	<string>1.0</string>
	<key>CFPlugInDynamicRegisterFunction</key>
	<string></string>
	<key>CFPlugInDynamicRegistration</key>
	<string>NO</string>
	<key>CFPlugInFactories</key>
	<dict>
		<key>D87857F7-B0C0-4C70-9B8F-2E3D8E55198C</key>
		<string>MetadataImporterPluginFactory</string>
	</dict>
	<key>CFPlugInTypes</key>
	<dict>
		<key>8B08C4BF-415B-11D8-B3F9-0003936726FC</key>
		<array>
			<string>D87857F7-B0C0-4C70-9B8F-2E3D8E55198C</string>
		</array>
	</dict>
	<key>CFPlugInUnloadFunction</key>
	<string></string>
	<key>UTExportedTypeDeclarations</key>
	<array>
		<dict>
			<key>UTTypeConformsTo</key>
			<array>
				<string>public.text</string>
			</array>
			<key>UTTypeDescription</key>
			<string>GREAT text file</string>
			<key>UTTypeIdentifier</key>
			<string>vin.je.great</string>
			<key>UTTypeTagSpecification</key>
			<dict>
				<key>public.filename-extension</key>
				<array>
					<string>great</string>
				</array>
			</dict>
		</dict>
	</array>
</dict>
</plist>"#;
    let plist_path = contents_dir.join("Info.plist");
    let mut plist_file = File::create(&plist_path).unwrap();
    plist_file.write_all(plist.as_bytes()).unwrap();

    // Copy or symlink compiled dylib to MacOS/
    let profile = env::var("PROFILE").unwrap();
    let crate_out = PathBuf::from("target")
        .join(&profile)
        .join("libminimal_importer_bundle.dylib"); // default name from Cargo

    let dest = macos_dir.join("minimal-importer");

    let _ = fs::remove_file(&dest);
    // symlink(&crate_out, &dest).unwrap();
    println!(
        "cargo:warning=PWD: {}",
        env::current_dir().unwrap().display()
    );
    println!(
        "cargo:warning=crate_out: {} dest: {}",
        crate_out.display(),
        dest.display()
    );
    fs::copy(&crate_out, &dest).expect("couldnt copy macho");

    println!("cargo:warning=Bundle output at: {}", bundle_dir.display());
}
