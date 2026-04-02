// Program entry point; delegates to the SDL2 app runner.
mod app;
mod input;
mod render;
mod sim;
mod stats;

fn main() -> Result<(), String> {
    app::run()
}
