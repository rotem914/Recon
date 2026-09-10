//! S0.3: the process boundary, measured in isolation, with no editor code involved.
//!
//! Part 5 pins a design where the decoded original never crosses into the web view at full
//! resolution: the web view gets a display-resolution proxy, the export comes back as an
//! annotation layer, and the host composites. The plan says that design is adopted either
//! way, because exact equality and the clipboard need it regardless, and that what this step
//! settles is whether the naive full-resolution design was ever available at all.
//!
//! It also replaces the figure the design was reasoned against, a maintainer's own
//! explicitly unscientific number from a discussion thread, with one from this machine.
//!
//! HOW IT IS TIMED, because that is most of whether the numbers mean anything.
//!
//! Every measurement is taken by the WEB VIEW's own clock, and never across two clocks.
//! For a crossing into the web view that is exactly the question anyway: how long until the
//! editor holds the pixels. For a crossing back out, the page times send-to-acknowledgement
//! and the empty round trip is measured separately and subtracted, so the number is transfer
//! rather than protocol.
//!
//! Nothing here is product code. It builds no editor, keeps no state, and is reached only
//! with `--bench`.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, OnceLock};

use tauri::ipc::{InvokeBody, Request, Response};
use tauri::{WebviewUrl, WebviewWindowBuilder};

/// The payloads, and why each one is here.
///
/// `full4k` is the naive design: a whole 3840x2160 frame in 32-bit pixels. `proxy1080` is
/// what part 5 says the web view should actually get. `png1080` is the same proxy encoded,
/// because a real implementation would probably send an image rather than raw pixels, and
/// the encode is a cost on the same path. `walk` is the folder-navigation proxy, which §3.4
/// says happens many times a day. `tiny` is the baseline: the protocol with nothing in it.
fn payload_bytes(kind: &str) -> Option<Vec<u8>> {
    let (w, h) = match kind {
        "full4k" => (3840usize, 2160usize),
        "proxy1080" => (1920, 1080),
        "walk" => (1280, 720),
        "tiny" => (16, 16),
        "png1080" => return Some(png_proxy()),
        _ => return None,
    };
    // A gradient rather than a constant, so a route that quietly truncates or pads is caught
    // by the page's own checksum instead of passing on a buffer of zeroes.
    let mut bytes = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            bytes[i] = (x % 251) as u8;
            bytes[i + 1] = (y % 241) as u8;
            bytes[i + 2] = ((x + y) % 239) as u8;
            bytes[i + 3] = 255;
        }
    }
    Some(bytes)
}

/// The encoded proxy, made once. The encode time itself is reported separately.
fn png_proxy() -> Vec<u8> {
    static CACHE: OnceLock<(Vec<u8>, u128)> = OnceLock::new();
    let (bytes, _) = CACHE.get_or_init(|| {
        let raw = {
            let mut bytes = vec![0u8; 1920 * 1080 * 4];
            for y in 0..1080usize {
                for x in 0..1920usize {
                    let i = (y * 1920 + x) * 4;
                    bytes[i] = (x % 251) as u8;
                    bytes[i + 1] = (y % 241) as u8;
                    bytes[i + 2] = ((x + y) % 239) as u8;
                    bytes[i + 3] = 255;
                }
            }
            bytes
        };
        let started = std::time::Instant::now();
        let mut out = std::io::Cursor::new(Vec::new());
        let image = image::RgbaImage::from_raw(1920, 1080, raw).expect("proxy dimensions");
        image
            .write_to(&mut out, image::ImageFormat::Png)
            .expect("png encode");
        let took = started.elapsed().as_millis();
        (out.into_inner(), took)
    });
    bytes.clone()
}

fn png_encode_ms() -> u128 {
    let started = std::time::Instant::now();
    let mut out = std::io::Cursor::new(Vec::new());
    let raw = payload_bytes("proxy1080").expect("proxy");
    let image = image::RgbaImage::from_raw(1920, 1080, raw).expect("proxy dimensions");
    image
        .write_to(&mut out, image::ImageFormat::Png)
        .expect("png encode");
    started.elapsed().as_millis()
}

/// The loopback route: a hand-rolled HTTP server on 127.0.0.1, no dependency added.
///
/// It exists because it is one of the three routes the plan asks to compare, and because a
/// local socket is the usual answer when an IPC channel turns out to be the bottleneck.
fn start_loopback() -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    std::thread::spawn(move || serve(stream));
                }
                Err(err) => eprintln!("loopback accept failed: {err}"),
            }
        }
    });
    Ok(port)
}

fn serve(mut stream: TcpStream) {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    // Read the request line and headers one byte at a time: the bodies here are megabytes
    // and must not be swallowed by a buffered reader that over-reads.
    while head.len() < 8192 {
        match stream.read(&mut byte) {
            Ok(0) => return,
            Ok(_) => {
                head.push(byte[0]);
                if head.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            Err(_) => return,
        }
    }
    let text = String::from_utf8_lossy(&head).to_string();
    let mut lines = text.lines();
    let request_line = lines.next().unwrap_or_default().to_string();
    let mut content_length = 0usize;
    for line in lines {
        if let Some(value) = line.strip_prefix("Content-Length: ") {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();

    if method == "OPTIONS" {
        let _ = stream.write_all(
            b"HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nContent-Length: 0\r\n\r\n",
        );
        return;
    }

    if method == "POST" {
        let mut body = vec![0u8; content_length];
        if stream.read_exact(&mut body).is_err() {
            return;
        }
        let reply = format!("{{\"received\":{}}}", body.len());
        let _ = stream.write_all(
            format!(
                "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                reply.len(),
                reply
            )
            .as_bytes(),
        );
        return;
    }

    let kind = path.trim_start_matches("/frame/");
    match payload_bytes(kind) {
        Some(bytes) => {
            let header = format!(
                "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                bytes.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&bytes);
        }
        None => {
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\n\r\n");
        }
    }
}

// ---------- the commands the page calls ----------

#[tauri::command]
fn bench_frame(kind: String) -> Result<Response, String> {
    payload_bytes(&kind)
        .map(Response::new)
        .ok_or_else(|| format!("unknown payload {kind}"))
}

#[tauri::command]
fn bench_layer(request: Request<'_>) -> Result<usize, String> {
    match request.body() {
        InvokeBody::Raw(bytes) => Ok(bytes.len()),
        InvokeBody::Json(_) => Err("the layer arrived as JSON, not as bytes".into()),
    }
}

#[tauri::command]
fn bench_port(state: tauri::State<'_, BenchState>) -> u16 {
    state.port
}

#[tauri::command]
fn bench_report(state: tauri::State<'_, BenchState>, csv: String, table: String) {
    if let Ok(sender) = state.done.lock() {
        let _ = sender.send((csv, table));
    }
}

struct BenchState {
    port: u16,
    done: Arc<Mutex<Sender<(String, String)>>>,
}

/// Runs the whole measurement and prints the result. Returns a process exit code.
pub fn run() -> i32 {
    let port = match start_loopback() {
        Ok(port) => port,
        Err(err) => {
            println!("the loopback route could not start: {err}");
            0
        }
    };
    println!("loopback route on 127.0.0.1:{port}");
    println!("png encode of the 1920x1080 proxy: {} ms", png_encode_ms());
    println!("opening the web view; the page drives the measurement and reports back");
    println!();

    let (sender, receiver) = channel::<(String, String)>();
    let state = BenchState {
        port,
        done: Arc::new(Mutex::new(sender)),
    };

    let app = tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            bench_frame,
            bench_layer,
            bench_port,
            bench_report
        ])
        .register_asynchronous_uri_scheme_protocol("frame", |_ctx, request, responder| {
            // The third route: a custom scheme, which on Windows arrives as
            // http://frame.localhost/<kind>.
            let kind = request.uri().path().trim_start_matches('/').to_string();
            std::thread::spawn(move || match payload_bytes(&kind) {
                Some(bytes) => responder.respond(
                    tauri::http::Response::builder()
                        .header("Content-Type", "application/octet-stream")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(bytes)
                        .expect("a response with a body"),
                ),
                None => responder.respond(
                    tauri::http::Response::builder()
                        .status(404)
                        .body(Vec::new())
                        .expect("an empty response"),
                ),
            });
        })
        .setup(|app| {
            WebviewWindowBuilder::new(app, "bench", WebviewUrl::App("bench.html".into()))
                .title("Recon boundary measurement")
                .inner_size(900.0, 600.0)
                .build()?;
            Ok(())
        })
        .build(tauri::generate_context!());

    let app = match app {
        Ok(app) => app,
        Err(err) => {
            println!("the web view could not be built: {err}");
            return 1;
        }
    };

    // The page reports on its own thread; this waits for it, then tears the runtime down.
    let handle = app.handle().clone();
    std::thread::spawn(move || {
        match receiver.recv_timeout(std::time::Duration::from_secs(300)) {
            Ok((csv, table)) => {
                println!("{table}");
                let path = std::env::current_exe()
                    .map(|exe| exe.with_file_name("s03-boundary.csv"))
                    .unwrap_or_else(|_| "s03-boundary.csv".into());
                match std::fs::write(&path, csv) {
                    Ok(()) => println!("\nrows written to {}", path.display()),
                    Err(err) => println!("\nthe csv could not be written: {err}"),
                }
            }
            Err(_) => println!("the page did not report within five minutes"),
        }
        handle.exit(0);
    });

    app.run(|_app, _event| {});
    0
}
