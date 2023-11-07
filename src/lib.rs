use std::{collections::HashSet, f32::consts::PI};

use ggez::{
    event::EventHandler,
    graphics::{self, Color, DrawMode, DrawParam, Mesh, Rect},
    mint::Point2,
    winit::event::VirtualKeyCode,
    Context,
};
use rand::Rng;

const RAY_COUNT: usize = 300;
const RAY_FOV_RADIANS: f32 = PI * 0.6;
const RAY_DISTANCE_STEP: f32 = 3.0;
const RAY_MAX_DISTANCE: f32 = 400.0;
const RAY_FADE_DISTANCE: f32 = 400.0;

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
    direction: f32,
    rects: Vec<(Rect, Color)>,
    top_down_view: bool,
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

        Self {
            keys_down: HashSet::new(),
            camera,
            mouse: camera,
            direction: 0.0,
            rects,
            top_down_view: false,
        }
    }
    fn reset(&mut self, ctx: &mut Context) {
        *self = Self::new(ctx)
    }

    fn mouse_direction(&self) -> f32 {
        (self.mouse.y - self.camera.y).atan2(self.mouse.x - self.camera.x)
    }
}

impl EventHandler for App {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), ggez::GameError> {
        macro_rules! any_keys {
            ( $( $key:ident ),+ ) => {
                false $( || self.keys_down.contains(&VirtualKeyCode::$key) )*
            };
        }

        let speed = 5.0;
        let turn_speed = 0.05;

        if self.top_down_view {
            // Move x and y normally
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
            self.camera.x += x * speed;
            self.camera.y += y * speed;
        } else {
            // Turn camera
            let x = match (any_keys!(A, H, Left), any_keys!(D, L, Right)) {
                (true, false) => -1.0,
                (false, true) => 1.0,
                _ => 0.0,
            };
            if x != 0.0 {
                self.direction += x * turn_speed;
            }

            // Move forward or back
            let y = match (any_keys!(W, K, Up), any_keys!(S, J, Down)) {
                (true, false) => 1.0,
                (false, true) => -1.0,
                _ => 0.0,
            };
            if y != 0.0 {
                self.camera.x += y * speed * self.direction.cos();
                self.camera.y += y * speed * self.direction.sin();
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, color!(BLACK));
        let (width, height) = ctx.gfx.size();

        if self.top_down_view {
            for (rect, color) in &self.rects {
                let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), *rect, *color)?;
                canvas.draw(&mesh, DrawParam::default());
            }
        }

        for i in 0..RAY_COUNT {
            let ray = i as f32 / (RAY_COUNT - 1) as f32;
            let ray_dir = self.direction + (ray - 0.5) * RAY_FOV_RADIANS;

            let rects_collision_check = |point| {
                for (rect, color) in &self.rects {
                    if is_point_in_rect(point, &rect) {
                        return Some(*color);
                    }
                }
                None
            };

            let ray_hit = ray_cast(self.camera, ray_dir, rects_collision_check);

            if let Some(RayHit {
                mut color,
                point,
                distance,
            }) = ray_hit
            {
                let color_value = 1.0 - distance / RAY_FADE_DISTANCE;

                if self.top_down_view {
                    color.a = color_value;

                    let mesh = Mesh::new_line(ctx, &[self.camera, point], 2.0, color)?;
                    canvas.draw(&mesh, DrawParam::default());
                } else {
                    color.r *= color_value;
                    color.g *= color_value;
                    color.b *= color_value;

                    let rect = Rect::new(width * ray, 0.0, width / RAY_COUNT as f32 + 1.0, height);
                    let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, color)?;
                    canvas.draw(&mesh, DrawParam::default());
                }
            }
        }

        if self.top_down_view {
            let radius = 10.0;
            let mesh =
                Mesh::new_circle(ctx, DrawMode::fill(), self.camera, radius, 0.5, color!(RED))?;
            canvas.draw(&mesh, DrawParam::default());
        }

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
        if self.top_down_view {
            self.direction = self.mouse_direction();
        }
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
            // Toggle top down view
            Some(Key::Space) => self.top_down_view ^= true,
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

struct RayHit {
    color: Color,
    point: Point2<f32>,
    distance: f32,
}

fn ray_cast(
    origin: Point2<f32>,
    direction: f32,
    collision_check: impl Fn(Point2<f32>) -> Option<Color>,
) -> Option<RayHit> {
    let ray_step_count = (RAY_MAX_DISTANCE / RAY_DISTANCE_STEP).ceil() as usize;

    for j in 0..ray_step_count {
        let distance = j as f32 * RAY_DISTANCE_STEP;
        let point = Point2 {
            x: origin.x + distance * direction.cos(),
            y: origin.y + distance * direction.sin(),
        };

        if let Some(color) = collision_check(point) {
            return Some(RayHit {
                color,
                point,
                distance,
            });
        }
    }
    None
}

fn is_point_in_rect(point: Point2<f32>, rect: &Rect) -> bool {
    let Point2 { x, y } = point;
    rect.x <= x && rect.y <= y && rect.x + rect.w > x && rect.y + rect.h > y
}
