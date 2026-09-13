//! S0.5's evidence, run with `--decode-report <dir>`: one line per file, and the invariant
//! proved by hashing.
//!
//! For every file in the folder: what it is, whether it decoded, how long the open took,
//! and what came out. Where the file is one of the generated fixtures, the mark it carries
//! is checked: the orientation moved the red corner, the profile turned red into green,
//! frame two is blue, page three is blue, transparency is still transparent. A file that is
//! not a fixture gets its line without a verdict.
//!
//! Before anything is opened every file is hashed, and after everything is opened and every
//! frame asked for, hashed again: the same bytes, the same files, nothing new beside them,
//! and nothing new in Recon's own data folder. That is CLAUDE.md rule 11 measured.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::{open, Format, Kind, OpenError, Opened};

/// FNV-1a over the whole file: fast, dependency-free, and a single changed byte changes it.
fn hash_file(path: &Path) -> Option<u64> {
    let bytes = std::fs::read(path).ok()?;
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    Some(h)
}

fn snapshot(dir: &Path) -> BTreeMap<PathBuf, u64> {
    let mut map = BTreeMap::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(h) = hash_file(&path) {
                    map.insert(path, h);
                }
            }
        }
    }
    map
}

fn recon_data_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("Recon"))
}

fn pixel(frame: &super::DecodedFrame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * frame.width + x) * 4) as usize;
    [
        frame.rgba[i],
        frame.rgba[i + 1],
        frame.rgba[i + 2],
        frame.rgba[i + 3],
    ]
}

fn near(a: [u8; 4], b: [u8; 4], tol: u8) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| x.abs_diff(*y) <= tol)
}

struct Line {
    file: String,
    verdict: &'static str,
    text: String,
}

/// The fixture-specific checks, keyed by the file name the generator used. Returns the
/// failures, empty when every mark is where it should be.
fn check_fixture(name: &str, opened: &mut Opened) -> Vec<String> {
    let mut bad = Vec::new();
    let mut expect = |ok: bool, what: &str| {
        if !ok {
            bad.push(what.to_string());
        }
    };
    match name {
        "png-alpha-and-swapped-profile.png" => {
            let f = opened.frame(0).expect("frame 0");
            let left = pixel(&f, 10, 50);
            let right = pixel(&f, 190, 50);
            expect(right[3] == 0, "the transparent half is still transparent");
            expect(
                left[1] > left[0] + 60,
                &format!(
                    "the swapped profile was honoured: stored red reads as green, got {left:?}"
                ),
            );
            expect(
                opened.notes.color.starts_with("converted"),
                "the colour note says converted",
            );
        }
        "jpeg-orientation-6.jpg" => {
            let f = opened.frame(0).expect("frame 0");
            expect(
                opened.width == 200 && opened.height == 300,
                &format!(
                    "dimensions are after orientation, 200x300, got {}x{}",
                    opened.width, opened.height
                ),
            );
            expect(
                near(pixel(&f, 190, 10), [255, 0, 0, 255], 40),
                "the red corner moved to the top right",
            );
            expect(
                near(pixel(&f, 10, 290), [0, 0, 255, 255], 40),
                "the blue corner moved to the bottom left",
            );
            expect(
                opened.notes.orientation_applied == Some(6),
                "the note says orientation 6 was applied",
            );
        }
        "bmp-corners.bmp" => {
            let f = opened.frame(0).expect("frame 0");
            expect(
                near(pixel(&f, 5, 5), [255, 0, 0, 255], 2),
                "the red corner is top left",
            );
            expect(
                near(pixel(&f, 155, 115), [0, 0, 255, 255], 2),
                "the blue corner is bottom right",
            );
        }
        "gif-three-frames.gif" | "webp-three-frames.webp" => {
            expect(
                opened.kind.count() == 3,
                &format!("three frames, got {}", opened.kind.count()),
            );
            for i in 0..opened.kind.count().min(3) {
                let f = opened.frame(i).expect("frame");
                let want = super::fixtures::index_color(i);
                expect(
                    near(pixel(&f, 5, 5), want, 8),
                    &format!(
                        "frame {i} is its own colour {want:?}, got {:?}",
                        pixel(&f, 5, 5)
                    ),
                );
            }
            if let Kind::Animation { delays_ms } = &opened.kind {
                expect(
                    delays_ms.iter().all(|d| *d == 80),
                    &format!("every delay is 80 ms, got {delays_ms:?}"),
                );
            }
            expect(opened.frame(3).is_err(), "frame 3 is refused");
        }
        "webp-still-with-alpha.webp" => {
            let f = opened.frame(0).expect("frame 0");
            expect(
                pixel(&f, 100, 45)[3] == 0,
                "the transparent half is still transparent",
            );
            expect(
                near(pixel(&f, 10, 45), [220, 30, 30, 255], 2),
                "the opaque half is exact, lossless",
            );
        }
        "tiff-three-pages.tif" => {
            expect(
                matches!(opened.kind, Kind::Pages { count: 3 }),
                &format!("three pages, got {:?}", opened.kind),
            );
            for i in 0..opened.kind.count().min(3) {
                let f = opened.frame(i).expect("page");
                let want = super::fixtures::index_color(i);
                expect(
                    near(pixel(&f, 5, 5), want, 2),
                    &format!("page {i} is its own colour, got {:?}", pixel(&f, 5, 5)),
                );
            }
        }
        "png-large-8000x6000.png" => {
            expect(opened.width == 8000 && opened.height == 6000, "8000x6000");
            let f = opened.frame(0).expect("frame 0");
            expect(f.rgba.len() == 8000 * 6000 * 4, "every byte present");
        }
        "svg-text-labels.svg" | "svg-css-style-block.svg" | "svg-mask-and-clip.svg" => {
            let f = opened.frame(0).expect("frame 0");
            let painted = f.rgba.chunks(4).filter(|p| p[3] > 0).count();
            expect(
                painted > (f.width * f.height / 20) as usize,
                "something was painted",
            );
            if name == "svg-css-style-block.svg" {
                expect(
                    near(pixel(&f, 60, 60), [0x33, 0x66, 0xcc, 255], 2),
                    "the CSS class fill reached the rectangle",
                );
            }
            if name == "svg-mask-and-clip.svg" {
                expect(pixel(&f, 80, 80)[3] == 0, "the masked hole is transparent");
                expect(
                    near(pixel(&f, 30, 30), [0xff, 0x99, 0x00, 255], 2),
                    "the masked rectangle is orange outside the hole",
                );
                expect(
                    near(pixel(&f, 110, 50), [0xff, 0x99, 0x00, 255], 2),
                    "the clipped circle is absent left of the clip, so the rectangle shows",
                );
                expect(
                    near(pixel(&f, 170, 80), [0x22, 0x44, 0xaa, 255], 2),
                    "the clipped circle is present right of the clip",
                );
            }
            if name == "svg-text-labels.svg" {
                let dark = (14..300u32)
                    .filter(|x| {
                        let p = pixel(&f, *x, 45);
                        p[0] < 0x80 && p[3] > 0
                    })
                    .count();
                expect(dark > 10, "text pixels exist on the label's baseline row");
            }
        }
        _ => {}
    }
    bad
}

pub fn run(dir: &Path) -> i32 {
    println!(
        "=== S0.5: open and decode, every file in {} ===",
        dir.display()
    );
    let before = snapshot(dir);
    let data_before = recon_data_dir().map(|d| snapshot(&d)).unwrap_or_default();
    println!("{} files hashed before anything was opened", before.len());
    println!();

    let mut lines: Vec<Line> = Vec::new();
    let mut failures = 0;
    for path in before.keys() {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let started = Instant::now();
        match open(path) {
            Ok(mut opened) => {
                let open_ms = started.elapsed().as_millis();
                let started = Instant::now();
                let first = opened.frame(0);
                let frame_ms = started.elapsed().as_millis();
                let count = opened.kind.count();
                if count > 1 {
                    let _ = opened.frame(count - 1);
                }
                let mut text = format!(
                    "{} {}x{} {} · open {open_ms} ms, frame 0 {frame_ms} ms · {} · {} · {}",
                    opened.format.name(),
                    opened.width,
                    opened.height,
                    match &opened.kind {
                        Kind::Still => "still".to_string(),
                        Kind::Animation { delays_ms } =>
                            format!("animation, {} frames", delays_ms.len()),
                        Kind::Pages { count } => format!("{count} pages"),
                        Kind::Vector {
                            intrinsic_w,
                            intrinsic_h,
                        } => format!("vector {intrinsic_w}x{intrinsic_h}"),
                    },
                    opened.notes.provider,
                    opened.notes.color,
                    opened.notes.alpha,
                );
                if let Some(o) = opened.notes.orientation_applied {
                    text.push_str(&format!(" · orientation {o} applied"));
                }
                for r in &opened.notes.remarks {
                    text.push_str(&format!(" · {r}"));
                }
                let verdict = match first {
                    Ok(_) => {
                        let bad = check_fixture(&name, &mut opened);
                        if bad.is_empty() {
                            "pass"
                        } else {
                            failures += 1;
                            text.push_str(&format!(" · FAILED: {}", bad.join("; ")));
                            "FAIL"
                        }
                    }
                    Err(err) => {
                        failures += 1;
                        text.push_str(&format!(" · FAILED: frame 0: {err}"));
                        "FAIL"
                    }
                };
                lines.push(Line {
                    file: name,
                    verdict,
                    text,
                });
            }
            Err(err) => {
                let expected_refusal = name == "not-an-image.txt";
                let verdict = match (&err, expected_refusal) {
                    (OpenError::Unsupported(_), true) => "pass",
                    (OpenError::MissingCodec { .. }, _) => "codec",
                    _ => {
                        failures += 1;
                        "FAIL"
                    }
                };
                lines.push(Line {
                    file: name,
                    verdict,
                    text: format!("{err} · {} ms", started.elapsed().as_millis()),
                });
            }
        }
    }

    let after = snapshot(dir);
    let data_after = recon_data_dir().map(|d| snapshot(&d)).unwrap_or_default();
    let unchanged = before == after;
    let nothing_new_in_data = data_before == data_after;

    let mut out = String::new();
    for line in &lines {
        out.push_str(&format!(
            "  {:<5} {:<36} {}\n",
            line.verdict, line.file, line.text
        ));
    }
    out.push('\n');
    out.push_str(&format!(
        "  {}  every source file is byte for byte what it was, and no file appeared beside them ({} files before, {} after)\n",
        if unchanged { "pass " } else { "FAIL " },
        before.len(),
        after.len()
    ));
    out.push_str(&format!(
        "  {}  nothing was written to Recon's own data folder ({} files before, {} after)\n",
        if nothing_new_in_data {
            "pass "
        } else {
            "FAIL "
        },
        data_before.len(),
        data_after.len()
    ));
    if !unchanged || !nothing_new_in_data {
        failures += 1;
    }
    let pending: Vec<&str> = [Format::Heic, Format::Avif]
        .iter()
        .filter(|f| !lines.iter().any(|l| l.text.starts_with(f.name())))
        .map(|f| f.name())
        .collect();
    if !pending.is_empty() {
        out.push_str(&format!(
            "\n  PENDING INPUT: no {} file was in the folder, so those rows have no evidence yet; a real file from a phone or a browser is needed\n",
            pending.join(" or ")
        ));
    }
    out.push_str(&format!(
        "\nRESULT: {}\n",
        if failures == 0 {
            "every check that had a file to run on passed.".to_string()
        } else {
            format!("{failures} check(s) failed.")
        }
    ));
    print!("{out}");

    let report_path = std::env::current_exe()
        .map(|exe| exe.with_file_name("s05-decode-report.txt"))
        .unwrap_or_else(|_| "s05-decode-report.txt".into());
    match std::fs::write(&report_path, &out) {
        Ok(()) => println!("report written to {}", report_path.display()),
        Err(err) => println!("the report could not be written: {err}"),
    }
    if failures == 0 {
        0
    } else {
        1
    }
}
