use arboard::Clipboard;
use chrono::Local;
use clap::Parser;
use dirs::home_dir;
use rand::{thread_rng, Rng};
use rdev::{listen, EventType, Key};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to save notes (default: ~/notes/1cmdcc)
    #[arg(long, value_name = "PATH")]
    path: Option<PathBuf>,
}

#[derive(Default)]
struct KeyState {
    meta_down: bool,
    last_cmd_c: Option<Instant>,
    notes_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Canvas {
    nodes: Vec<Node>,
    edges: Vec<Value>,
}

#[derive(Serialize, Deserialize)]
struct Node {
    id: String,
    #[serde(rename = "type")]
    node_type: String,
    url: String,
    #[serde(default)]
    date: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn random_id() -> String {
    let mut rng = thread_rng();
    let id: u64 = rng.gen();
    format!("{:016x}", id)
}

fn main() {
    let args = Args::parse();

    let notes_path = args
        .path
        .unwrap_or_else(|| home_dir().unwrap().join("notes/1cmdcc"));

    let state = Arc::new(Mutex::new(KeyState {
        notes_path,
        ..Default::default()
    }));
    let state_cb = state.clone();

    // Callback triggered on any keyboard event
    let callback = move |event: rdev::Event| {
        let mut st = state_cb.lock().unwrap();

        match event.event_type {
            EventType::KeyPress(key) => {
                match key {
                    Key::ControlLeft | Key::ControlRight => {
                        st.meta_down = true;
                    }
                    Key::KeyC => {
                        if st.meta_down {
                            let now = Instant::now();
                            if let Some(prev) = st.last_cmd_c {
                                if now.duration_since(prev) < Duration::from_millis(600) {
                                    // Double Cmd+C
                                    st.last_cmd_c = None;
                                    let notes_path = st.notes_path.clone();
                                    drop(st); // release lock before heavy work
                                    if let Err(e) = handle_clipboard(&notes_path) {
                                        eprintln!("Erro ao salvar link: {e}");
                                    }
                                    return;
                                }
                            }
                            st.last_cmd_c = Some(now);
                        }
                    }
                    _ => {}
                }
            }
            EventType::KeyRelease(key) => {
                if key == Key::ControlLeft || key == Key::ControlRight {
                    st.meta_down = false;
                }
            }
            _ => {}
        }
    };

    // Start the listener (blocking)
    if let Err(e) = listen(callback) {
        eprintln!("Erro no listener: {:?}", e);
    }
}

// ------------------------------------------------
//   Helper functions
// ------------------------------------------------

fn handle_clipboard(notes_path: &PathBuf) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::new()?;
    let text = clipboard.get_text()?;

    if is_url(&text) {
        save_link(&text, notes_path)?;
        println!("Link salvo: {text}");
    }
    Ok(())
}

fn is_url(s: &str) -> bool {
    static RE: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| Regex::new(r"^(https?|ftp)://[^\s/$.?#].[^\s]*$").unwrap());
    RE.is_match(s)
}

fn save_link(link: &str, notes_path: &PathBuf) -> anyhow::Result<()> {
    let dir = notes_path.clone();
    std::fs::create_dir_all(&dir)?;

    let filename = Local::now().format("%Y-%m-%d").to_string() + ".canvas";
    let path = dir.join(filename);

    let mut canvas: Canvas = if path.exists() {
        let data = std::fs::read_to_string(&path)?;
        serde_json::from_str(&data)?
    } else {
        Canvas {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    };

    let now = chrono::Local::now();
    let now_str = now.to_rfc3339();

    // Decide position based on time, allowing unlimited columns
    let (row, col) = if let Some(last) = canvas.nodes.last() {
        // Determine row/col of last node (base -400, step 960)
        let last_row = ((last.y + 180) / 780) as i32;
        let last_col = ((last.x + 400) / 960) as i32;

        // Calculate time difference
        let same_block = chrono::DateTime::parse_from_rfc3339(&last.date)
            .ok()
            .map(|dt| {
                now.signed_duration_since(dt.with_timezone(&now.timezone()))
                    < chrono::Duration::minutes(5)
            })
            .unwrap_or(false);

        if same_block {
            (last_row, last_col + 1)
        } else {
            (last_row + 1, 0)
        }
    } else {
        (0, 0)
    };

    let x = -400 + col * 960;
    let y = -180 + row * 780;

    // Create new node
    let node = Node {
        id: random_id(),
        node_type: "link".into(),
        url: link.to_string(),
        date: now_str,
        x,
        y,
        width: 880,
        height: 680,
    };

    canvas.nodes.push(node);

    // Save again (pretty)
    let json = serde_json::to_string_pretty(&canvas)?;
    std::fs::write(path, json)?;

    Ok(())
}
