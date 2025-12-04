// editor\src\storage\export.rs
use crate::editor_assets::editor_assets::GAME_BIN;
use std::os::unix::fs::PermissionsExt;
use engine_core::constants::CONTENTS_FOLDER;
use engine_core::constants::RESOURCES_FOLDER;
use engine_core::*;
use std::io::Write;
use std::path::PathBuf;
use winres_edit::Id;
use winres_edit::Resources;
use winres_edit::resource_type;
use std::io::Error;
use std::io::ErrorKind;
use engine_core::storage::path_utils::*;
use engine_core::game::game::*;
use macroquad::prelude::*;
use std::io;
use std::fs;
use crate::editor_assets::editor_assets::GAME_EXE;

/// Removes `path` when dropped unless `success()` has been called.
struct ExportGuard {
    path: PathBuf,
    ok: bool,
}

impl ExportGuard {
    fn new(path: PathBuf) -> Self {
        Self { path, ok: false }
    }

    fn success(&mut self) {
        self.ok = true;
    }
}

impl Drop for ExportGuard {
    fn drop(&mut self) {
        if !self.ok && self.path.exists() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

/// Exports the game to the chosen folder on all platforms.
pub async fn export_game(game: &Game) -> io::Result<PathBuf> {
    let dest_root = rfd::FileDialog::new()
        .set_title("Select destination folder for export:")
        .pick_folder()
        .ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidInput,  
                "No destination folder was selected.")
        })?;

    // TODO: This overwrites, check for duplicates

    #[cfg(target_os = "windows")]
    {
        onscreen_info!("Exporting for windows");
        let exe_path = export_for_windows(&dest_root, game).await?;
        return Ok(exe_path);
    }
    #[cfg(target_os = "macos")]
    {
        let bundle_path = export_for_mac(dest_root, game).await?;
        return Ok(bundle_path);
    }

    // TODO Handle Linux
}

async fn export_for_windows(dest_root: &PathBuf, game: &Game) -> io::Result<PathBuf> {
    let target_package = dest_root.join(format!("{}", &game.name));

    // Guard will clear up the package if there is an error
    let mut guard = ExportGuard::new(target_package.clone());

    // Write the .exe to the package directory
    fs::create_dir_all(&target_package)?;
    let exe_path = &target_package.join(format!("{}.exe", game.name));

    onscreen_debug!("Creating new .exe at: {}", exe_path.display());
    let mut exe_file = fs::File::create(&exe_path)?;

    onscreen_debug!("Writing buffer into .exe");
    exe_file.write_all(GAME_EXE)?;

    onscreen_debug!("Updating .exe");
    if let Err(e) = update_exe(&exe_path, game) {
        return Err(Error::new(
            ErrorKind::Other,  
            format!("Could not update .exe: {e}"))
        );
    }

    // Everything else goes in /Resources to mirror macOS structure
    let src_resources = resources_folder(&game.name);
    let target_resources = target_package.join(RESOURCES_FOLDER);
    copy_dir_recursive(&src_resources, &target_resources)?;

    // TODO: Write manifest for game
    
    // Tell the guard to keep the folder
    guard.success();
    Ok(target_package)
}

async fn export_for_mac(dest_root: PathBuf, game: &Game) -> io::Result<PathBuf> {
    let bundle_path = dest_root.join(format!("{}.app", game.name));

    // Guard will clear up the export if there are errors
    let mut guard = ExportGuard::new(bundle_path.clone());

    // Write the game binary to the bundle
    let macos_dir = bundle_path
        .join(CONTENTS_FOLDER)
        .join("MacOS");

    // Make sure this file exists
    fs::create_dir_all(&macos_dir)?;

    let bin_path = &macos_dir.join(format!("{}", game.name));

    onscreen_debug!("Creating new binary at: {}", bin_path.display());
    let mut bin_file = fs::File::create(&bin_path)?;

    onscreen_debug!("Writing buffer into binary.");
    bin_file.write_all(GAME_BIN)?;
    bin_file.flush()?;  

    // Set executable permissions
    onscreen_debug!("Writing binary permissions.");
    let mut permissions = fs::metadata(&bin_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&bin_path, permissions)?;

    // Copy /Resources
    onscreen_debug!("Copying /Resources.");
    let src_resources = resources_folder(&game.name);
    let target_resources = bundle_path
        .join(CONTENTS_FOLDER)
        .join(RESOURCES_FOLDER);

    copy_dir_recursive(&src_resources, &target_resources)?;

    // Copy Icon.icns
    onscreen_debug!("Copying Icon.icns.");
    let src_icns = mac_os_folder(&game.name)
        .join("Icon.icns");

    let target_icns = target_resources
        .join("Icon.icns");

    fs::copy(src_icns, target_icns)?;

    // Copy Info.plist
    if let Some(bundle_assets) = bundle_assets_folder() {
        // TODO: add more to plist
        onscreen_debug!("Copying Info.plist.");
        let src_plist = bundle_assets.join("Info.plist");
        let target_plist = bundle_path.join(CONTENTS_FOLDER).join("Info.plist");
        let _ = fs::copy(src_plist, target_plist);
    }

    // Tell the guard to keep the folder
    guard.success();
    onscreen_debug!("Export successful.");
    Ok(bundle_path)
}

/// Updates the game .exe with the game information.
fn update_exe(exe_path: &PathBuf, game: &Game) -> Result<(), winres_edit::Error> {
    let resources = Resources::new(&exe_path);

    let icon_path = windows_folder(&game.name).join("Icon.ico");

    // TODO: Maybe 1 PNG which the program can handle 
    // all together using Image or .ico crate and icns

    // Read the file and replace the icon
    if let Ok(png_bytes) = fs::read(&icon_path) {
        onscreen_debug!("Replacing .ico from: {}", icon_path.display());
        if let Some(icon_resource) = resources.find(resource_type::ICON, Id::Integer(1)) {
            icon_resource.replace(&png_bytes)?
                .update()?;
        }
    } else {
        onscreen_warn!("Could not read .ico");
    }
    
    if let Some(mut version_info) = resources.get_version_info()? {
        onscreen_debug!("Updating version info");
        // TODO: Update with actual version
        let version: [u16;4] = [0,1,0,0];

        let game_name = game.name.as_str();

        version_info.set_file_version(&version)
            .set_product_version(&version)
            .insert_strings(
                &[
                    // TODO: Use real values:
                    ("ProductName", game_name),
                    ("OriginalFilename", format!("{}.exe", game_name).as_str()),
                    ("FileDescription", "Game Description."),
                    ("LegalCopyright", "© 2025 Clashmar"),
                    ("LegalTrademark", "Bishop Engine™"),
                    ("CompanyName", "Clashmar Ltd."),
                    ("Comments", "A 2D Game made with Bishop Engine"),
                    ("InternalName", game_name),
                ]
            )
            .update()?;
    } else {
        onscreen_warn!("Could not get version info");
    }

    Ok(())
}
