//! TIFF, HEIC and AVIF through the Windows Imaging Component.
//!
//! TIFF is here for its pages: WIC exposes every page as a frame. HEIC and AVIF are here
//! because their decoders are codecs installed on the machine (the HEIF Image Extension,
//! the HEVC and AV1 Video Extensions), and part 5 says a missing codec is reported as a
//! missing codec with the way to install it, never as a corrupt file. The plan named
//! libavif with dav1d for AVIF; that chain needs a C toolchain this machine does not have,
//! and the pure-Rust ports did not compile here, so AVIF takes this route today and the
//! report says so (S0.5's evidence line).
//!
//! Every frame is converted to 32-bit straight-alpha RGBA by WIC's own converter, so one
//! shape comes out whatever the file held. The file is opened for reading only, with the
//! decoder told to read metadata on demand, and nothing here ever obtains a write handle.

use std::path::Path;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{GENERIC_READ, WINCODEC_ERR_COMPONENTNOTFOUND};
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, GUID_WICPixelFormat32bppRGBA, IWICBitmapDecoder, IWICImagingFactory,
    WICBitmapDitherTypeNone, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};

use super::{DecodedFrame, Format, FrameSource, Kind, Notes, OpenError, Opened};

fn factory() -> Result<IWICImagingFactory, String> {
    unsafe {
        // Per thread, and harmless when already done: the decode runs on whichever thread
        // asked, and COM wants to be told once on each.
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
            .map_err(|err| err.to_string())
    }
}

fn wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn how_to_install(format: Format) -> String {
    match format {
        Format::Heic => "Install \"HEIF Image Extensions\" and \"HEVC Video Extensions\" from the Microsoft Store, then open the file again.".into(),
        Format::Avif => "Install \"AV1 Video Extension\" from the Microsoft Store, then open the file again.".into(),
        _ => "Install the matching image extension from the Microsoft Store, then open the file again.".into(),
    }
}

pub fn open(path: &Path, format: Format, _bytes: Vec<u8>) -> Result<Opened, OpenError> {
    let factory = factory().map_err(OpenError::Unreadable)?;
    let name = wide(path);
    let decoder: IWICBitmapDecoder = unsafe {
        factory.CreateDecoderFromFilename(
            PCWSTR(name.as_ptr()),
            None,
            GENERIC_READ,
            WICDecodeMetadataCacheOnDemand,
        )
    }
    .map_err(|err| {
        if err.code() == WINCODEC_ERR_COMPONENTNOTFOUND {
            OpenError::MissingCodec {
                format,
                how: how_to_install(format),
            }
        } else {
            OpenError::Corrupt {
                format,
                detail: err.to_string(),
            }
        }
    })?;

    let count = unsafe { decoder.GetFrameCount() }.map_err(|err| OpenError::Corrupt {
        format,
        detail: err.to_string(),
    })?;
    let mut source = Wic {
        factory,
        decoder,
        format,
    };
    let first = source.frame(0)?;
    let kind = match format {
        Format::Tiff if count > 1 => Kind::Pages { count },
        // An animated AVIF is an image sequence; WIC exposes it as frames too.
        Format::Avif if count > 1 => Kind::Animation {
            delays_ms: vec![0; count as usize],
        },
        _ => Kind::Still,
    };
    let notes = Notes {
        // WIC's HEIF decoder applies the file's rotation itself; whether it does is one of
        // the things the report checks on a real file rather than assumes.
        orientation_applied: None,
        color: "as WIC converts it: sRGB for tagged files, untagged otherwise".into(),
        alpha: "alpha preserved through 32bppRGBA".into(),
        provider: "Windows Imaging Component",
        remarks: vec![format!("{count} frame(s) in the container")],
    };
    Ok(Opened {
        path: path.to_path_buf(),
        format,
        kind,
        width: first.width,
        height: first.height,
        notes,
        source: Box::new(source),
    })
}

struct Wic {
    factory: IWICImagingFactory,
    decoder: IWICBitmapDecoder,
    format: Format,
}

// COM interfaces are apartment-bound in general, but WIC's factory and decoders are free
// threaded, and every call here happens on the thread that opened the file anyway.
unsafe impl Send for Wic {}

impl FrameSource for Wic {
    fn frame(&mut self, index: u32) -> Result<DecodedFrame, OpenError> {
        let corrupt = |err: windows::core::Error| OpenError::Corrupt {
            format: self.format,
            detail: err.to_string(),
        };
        unsafe {
            let frame = self.decoder.GetFrame(index).map_err(corrupt)?;
            let converter = self.factory.CreateFormatConverter().map_err(corrupt)?;
            converter
                .Initialize(
                    &frame,
                    &GUID_WICPixelFormat32bppRGBA,
                    WICBitmapDitherTypeNone,
                    None,
                    0.0,
                    WICBitmapPaletteTypeCustom,
                )
                .map_err(corrupt)?;
            let (mut width, mut height) = (0u32, 0u32);
            converter
                .GetSize(&mut width, &mut height)
                .map_err(corrupt)?;
            if width == 0 || height == 0 {
                return Err(OpenError::Corrupt {
                    format: self.format,
                    detail: "a frame with no size".into(),
                });
            }
            let stride = width * 4;
            let mut rgba = vec![0u8; (stride * height) as usize];
            converter
                .CopyPixels(std::ptr::null(), stride, &mut rgba)
                .map_err(corrupt)?;
            Ok(DecodedFrame {
                width,
                height,
                rgba,
            })
        }
    }
}
