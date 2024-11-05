# Macos Window Manager

Built this because coming from Linux I am missing an easy way to interact with windows on Mac with the Keyboard

I will add Keycodes as config too so you don't have to use mine.

right now its Cmd + Ctrl + (left,right,up) Arrows. Will position your window based on the screen size.

## Run locally

Install rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### build and run the project
```bash
cargo build --release && ./build/release/macos-window-manager
# could also
cargo run --release
```
