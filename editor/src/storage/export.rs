// editor\src\storage\export.rs
use crate::storage::editor_storage::*;
use engine_core::constants::GAME_RON;
use engine_core::constants::RESOURCES_FOLDER;
use engine_core::*;
use std::io::Write;
use std::path::PathBuf;
use engine_core::onscreen_info;
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

/// TODO: Implement add recursively properly
async fn export_for_mac(dest_root: PathBuf, game: &Game) -> io::Result<PathBuf> {
    let bundle_path = dest_root.join(format!("{}.app", game.name));

    // Guard will clear up the export if there are errors
    let mut guard = ExportGuard::new(bundle_path.clone());

    // Copy template structure
    let template_dir = templates_dir()
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,  
                "Could not find templates.",)
        })?;

    // TODO: Sort this out
    let template_path = template_dir.join("template.app");
    copy_dir_recursive(&template_path, &bundle_path)?;

    // Copy game binary
    let macos_dir = bundle_path
        .join("Contents")
        .join("MacOS");

    // Make sure this file exists
    fs::create_dir_all(&macos_dir)?;

    let game_binary_dir = game_binary_dir()
    .ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,  
            "Could not find game binaries.",)
        })?;

    let src_binary = game_binary_dir.join("game");
    let target_binary = macos_dir.join(&game.name);
    fs::copy(src_binary, &target_binary)?;

    // Copy assets
    let src_assets = assets_folder(&game.name);

    let target_assets = bundle_path
        .join("Contents")
        .join("Resources")
        .join("assets");
    
    copy_dir_recursive(&src_assets, &target_assets)?;

    // Copy the game.ron
    let src_ron = game_folder(&game.name)
        .join(GAME_RON);

    let target_ron = bundle_path
        .join("Contents")
        .join("Resources")
        .join(GAME_RON);

    fs::copy(src_ron, target_ron)?;

    // Create Info.plist TODO: this does not work
    let target_plist = bundle_path.join("Contents").join("Info.plist");
    let mut plist = fs::read_to_string(&target_plist)?;
    plist = plist
        .replace("__BUNDLE_NAME__", &game.name)
        .replace("__BUNDLE_IDENTIFIER__", &format!("com.bishop.{}", game.name.to_lowercase()))
        .replace("__BUNDLE_VERSION__", "0.1.0");
    fs::write(&target_plist, plist)?;

    // Copy app icons
    let src_icons = game_folder(&game.name)
        .join("Icon.icns");

    let target_icons = bundle_path
        .join("Contents")
        .join("Resources")
        .join("Icon.icns");

    fs::copy(src_icons, target_icons)?;

    // Copy app window icon
    let src_window_icon = game_folder(&game.name)
        .join("icon.png");

    let target_window_icon = bundle_path
        .join("Contents")
        .join("Resources")
        .join("icon.png");

    fs::copy(src_window_icon, target_window_icon)?;

    // Tell the guard to keep the folder
    guard.success();
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
