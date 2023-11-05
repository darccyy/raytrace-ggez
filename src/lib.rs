use std::collections::HashSet;

use ggez::{
    event::{EventHandler, MouseButton},
    graphics::{self, Color, DrawMode, DrawParam, Mesh, Rect, TextLayout},
    mint::Point2,
    winit::event::VirtualKeyCode,
    Context,
};
use rand::Rng;

/// Returns `ggez::graphics::Color` value, as const
macro_rules! color {
    ($name:ident $(,)?) => {
        ::ggez::graphics::Color::$name
    };
    ($hex:literal $(,)?) => {
        color!(($hex >> 16) & 0xFF, ($hex >> 8) & 0xFF, $hex & 0xFF,)
    };
    ($r:expr, $g:expr, $b:expr $(,)?) => {
        ::ggez::graphics::Color::new(
            $r as u8 as f32 / 255.0,
            $g as u8 as f32 / 255.0,
            $b as u8 as f32 / 255.0,
            255.0,
        )
    };
    ($r:expr, $g:expr, $b:expr, $a:expr $(,)?) => {
        ::ggez::graphics::Color::new(
            $r as u8 as f32 / 255.0,
            $g as u8 as f32 / 255.0,
            $b as u8 as f32 / 255.0,
            $a as u8 as f32 / 255.0,
        )
    };
}

pub struct App {
    keys_down: HashSet<VirtualKeyCode>,
    camera: Point2<f32>,
    mouse: Point2<f32>,
    rects: Vec<(Rect, Color)>,
}

impl App {
    pub fn new(ctx: &mut Context) -> Self {
        let (width, height) = ctx.gfx.size();

        let camera = Point2 { x: 100.0, y: 100.0 };

        let mut rects = Vec::new();
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            let rw = rng.gen_range(10.0..80.0);
            let rh = rng.gen_range(10.0..80.0);

            let rect = Rect::new(
                rng.gen_range(0.0..width - rw),
                rng.gen_range(0.0..height - rh),
                rw,
                rh,
            );
            let color = color!(
                rng.gen_range(0..255),
                rng.gen_range(0..255),
                rng.gen_range(0..255),
            );
            rects.push((rect, color));
        }

        // let rects = vec![(Rect::new(200.0, 70.0, 30.0, 50.0), color!(MAGENTA))];

        Self {
            keys_down: HashSet::new(),
            camera,
            mouse: camera,
            rects,
        }
    }
    fn reset(&mut self, ctx: &mut Context) {
        *self = Self::new(ctx)
    }
}

impl EventHandler for App {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), ggez::GameError> {
        macro_rules! any_keys {
            ( $( $key:ident ),+ ) => {
                false $( || self.keys_down.contains(&VirtualKeyCode::$key) )*
            };
        }

        let x = match (any_keys!(A, H, Left), any_keys!(D, L, Right)) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };
        let y = match (any_keys!(W, K, Up), any_keys!(S, J, Down)) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };

        let speed = 5.0;
        self.camera.x += x * speed;
        self.camera.y += y * speed;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, color!(BLACK));

        for (rect, color) in &self.rects {
            let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), *rect, *color)?;
            canvas.draw(&mesh, DrawParam::default());
        }

        let radius = 10.0;
        let mesh = Mesh::new_circle(ctx, DrawMode::fill(), self.camera, radius, 0.5, color!(RED))?;
        canvas.draw(&mesh, DrawParam::default());

        let mesh = Mesh::new_circle(ctx, DrawMode::fill(), self.mouse, 5.0, 0.5, color!(BLUE))?;
        canvas.draw(&mesh, DrawParam::default());

        let mesh = Mesh::new_line(ctx, &[self.camera, self.mouse], 2.0, color!(GREEN))?;
        canvas.draw(&mesh, DrawParam::default());

        canvas.finish(ctx)
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Result<(), ggez::GameError> {
        self.mouse = Point2 { x, y };
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> Result<(), ggez::GameError> {
        // Immediate keybind
        use VirtualKeyCode as Key;
        match input.keycode {
            // Reset game
            Some(Key::R) => self.reset(ctx),
            _ => (),
        }

        // Controls
        if let Some(keycode) = input.keycode {
            self.keys_down.insert(keycode);
        }
        Ok(())
    }
    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
    ) -> Result<(), ggez::GameError> {
        if let Some(keycode) = input.keycode {
            self.keys_down.remove(&keycode);
        }
        Ok(())
    }
}
