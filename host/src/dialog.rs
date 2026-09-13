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
    FileOpenDialog, IFileOpenDialog, FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, SIGDN_FILESYSPATH,
};

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
