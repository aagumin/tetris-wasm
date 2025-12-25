# WASM Tetris (Rust)

A small web app with a Tetris game. All game logic is written in Rust and compiled to WebAssembly. JavaScript is only used to load the WASM module.

## Requirements

- Rust (stable)
- `wasm-pack`
- Node.js + npm

Install `wasm-pack`:

```bash
cargo install wasm-pack
```

## Quick start

```bash
npm install
npm run start
```

Open in your browser: `http://localhost:3000`

## Controls

- Move left/right: Arrow Left/Right
- Soft drop: Arrow Down
- Rotate: Arrow Up
- Hard drop: Space
- Restart: R

## Project structure

- `src/lib.rs` - game logic and rendering via `CanvasRenderingContext2d`
- `index.html` - page markup
- `main.js` - minimal WASM loader
- `style.css` - UI styles
- `pkg/` - `wasm-pack` artifacts (generated on build)

## Useful commands

- `npm run build` - build WASM into `pkg/`
- `npm run start` - build and start local server
