# WASM Tetris (Rust)

Небольшое веб‑приложение с игрой «Тетрис». Вся логика написана на Rust и компилируется в WebAssembly. JavaScript используется только для загрузки WASM-модуля.

## Требования

- Rust (stable)
- `wasm-pack`
- Node.js + npm

Установка `wasm-pack`:

```bash
cargo install wasm-pack
```

## Быстрый старт

```bash
npm install
npm run start
```

Откройте браузер: `http://localhost:3000`

## Управление

- Влево/вправо: Arrow Left/Right
- Мягкое падение: Arrow Down
- Поворот: Arrow Up
- Жесткое падение: Space
- Перезапуск: R

## Структура проекта

- `src/lib.rs` — игровая логика и отрисовка через `CanvasRenderingContext2d`
- `index.html` — разметка страницы
- `main.js` — минимальный загрузчик WASM
- `style.css` — стили интерфейса
- `pkg/` — артефакты `wasm-pack` (создается при сборке)

## Полезные команды

- `npm run build` — сборка WASM в `pkg/`
- `npm run start` — сборка и запуск локального сервера
