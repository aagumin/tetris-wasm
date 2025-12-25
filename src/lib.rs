use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent};

const WIDTH: usize = 10;
const HEIGHT: usize = 20;
const BLOCK: f64 = 32.0;

const COLORS: [&str; 8] = [
    "#0e0f14", // empty
    "#2dd4bf", // I
    "#fbbf24", // O
    "#a855f7", // T
    "#22c55e", // S
    "#ef4444", // Z
    "#3b82f6", // J
    "#f97316", // L
];

const SHAPES: [[[(i8, i8); 4]; 4]; 7] = [
    // I
    [
        [(0, 1), (1, 1), (2, 1), (3, 1)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
        [(0, 2), (1, 2), (2, 2), (3, 2)],
        [(1, 0), (1, 1), (1, 2), (1, 3)],
    ],
    // O
    [
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
    ],
    // T
    [
        [(1, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (1, 2)],
        [(1, 0), (0, 1), (1, 1), (1, 2)],
    ],
    // S
    [
        [(1, 0), (2, 0), (0, 1), (1, 1)],
        [(1, 0), (1, 1), (2, 1), (2, 2)],
        [(1, 1), (2, 1), (0, 2), (1, 2)],
        [(0, 0), (0, 1), (1, 1), (1, 2)],
    ],
    // Z
    [
        [(0, 0), (1, 0), (1, 1), (2, 1)],
        [(2, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (1, 2), (2, 2)],
        [(1, 0), (0, 1), (1, 1), (0, 2)],
    ],
    // J
    [
        [(0, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (2, 2)],
        [(1, 0), (1, 1), (0, 2), (1, 2)],
    ],
    // L
    [
        [(2, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (1, 2), (2, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 2)],
        [(0, 0), (1, 0), (1, 1), (1, 2)],
    ],
];

#[derive(Default)]
struct InputState {
    left: bool,
    right: bool,
    down: bool,
    left_just: bool,
    right_just: bool,
    rotate: bool,
    hard_drop: bool,
    restart: bool,
}

struct Rng {
    state: u32,
}

impl Rng {
    fn new(seed: u32) -> Self {
        Self { state: seed } 
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    fn next_usize(&mut self, max: usize) -> usize {
        (self.next_u32() as usize) % max
    }
}

struct Game {
    ctx: CanvasRenderingContext2d,
    board: [u8; WIDTH * HEIGHT],
    piece: usize,
    next_piece: usize,
    x: i32,
    y: i32,
    rotation: usize,
    rng: Rng,
    input: InputState,
    last_time: f64,
    fall_accum: f64,
    last_left: f64,
    last_right: f64,
    score: u32,
    lines: u32,
    over: bool,
}

impl Game {
    fn new(ctx: CanvasRenderingContext2d, seed: u32) -> Self {
        let mut rng = Rng::new(seed);
        let piece = rng.next_usize(7);
        let next_piece = rng.next_usize(7);
        let mut game = Self {
            ctx,
            board: [0; WIDTH * HEIGHT],
            piece,
            next_piece,
            x: 3,
            y: 0,
            rotation: 0,
            rng,
            input: InputState::default(),
            last_time: 0.0,
            fall_accum: 0.0,
            last_left: 0.0,
            last_right: 0.0,
            score: 0,
            lines: 0,
            over: false,
        };
        if game.collides(game.x, game.y, game.rotation) {
            game.over = true;
        }
        game
    }

    fn reset(&mut self) {
        self.board = [0; WIDTH * HEIGHT];
        self.piece = self.rng.next_usize(7);
        self.next_piece = self.rng.next_usize(7);
        self.x = 3;
        self.y = 0;
        self.rotation = 0;
        self.score = 0;
        self.lines = 0;
        self.over = false;
        self.fall_accum = 0.0;
    }

    fn fall_delay_ms(&self) -> f64 {
        let level = (self.lines / 10) as f64;
        let base = 550.0 - level * 40.0;
        base.max(120.0)
    }

    fn collides(&self, x: i32, y: i32, rotation: usize) -> bool {
        for (cx, cy) in SHAPES[self.piece][rotation].iter() {
            let px = x + i32::from(*cx);
            let py = y + i32::from(*cy);
            if px < 0 || px >= WIDTH as i32 || py < 0 || py >= HEIGHT as i32 {
                return true;
            }
            let idx = (py as usize) * WIDTH + (px as usize);
            if self.board[idx] != 0 {
                return true;
            }
        }
        false
    }

    fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        let nx = self.x + dx;
        let ny = self.y + dy;
        if !self.collides(nx, ny, self.rotation) {
            self.x = nx;
            self.y = ny;
            true
        } else {
            false
        }
    }

    fn try_rotate(&mut self) {
        let next = (self.rotation + 1) % 4;
        if !self.collides(self.x, self.y, next) {
            self.rotation = next;
            return;
        }
        if !self.collides(self.x + 1, self.y, next) {
            self.x += 1;
            self.rotation = next;
            return;
        }
        if !self.collides(self.x - 1, self.y, next) {
            self.x -= 1;
            self.rotation = next;
        }
    }

    fn lock_piece(&mut self) {
        for (cx, cy) in SHAPES[self.piece][self.rotation].iter() {
            let px = (self.x + i32::from(*cx)) as usize;
            let py = (self.y + i32::from(*cy)) as usize;
            let idx = py * WIDTH + px;
            self.board[idx] = (self.piece + 1) as u8;
        }
        self.clear_lines();
        self.spawn_next();
    }

    fn clear_lines(&mut self) {
        let mut new_board = [0u8; WIDTH * HEIGHT];
        let mut write_row = HEIGHT as i32 - 1;
        let mut cleared = 0;

        for row in (0..HEIGHT).rev() {
            let mut full = true;
            for col in 0..WIDTH {
                if self.board[row * WIDTH + col] == 0 {
                    full = false;
                    break;
                }
            }
            if full {
                cleared += 1;
                continue;
            }
            for col in 0..WIDTH {
                let dst = (write_row as usize) * WIDTH + col;
                new_board[dst] = self.board[row * WIDTH + col];
            }
            write_row -= 1;
        }

        self.board = new_board;
        if cleared > 0 {
            self.lines += cleared;
            let gain = match cleared {
                1 => 100,
                2 => 300,
                3 => 500,
                _ => 800,
            };
            self.score += gain * ((self.lines / 10) + 1);
        }
    }

    fn spawn_next(&mut self) {
        self.piece = self.next_piece;
        self.next_piece = self.rng.next_usize(7);
        self.x = 3;
        self.y = 0;
        self.rotation = 0;
        if self.collides(self.x, self.y, self.rotation) {
            self.over = true;
        }
    }

    fn update(&mut self, now: f64) {
        if self.last_time == 0.0 {
            self.last_time = now;
        }
        let dt = now - self.last_time;
        self.last_time = now;

        if self.input.restart {
            self.reset();
            self.input.restart = false;
        }
        if self.over {
            return;
        }

        if self.input.rotate {
            self.try_rotate();
            self.input.rotate = false;
        }

        if self.input.hard_drop {
            while self.try_move(0, 1) {}
            self.lock_piece();
            self.input.hard_drop = false;
            return;
        }

        let repeat_delay = 90.0;
        if self.input.left_just {
            self.try_move(-1, 0);
            self.input.left_just = false;
            self.last_left = now;
        } else if self.input.left && now - self.last_left > repeat_delay {
            self.try_move(-1, 0);
            self.last_left = now;
        }

        if self.input.right_just {
            self.try_move(1, 0);
            self.input.right_just = false;
            self.last_right = now;
        } else if self.input.right && now - self.last_right > repeat_delay {
            self.try_move(1, 0);
            self.last_right = now;
        }

        let mut fall_delay = self.fall_delay_ms();
        if self.input.down {
            fall_delay *= 0.08;
        }

        self.fall_accum += dt;
        if self.fall_accum >= fall_delay {
            if !self.try_move(0, 1) {
                self.lock_piece();
            }
            self.fall_accum = 0.0;
        }
    }

    fn draw_cell(&self, x: i32, y: i32, color: &str) {
        let px = x as f64 * BLOCK;
        let py = y as f64 * BLOCK;
        self.ctx.set_fill_style(&color.into());
        self.ctx.fill_rect(px + 1.0, py + 1.0, BLOCK - 2.0, BLOCK - 2.0);
    }

    fn render(&self) {
        let width = WIDTH as f64 * BLOCK;
        let height = HEIGHT as f64 * BLOCK;
        self.ctx.set_fill_style(&"#11131a".into());
        self.ctx.fill_rect(0.0, 0.0, width, height);

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = y * WIDTH + x;
                let color = COLORS[self.board[idx] as usize];
                self.draw_cell(x as i32, y as i32, color);
            }
        }

        if !self.over {
            for (cx, cy) in SHAPES[self.piece][self.rotation].iter() {
                let px = self.x + i32::from(*cx);
                let py = self.y + i32::from(*cy);
                self.draw_cell(px, py, COLORS[self.piece + 1]);
            }
        }

        self.ctx.set_fill_style(&"#e2e8f0".into());
        self.ctx.set_font("16px 'Fira Mono', monospace");
        self.ctx.fill_text(&format!("Score: {}", self.score), 8.0, 20.0).ok();
        self.ctx.fill_text(&format!("Lines: {}", self.lines), 8.0, 40.0).ok();

        if self.over {
            self.ctx.set_fill_style(&"rgba(15, 23, 42, 0.75)".into());
            self.ctx.fill_rect(0.0, 0.0, width, height);
            self.ctx.set_fill_style(&"#f8fafc".into());
            self.ctx.set_font("24px 'Fira Mono', monospace");
            self.ctx.fill_text("Game Over", 60.0, height / 2.0).ok();
            self.ctx.set_font("14px 'Fira Mono', monospace");
            self.ctx.fill_text("Press R to restart", 56.0, height / 2.0 + 24.0).ok();
        }
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global window")
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register raf");
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let document = window().document().expect("no document");
    let canvas = document
        .get_element_by_id("game")
        .expect("no canvas")
        .dyn_into::<HtmlCanvasElement>()?;

    let dpr = window().device_pixel_ratio();
    let width = (WIDTH as f64 * BLOCK) as u32;
    let height = (HEIGHT as f64 * BLOCK) as u32;
    canvas.set_width((width as f64 * dpr) as u32);
    canvas.set_height((height as f64 * dpr) as u32);
    canvas
        .style()
        .set_property("width", &format!("{}px", width))?;
    canvas
        .style()
        .set_property("height", &format!("{}px", height))?;

    let ctx = canvas
        .get_context("2d")?
        .expect("no context")
        .dyn_into::<CanvasRenderingContext2d>()?;
    ctx.scale(dpr, dpr)?;

    let seed = (js_sys::Date::now() as u32).wrapping_mul(1664525).wrapping_add(1013904223);
    let game = std::rc::Rc::new(std::cell::RefCell::new(Game::new(ctx, seed)));

    let keydown_game = game.clone();
    let keydown = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        let code = event.code();
        if matches!(
            code.as_str(),
            "ArrowLeft" | "ArrowRight" | "ArrowDown" | "ArrowUp" | "Space"
        ) {
            event.prevent_default();
        }
        let mut game = keydown_game.borrow_mut();
        match code.as_str() {
            "ArrowLeft" => {
                if !game.input.left {
                    game.input.left = true;
                    game.input.left_just = true;
                }
            }
            "ArrowRight" => {
                if !game.input.right {
                    game.input.right = true;
                    game.input.right_just = true;
                }
            }
            "ArrowDown" => {
                game.input.down = true;
            }
            "ArrowUp" => {
                game.input.rotate = true;
            }
            "Space" => {
                game.input.hard_drop = true;
            }
            "KeyR" => {
                game.input.restart = true;
            }
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);
    window().add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())?;
    keydown.forget();

    let keyup_game = game.clone();
    let keyup = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        let mut game = keyup_game.borrow_mut();
        match event.code().as_str() {
            "ArrowLeft" => game.input.left = false,
            "ArrowRight" => game.input.right = false,
            "ArrowDown" => game.input.down = false,
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);
    window().add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())?;
    keyup.forget();

    let f = std::rc::Rc::new(std::cell::RefCell::new(None));
    let g = f.clone();
    let loop_game = game.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |time: f64| {
        let mut game = loop_game.borrow_mut();
        game.update(time);
        game.render();
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
