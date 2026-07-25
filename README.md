# Start to Run — Interval Timer

A phone-friendly interval timer for the Belgian "Start to Run" 0→5K in 10 weeks
programme.  Built in Rust, compiled to WebAssembly, served as an installable PWA.

**Green screen = run.  Blue screen = walk.  Tap anywhere to start/pause.**

## Features

- Full 10-week / 30-session schedule baked in
- Full-screen colour changes on interval transitions (visible at a glance)
- Audio beeps on each transition (double beep = run, single = walk)
- Toggle to hide all time displays (the "I don't want to watch the clock" mode)
- Screen Wake Lock so your phone stays on
- Installable as a PWA — works offline

## Prerequisites

```bash
# Install Rust (if you haven't already)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the WASM target
rustup target add wasm32-unknown-unknown

# Install trunk (the build tool for Leptos CSR apps)
cargo install trunk

# (Optional) Install wasm-bindgen-cli for manual builds
cargo install wasm-bindgen-cli
```

## Development

```bash
# From the project root:
trunk serve --open

# This will:
# 1. Compile the Rust code to WASM
# 2. Run wasm-bindgen to generate JS glue
# 3. Serve everything on http://localhost:8080
# 4. Hot-reload on changes
```

Open `http://localhost:8080` on your phone (same Wi-Fi network) or use your
computer's browser for testing.

## Production Build

```bash
trunk build --release

# Output lands in ./dist/
# Deploy the contents of dist/ to any static file host:
# - GitHub Pages
# - Netlify
# - Cloudflare Pages
# - A simple nginx/caddy server
# - Even `python3 -m http.server` in the dist/ folder
```

## Installing on Your Phone

1. Deploy the `dist/` folder to any HTTPS host (PWA requires HTTPS).
2. Open the URL on your phone's browser.
3. You should see an "Add to Home Screen" prompt (or find it in the browser
   menu).
4. Once installed, it works offline and launches like a native app.

**For local testing** you can use `trunk serve --address 0.0.0.0` and visit
`http://<your-computer-ip>:8080` from your phone.  Note: the PWA install
prompt won't appear over plain HTTP, but the app itself works fine.

## Project Structure

```
src/
├── main.rs        # Leptos components (App, WeekList, SessionList, TimerView)
├── schedule.rs    # The 10-week schedule as Rust data structures
├── timer.rs       # Timer state machine (pure Rust, unit-tested)
└── audio.rs       # Web Audio API beep helper via wasm-bindgen

public/
├── index.html     # HTML shell
├── style.css      # All styling
├── manifest.json  # PWA manifest
└── sw.js          # Service worker for offline caching
```

## Customising the Schedule

Edit `src/schedule.rs`.  Each `Session` is a `Vec<Interval>` where each
interval has a `kind` (Run or Walk) and a `duration_secs`.  Helper functions
`r(mins)` and `w(mins)` make it readable:

```rust
Session {
    week: 1,
    session: 1,
    intervals: vec![
        r(1), w(1),  // run 1 min, walk 1 min
        r(2), w(2),  // run 2 min, walk 2 min
        // ...
    ],
}
```

If you want to cross-check against the actual start-to-run.be sessions, play
each audio session once and note the timings, then update the data here.

## Trunk Configuration

Trunk reads the `public/index.html` as its entry point by default.  If you
need to customise the build, create a `Trunk.toml`:

```toml
[build]
target = "public/index.html"
dist = "dist"
```
