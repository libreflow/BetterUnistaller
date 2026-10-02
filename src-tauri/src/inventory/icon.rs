use base64::Engine;
use image::ImageEncoder;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconRequest {
    pub id: String,
    pub path: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IconReady {
    pub id: String,
    pub data_uri: String,
}

/// Résout un chemin d'icône (fichier .ico direct, ou icône embarquée d'un .exe/.dll
/// via SHGetFileInfoW) en data URI PNG 32x32. None si introuvable.
pub fn icon_data_uri(path: &str) -> Option<String> {
    let p = Path::new(path);
    if !p.exists() {
        return None;
    }
    let rgba = if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("ico")) {
        load_ico(p)?
    } else {
        extract_exe_icon(p)?
    };
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(rgba.as_raw(), rgba.width(), rgba.height(), image::ExtendedColorType::Rgba8)
        .ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    ))
}

fn load_ico(path: &Path) -> Option<image::RgbaImage> {
    let img = image::open(path).ok()?;
    Some(img.thumbnail(32, 32).to_rgba8())
}

#[cfg(windows)]
fn extract_exe_icon(path: &Path) -> Option<image::RgbaImage> {
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC,
        SelectObject, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

    let wide: Vec<u16> = encode_wide_nul(path.as_os_str());
    let mut info = SHFILEINFOW::default();
    unsafe {
        let ok = SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        if ok == 0 || info.hIcon.is_invalid() {
            return None;
        }
        let hicon = info.hIcon;

        let mut icon_info = ICONINFO::default();
        if GetIconInfo(hicon, &mut icon_info).is_err() {
            let _ = DestroyIcon(hicon);
            return None;
        }

        // Icônes monochromes : hbmColor peut être nul (le masque hbmMask porte
        // alors AND+XOR combinés). On ne gère pas ce cas — retour sûr à None
        // plutôt que de déréférencer un handle invalide dans GetDIBits.
        if icon_info.hbmColor.is_invalid() {
            let _ = DeleteObject(icon_info.hbmMask.into());
            let _ = DestroyIcon(hicon);
            return None;
        }

        const TARGET: u32 = 32;

        // SHGFI_LARGEICON n'est pas forcément 32x32 : à >=150% d'échelle
        // d'affichage Windows retourne des icônes 48px, 64px... On interroge
        // la bitmap réelle plutôt que de supposer 32x32, sous peine de
        // GetDIBits qui produit une image tronquée/déformée.
        let mut bitmap = BITMAP::default();
        let got = GetObjectW(
            icon_info.hbmColor.into(),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut _),
        );
        if got == 0 {
            let _ = DeleteObject(icon_info.hbmColor.into());
            let _ = DeleteObject(icon_info.hbmMask.into());
            let _ = DestroyIcon(hicon);
            return None;
        }
        let width = bitmap.bmWidth;
        let height = bitmap.bmHeight;
        if width <= 0 || height <= 0 {
            let _ = DeleteObject(icon_info.hbmColor.into());
            let _ = DeleteObject(icon_info.hbmMask.into());
            let _ = DestroyIcon(hicon);
            return None;
        }

        let screen_dc = GetDC(None);
        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        let old = SelectObject(mem_dc, icon_info.hbmColor.into());

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let lines = GetDIBits(
            mem_dc,
            icon_info.hbmColor,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        SelectObject(mem_dc, old);
        let _ = DeleteDC(mem_dc);
        ReleaseDC(None, screen_dc);
        let _ = DeleteObject(icon_info.hbmColor.into());
        let _ = DeleteObject(icon_info.hbmMask.into());
        let _ = DestroyIcon(hicon);

        if lines == 0 {
            return None;
        }
        // BGRA -> RGBA
        for px in pixels.as_chunks_mut::<4>().0 {
            px.swap(0, 2);
        }
        let full = image::RgbaImage::from_raw(width as u32, height as u32, pixels)?;
        if width as u32 == TARGET && height as u32 == TARGET {
            Some(full)
        } else {
            Some(image::imageops::thumbnail(&full, TARGET, TARGET))
        }
    }
}

#[cfg(not(windows))]
fn extract_exe_icon(_path: &Path) -> Option<image::RgbaImage> {
    None
}

/// OsStr -> UTF-16 terminé par NUL.
#[cfg(windows)]
fn encode_wide_nul(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(std::iter::once(0)).collect()
}

/// Calcule séquentiellement en tâche de fond ; chaque icône résolue est
/// poussée via l'événement `icon-ready` dès qu'elle est prête.
#[tauri::command]
pub fn load_icons(app: AppHandle, requests: Vec<IconRequest>) {
    tauri::async_runtime::spawn_blocking(move || {
        for req in requests {
            if let Some(data_uri) = icon_data_uri(&req.path) {
                let _ = app.emit("icon-ready", IconReady { id: req.id, data_uri });
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ico_file_is_converted_to_png_data_uri() {
        // Génère un .ico 16x16 monochrome sur disque via le crate image.
        let dir = std::env::temp_dir().join("bu_icon_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("t.ico");
        let img = image::RgbaImage::from_pixel(16, 16, image::Rgba([255, 0, 0, 255]));
        img.save(&path).unwrap();

        let uri = icon_data_uri(path.to_str().unwrap()).expect("doit produire une icône");
        assert!(uri.starts_with("data:image/png;base64,"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_yields_none() {
        assert_eq!(icon_data_uri(r"C:\bu_inexistant\x.exe"), None);
    }
}
