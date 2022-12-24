use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct Keypad {
    keypad: [bool; 16],
    events: sdl2::EventPump,
}

impl Keypad {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        Self {
            keypad: [false; 16],
            events: sdl_context.event_pump().unwrap(),
        }
    }

    pub fn get_keypad_len(&mut self) -> usize {
        self.keypad.len()
    }

    pub fn is_key_pressed(&mut self, index: usize) -> bool {
        self.keypad[index]
    }

    pub fn clear_keyboard(&mut self) {
        self.keypad = [false; 16];
    }

    pub fn poll(&mut self) {
        for each in self.events.poll_iter() {
            if let Event::Quit { .. } = each {
                println!("exiting");
                std::process::exit(0);
            }
        }

        // collself.media.display.set_drawflag(false);ect all the keys pressed
        let keys: Vec<Keycode> = self
            .events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        for key in keys {
            let key_index = match key {
                Keycode::Num1 => Some(1),
                Keycode::Num2 => Some(2),
                Keycode::Num3 => Some(3),
                Keycode::Num4 => Some(0xc),
                Keycode::Q => Some(4),
                Keycode::W => Some(5),
                Keycode::E => Some(6),
                Keycode::R => Some(0xd),
                Keycode::A => Some(7),
                Keycode::S => Some(8),
                Keycode::D => Some(9),
                Keycode::F => Some(0xe),
                Keycode::Z => Some(0xa),
                Keycode::X => Some(0x0),
                Keycode::C => Some(0xb),
                Keycode::V => Some(0xf),
                _ => None,
            };
            if let Some(index) = key_index {
                self.keypad[index] = true;
            }
        }
    }
}
