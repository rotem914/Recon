//! The Open dialog for `Ctrl+O` (§3.1, S1.2): Windows' own file picker, filtered to the
//! formats Recon displays, owned by the editor window.
//!
//! It runs on a thread of its own with its own COM apartment, because the picker blocks
//! until the user answers and a Tauri command must not.

use std::path::PathBuf;

use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::Common::COMDLG_FILTERSPEC;
use windows::Win32::UI::Shell::{
    FileOpenDialog, FileSaveDialog, IFileOpenDialog, IFileSaveDialog, IShellItem,
    SHCreateItemFromParsingName, FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, FOS_NOREADONLYRETURN,
    SIGDN_FILESYSPATH,
};

/// Shows the Save As dialog for a PNG (§3.6, S1.11), opened on `folder` with `name` filled
/// in, and blocks until it closes. `Ok(None)` is a cancel. The dialog's own overwrite
/// prompt is off on purpose: an existing name is never overwritten, the caller offers an
/// available one instead, so the question the prompt asks must never be asked.
pub fn save_png(
    owner: Option<HWND>,
    folder: &std::path::Path,
    name: &str,
) -> Result<Option<PathBuf>, String> {
    unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let result = (|| {
            let dialog: IFileSaveDialog =
                CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)
                    .map_err(|err| format!("the Save As dialog could not be created: {err}"))?;
            // PNG first, the default; JPEG beside it (S2.6). The dialog swaps the name's
            // extension when the type is switched, and the write encodes by the extension.
            let png_name = wide("PNG image");
            let png_spec = wide("*.png");
            let jpeg_name = wide("JPEG image");
            let jpeg_spec = wide("*.jpg;*.jpeg");
            let filters = [
                COMDLG_FILTERSPEC {
                    pszName: PCWSTR(png_name.as_ptr()),
                    pszSpec: PCWSTR(png_spec.as_ptr()),
                },
                COMDLG_FILTERSPEC {
                    pszName: PCWSTR(jpeg_name.as_ptr()),
                    pszSpec: PCWSTR(jpeg_spec.as_ptr()),
                },
            ];
            dialog
                .SetFileTypes(&filters)
                .map_err(|err| err.to_string())?;
            let ext = wide("png");
            dialog
                .SetDefaultExtension(PCWSTR(ext.as_ptr()))
                .map_err(|err| err.to_string())?;
            dialog
                .SetOptions(FOS_FORCEFILESYSTEM | FOS_NOREADONLYRETURN)
                .map_err(|err| err.to_string())?;
            let folder_wide = wide(&folder.display().to_string());
            if let Ok(item) = SHCreateItemFromParsingName::<
                PCWSTR,
                Option<&windows::Win32::System::Com::IBindCtx>,
                IShellItem,
            >(PCWSTR(folder_wide.as_ptr()), None)
            {
                let _ = dialog.SetFolder(&item);
            }
            let name_wide = wide(name);
            dialog
                .SetFileName(PCWSTR(name_wide.as_ptr()))
                .map_err(|err| err.to_string())?;
            let title = wide("Save As a PNG, a new file");
            dialog
                .SetTitle(PCWSTR(title.as_ptr()))
                .map_err(|err| err.to_string())?;
            match dialog.Show(owner) {
                Ok(()) => {}
                Err(err) if err.code().0 as u32 == 0x8007_04C7 => return Ok(None),
                Err(err) => return Err(format!("the Save As dialog failed: {err}")),
            }
            let item = dialog.GetResult().map_err(|err| err.to_string())?;
            let chosen = item
                .GetDisplayName(SIGDN_FILESYSPATH)
                .map_err(|err| err.to_string())?;
            let path = PathBuf::from(String::from_utf16_lossy(chosen.as_wide()));
            CoTaskMemFree(Some(chosen.0 as *const _));
            Ok(Some(path))
        })();
        if init.is_ok() {
            CoUninitialize();
        }
        result
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// The picker's pattern for the formats in §3.7.
fn image_patterns() -> String {
    crate::registration::EXTENSIONS
        .iter()
        .map(|ext| format!("*{ext}"))
        .collect::<Vec<_>>()
        .join(";")
}

/// Shows the picker owned by `owner` and blocks until it closes. `Ok(None)` is a cancel.
pub fn pick_image(owner: Option<HWND>) -> Result<Option<PathBuf>, String> {
    unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let result = (|| {
            let dialog: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
                    .map_err(|err| format!("the picker could not be created: {err}"))?;
            let images_name = wide("Images");
            let images_spec = wide(&image_patterns());
            let all_name = wide("All files");
            let all_spec = wide("*.*");
            let filters = [
                COMDLG_FILTERSPEC {
                    pszName: PCWSTR(images_name.as_ptr()),
                    pszSpec: PCWSTR(images_spec.as_ptr()),
                },
                COMDLG_FILTERSPEC {
                    pszName: PCWSTR(all_name.as_ptr()),
                    pszSpec: PCWSTR(all_spec.as_ptr()),
                },
            ];
            dialog
                .SetFileTypes(&filters)
                .map_err(|err| err.to_string())?;
            dialog
                .SetOptions(FOS_FILEMUSTEXIST | FOS_FORCEFILESYSTEM)
                .map_err(|err| err.to_string())?;
            let title = wide("Open an image");
            dialog
                .SetTitle(PCWSTR(title.as_ptr()))
                .map_err(|err| err.to_string())?;
            match dialog.Show(owner) {
                Ok(()) => {}
                // 0x800704C7 is the user cancelling, which is an answer, not an error.
                Err(err) if err.code().0 as u32 == 0x8007_04C7 => return Ok(None),
                Err(err) => return Err(format!("the picker failed: {err}")),
            }
            let item = dialog.GetResult().map_err(|err| err.to_string())?;
            let name = item
                .GetDisplayName(SIGDN_FILESYSPATH)
                .map_err(|err| err.to_string())?;
            let path = PathBuf::from(String::from_utf16_lossy(name.as_wide()));
            CoTaskMemFree(Some(name.0 as *const _));
            Ok(Some(path))
        })();
        if init.is_ok() {
            CoUninitialize();
        }
        result
    }
}
