use ggez::conf::WindowMode;
use ggez::event;
use ggez::ContextBuilder;
use ggez::GameResult;

use raytrace::App;

fn main() -> GameResult {
    let window_mode = WindowMode::default().dimensions(600.0, 400.0);

    // Create app context
    let (mut ctx, event_loop) = ContextBuilder::new("raytrace", "darcy")
        .window_mode(window_mode)
        .build()?;

    // Change window properties
    ctx.gfx.set_window_title("raytrace");

    // Create app state
    let app = App::new(&mut ctx);

    // Run game loop
    event::run(ctx, event_loop, app);
}
