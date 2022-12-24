use sdl2::render::Canvas;
use sdl2::{self, pixels, rect::Rect};

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const SCALE_FACTOR: usize = 15;

pub struct Video {
    draw: bool,
    canvas: Canvas<sdl2::video::Window>,
    screen: [bool; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize],
}

impl Video {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        // initialize video
        let v_sub = sdl_context.video().unwrap();
        let window = v_sub
            .window(
                "chip8",
                (SCREEN_WIDTH * SCALE_FACTOR) as u32,
                (SCREEN_HEIGHT * SCALE_FACTOR) as u32,
            )
            .position_centered()
            .borderless()
            .opengl()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Self {
            draw: false,
            canvas,
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    // calculate index of a pixel in screen array given x and y coordinates
    fn calculate_index(x: usize, y: usize) -> usize {
        // we multiply by SCREEN_WIDTH because eventhough data is represented by 8bit rows in
        // memory, in screen it should be 64
        let x = x % SCREEN_WIDTH;
        let y = y % SCREEN_HEIGHT;
        x + SCREEN_WIDTH * y
    }

    // calculate x and y coordinates of canvas given index
    pub fn calculate_coordinates(index: usize) -> (usize, usize) {
        let y = index / SCREEN_WIDTH;
        let x = index % SCREEN_WIDTH;
        (x, y)
    }

    // we could try with 2d array as screen
    pub fn draw_screen(&mut self) {
        self.canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.canvas.set_draw_color(pixels::Color::RGB(0, 255, 0));
        for i in 0..self.screen.len() {
            if self.screen[i] {
                let (x, y) = Video::calculate_coordinates(i);
                let rect = Rect::new(
                    (x * SCALE_FACTOR) as i32,
                    (y * SCALE_FACTOR) as i32,
                    SCALE_FACTOR as u32,
                    SCALE_FACTOR as u32,
                );
                let _ = self.canvas.fill_rect(rect);
            }
        }
        self.canvas.present();
        self.draw = false;
    }

    pub fn clear_screen(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.draw_screen();
    }

    pub fn get_screen_pixel_state(&mut self, x: usize, y: usize) -> bool {
        self.screen[Video::calculate_index(x, y)]
    }

    pub fn set_screen_pixel_state(&mut self, x: usize, y: usize, state: bool) {
        self.screen[Video::calculate_index(x, y)] ^= state;
    }

    pub fn is_drawflag_set(&mut self) -> bool {
        self.draw
    }

    pub fn set_drawflag(&mut self, state: bool) {
        self.draw = state;
    }
}
