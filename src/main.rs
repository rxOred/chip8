extern crate sdl2;

use chip8::Chip8;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("ROM not specified");
    }

    let sdl_context = sdl2::init().unwrap();

    let mut chip8 = Chip8::new(&sdl_context);
    chip8.load_and_init(args[1].as_str());

    loop {
        chip8.emulate_cycle();
        if chip8.is_drawflag_set() {
            chip8.media.display.draw_screen();
        }
    }
}
