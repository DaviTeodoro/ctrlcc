# ctrlcc - Clipboard Link Saver

`ctrlcc` is a simple Rust utility that runs in the background on macOS and Linux, listening for a specific keyboard shortcut to save web links.

## Features

- **Global Keyboard Listener:** Monitors the `Command + C` (on macOS) or `Ctrl + C` (on Linux) key combination pressed twice in quick succession (within 600ms).
- **Clipboard Link Capture:** When the shortcut is detected, the program reads the current content of the clipboard.
- **URL Validation:** Checks if the copied text is a valid URL (starting with `http://`, `https://`, or `ftp://`).
- **Canvas File Saving:** If it's a valid URL, it's saved to an Obsidian Canvas file format.
    - Links are saved in the `~/notes/1cmdcc/` directory by default (customizable with `--path`).
    - A new file is created for each day, in the format `YYYY-MM-DD.canvas` (e.g., `2024-05-26.canvas`).
    - Each link is added as a node in the canvas with positioning based on time.
- **Configurable Save Path:** You can specify a custom directory using the `--path` argument.

## Prerequisites

- **Rust:** Ensure you have Rust and Cargo installed. You can install them from [rustup.rs](https://rustup.rs/).
- **System Dependencies (Linux):**
    - For `rdev` (keyboard listener), you might need to install some X11 development dependencies. On Debian/Ubuntu-based systems:
        ```bash
        sudo apt-get install libxkbcommon-dev libxkbcommon-x11-dev libxmu-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev
        ```
      On other distributions, package names may vary (look for equivalents of `libX11-devel`, `libXtst-devel`, etc.).

## How to Compile and Run

1.  **Clone the repository (if applicable) or create the files:**
    If you have the `Cargo.toml` and `src/main.rs` files:

2.  **Compile the project:**
    Navigate to the project's root directory and run:
    ```bash
    cargo build --release
    ```
    The compiled binary will be in `target/release/ctrlcc`.

3.  **Command Line Arguments:**
    The program accepts the following arguments:
    ```bash
    # Use default path (~/notes/1cmdcc)
    ./target/release/ctrlcc
    
    # Specify custom save path
    ./target/release/ctrlcc --path="/path/to/your/notes"
    
    # Show help
    ./target/release/ctrlcc --help
    ```

## Background Execution Setup

### macOS (using `launchd`)

1.  **Copy the binary to an accessible location:**
    ```bash
    sudo cp target/release/ctrlcc /usr/local/bin/ctrlcc
    ```

2.  **Create a LaunchAgent file:**
    Create the file `~/Library/LaunchAgents/com.youruser.ctrlcc.plist` (replace `com.youruser` with a unique identifier, like `com.davi.ctrlcc`) with the following content:

    ```xml
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
    <plist version="1.0">
      <dict>
        <key>Label</key>
        <string>com.youruser.ctrlcc</string> <!-- Use the same identifier here -->

        <key>ProgramArguments</key>
        <array>
          <string>/usr/local/bin/ctrlcc</string>
          <!-- Uncomment and modify to use custom path:
          <string>--path</string>
          <string>/path/to/your/notes</string>
          -->
        </array>

        <key>RunAtLoad</key>
        <true/>

        <key>KeepAlive</key>
        <true/>

        <key>StandardOutPath</key>
        <string>/tmp/ctrlcc.out.log</string>
        <key>StandardErrorPath</key>
        <string>/tmp/ctrlcc.err.log</string>
      </dict>
    </plist>
    ```

3.  **Load and start the service:**
    ```bash
    launchctl load ~/Library/LaunchAgents/com.youruser.ctrlcc.plist
    launchctl start com.youruser.ctrlcc
    ```
    To check if it's running: `launchctl list | grep ctrlcc`

4.  **Grant Accessibility Permissions:**
    - Open "System Settings" → "Privacy & Security" → "Accessibility".
    - Add `/usr/local/bin/ctrlcc` to the list of allowed applications and enable it. You might need to click the lock and enter your password to make changes.

### Linux (using `systemd` as a user service)

1.  **Install the binary:**
    ```bash
    mkdir -p ~/.local/bin
    cp target/release/ctrlcc ~/.local/bin/ctrlcc
    ```
    (Ensure `~/.local/bin` is in your `$PATH`).

2.  **Create a systemd service file:**
    Create the file `~/.config/systemd/user/ctrlcc.service` with the following content:

    ```ini
    [Unit]
    Description=ctrlcc - Clipboard Link Saver

    [Service]
    ExecStart=%h/.local/bin/ctrlcc
    # Uncomment and modify to use custom path.
    # If your path does NOT contain spaces, do not use quotes:
    # ExecStart=%h/.local/bin/ctrlcc --path %h/Notas/1cmdcc
    # If your path DOES contain spaces, use quotes around the path:
    # ExecStart=%h/.local/bin/ctrlcc --path "%h/My Notes/1cmdcc"
    Restart=on-failure
    # Optional: Logs, create the directory if you use this
    # StandardOutput=append:%h/.local/share/ctrlcc/out.log 
    # StandardError=append:%h/.local/share/ctrlcc/err.log

    [Install]
    WantedBy=default.target
    ```

3.  **Enable and start the service:**
    ```bash
    systemctl --user daemon-reload
    systemctl --user enable --now ctrlcc.service
    ```
    To check the status: `systemctl --user status ctrlcc.service`

4.  **Group Permissions (if needed for `rdev`):**
    On some Linux distributions, for `rdev` to listen to global keyboard events without `sudo`, your user might need to belong to the `input` group (or `plugdev` on some systems).
    ```bash
    sudo usermod -a -G input $USER
    ```
    You will need to log out and log back in for the group change to take effect.
    **Warning:** Adding your user to the `input` group grants more privileges to access input devices. Do this with caution.

## Usage

After setup, `ctrlcc` will run silently in the background.

1.  Copy a URL to your clipboard (e.g., from a web browser).
2.  Press `Command + C` (macOS) or `Ctrl + C` (Linux) twice quickly.
3.  The link will be saved to the `~/notes/1cmdcc/YYYY-MM-DD.canvas` file (or to your custom path if specified with `--path`).

Logs (standard output and errors) can be found at the paths specified in the service configuration files (`/tmp/ctrlcc.*.log` on macOS, or optionally in `~/.local/share/ctrlcc/` on Linux).