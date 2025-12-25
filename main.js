import init, { start } from "./pkg/tetris.js";

async function boot() {
  await init();
  start();
}

boot();
