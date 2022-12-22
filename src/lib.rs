use rand::Rng;
use std::fs::File;
use std::io::Read;

mod drivers;

const FONTSET_SIZE: usize = 0x50;
const START_ADDRESS: usize = 0x200;

struct Cpu {
    // 16-bit program counter
    pc: u16,

    // 16-bit index register
    index: u16,

    // stack pointer
    sp: u16,

    v: [u8; 16],
}

struct Memory {
    // memory of the chip8. in truth this is too much. 4096 is enough for 12-bit addresses
    memory: [u8; 4096],

    // stack to store 16 16-bit addresses
    stack: [u16; 16],
}

// There are 2 timers in Chip8, both are 60Hz and once above 0, timers will decrement itself to 0
struct Timers {
    // delay timer
    delay: u8,

    // sound timer
    sound: u8,
}

pub struct Media {
    pub sound: i32, // TODO

    // display of the chip8 is 2048 pixels, each pixel can be either black or white
    pub display: drivers::Video,

    // chip8 has a hex keypad (0x0 - 0xf)
    pub keypad: drivers::Keypad,
}

pub struct Chip8 {
    cpu: Cpu,
    memory: Memory,
    timers: Timers,
    pub media: Media,
}

impl Chip8 {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        Self {
            cpu: Cpu {
                pc: 0x0,
                index: 0x0,
                sp: 0x0,
                v: [0x0; 16],
            },
            memory: Memory {
                memory: [0x0; 4096],
                stack: [0x0; 16],
            },
            timers: Timers {
                delay: 0x0,
                sound: 0x0,
            },
            media: Media {
                sound: 0x0,
                display: drivers::Video::new(sdl_context),
                keypad: drivers::Keypad::new(sdl_context),
            },
        }
    }

    pub fn load_and_init(&mut self, filepath: &str) {
        // setup font
        let chip8_font = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        self.memory.memory[..FONTSET_SIZE].copy_from_slice(&chip8_font);

        // load the game to memory
        let mut f = File::open(filepath).expect("ROM not found");
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        self.memory.memory[START_ADDRESS..START_ADDRESS + buffer.len()].copy_from_slice(&buffer);
        self.cpu.pc = START_ADDRESS as u16;
    }

    pub fn emulate_cycle(&mut self) {
        self.media.keypad.poll();
        let opcode = self.fetch_instr();
        if let Some(opcode) = opcode {
            self.execute_instr(opcode);
        }
        self.update_timers();
    }

    pub fn is_drawflag_set(&mut self) -> bool {
        self.media.display.is_drawflag_set()
    }

    fn fetch_instr(&mut self) -> Option<u16> {
        if self.cpu.pc >= 4096 || self.cpu.pc < 0x200 {
            return None;
        }
        let mut opcode: u16 = (self.memory.memory[self.cpu.pc as usize] as u16) << 8;
        opcode |= self.memory.memory[(self.cpu.pc + 1) as usize] as u16;
        self.cpu.pc += 2;
        Some(opcode)
    }

    fn execute_instr(&mut self, opcode: u16) {
        match opcode & 0xf000 {
            0x0000 => {
                match opcode & 0x00ff {
                    0x00e0 => self.media.display.clear_screen(),
                    0x00ee => {
                        // return from subroutine
                        self.cpu.pc = self.memory.stack[(self.cpu.sp) as usize];
                        self.memory.stack[(self.cpu.sp) as usize] = 0x0;
                        self.cpu.sp -= 1;
                    }
                    _ => {}
                }
            }
            0x1000 => {
                // jump to address
                self.cpu.pc = opcode & 0x0fff;
            }
            0x2000 => {
                // call subroutine
                self.memory.stack[(self.cpu.sp) as usize] = self.cpu.pc;
                self.cpu.sp += 1;
                self.cpu.pc = opcode & 0x0fff;
            }
            0x3000 => {
                // skip next instruction if Vx == NN
                if self.cpu.v[((opcode & 0x0f00) >> 8) as usize] == (opcode & 0x00ff) as u8 {
                    self.cpu.pc += 2;
                }
            }
            0x4000 => {
                // skip next instruction if Vx != NN
                if self.cpu.v[((opcode & 0x0f00) >> 8) as usize] != (opcode & 0x00fff) as u8 {
                    self.cpu.pc += 2;
                }
            }
            0x5000 => {
                // skip next instruction if Vx == Vy
                if self.cpu.v[((opcode & 0x0f00) >> 8) as usize]
                    == self.cpu.v[((opcode & 0x00f0) >> 4) as usize]
                {
                    self.cpu.pc += 2;
                }
            }
            0x6000 => {
                // set Vx to NN
                self.cpu.v[((opcode & 0x0f00) >> 8) as usize] = (opcode & 0x00ff) as u8;
            }
            0x7000 => {
                // set Vx = Vx + NN
                let result =
                    self.cpu.v[((opcode & 0x0f00) >> 8) as usize] as u16 + (opcode & 0x00ff) as u16;
                self.cpu.v[((opcode & 0x0f00) >> 8) as usize] = result as u8;
            }
            0x8000 => {
                match opcode & 0x800f {
                    0x8000 => {
                        // set Vx = Vy
                        self.cpu.v[((opcode & 0x0f00) >> 8) as usize] =
                            self.cpu.v[((opcode & 0x00f0) >> 4) as usize];
                    }
                    0x8001 => {
                        // set Vx |= Vy
                        self.cpu.v[((opcode & 0x0f00) >> 8) as usize] |=
                            self.cpu.v[((opcode & 0x00f0) >> 4) as usize]
                    }
                    0x8002 => {
                        // set Vx &= Vy
                        self.cpu.v[((opcode & 0x0f00) >> 8) as usize] &=
                            self.cpu.v[((opcode & 0x00f0) >> 4) as usize]
                    }
                    0x8003 => {
                        // set Vx ^= Vy
                        self.cpu.v[((opcode & 0x0f00) >> 8) as usize] ^=
                            self.cpu.v[((opcode & 0x00f0) >> 4) as usize]
                    }
                    0x8004 => {
                        // Vx = Vx + Vy ; if carry then v[f] = 1; else v[f] = 0;
                        let result: u16 = (self.cpu.v[((opcode & 0x0f00) >> 8) as usize]
                            + self.cpu.v[((opcode & 0x00f0) >> 4) as usize])
                            as u16;
                        self.cpu.v[((opcode & 0x0f00) >> 8) as usize] = result as u8;
                        if result > 0xff {
                            self.cpu.v[0xf] = 1;
                        } else {
                            self.cpu.v[0xf] = 0;
                        }
                    }
                    0x8005 => {
                        // Vx = Vx - Vy ; if borrow then v[f] = 1; else v[f] = 0;
                        if self.cpu.v[((opcode & 0x0f00) >> 8) as usize]
                            > self.cpu.v[((opcode & 0x00f0) >> 4) as usize]
                        {
                            self.cpu.v[0xf] = 1;
                        } else {
                            self.cpu.v[0xf] = 0;
                        }
                        self.cpu.v[((opcode & 0x0f00) >> 8) as usize] = self.cpu.v
                            [((opcode & 0x0f00) >> 8) as usize]
                            .wrapping_sub(self.cpu.v[((opcode & 0x00f0) >> 4) as usize]);
                    }
                    0x8006 => {
                        // set Vx >>= 1
                        self.cpu.v[0xf] = self.cpu.v[((opcode & 0x0f00) >> 8) as usize] & 1;
                        self.cpu.v[((opcode & 0x0f00) >> 8) as usize] >>= 1;
                    }
                    0x8007 => {
                        if self.cpu.v[((opcode & 0x00f0) >> 4) as usize]
                            > self.cpu.v[((opcode & 0x0f00) >> 8) as usize]
                        {
                            self.cpu.v[0xf] = 1;
                        } else {
                            self.cpu.v[0xf] = 0;
                        }
                        self.cpu.v[((opcode & 0x0f00) >> 8) as usize] = self.cpu.v
                            [((opcode & 0x00f0) >> 4) as usize]
                            .wrapping_sub(self.cpu.v[((opcode & 0x0f00) >> 8) as usize]);
                    }
                    0x800e => {
                        // set Vx <<= 1
                        self.cpu.v[0xf] =
                            (self.cpu.v[((opcode & 0x0f00) >> 8) as usize] & 0b1000000) >> 7;
                        self.cpu.v[((opcode & 0x0f00) >> 8) as usize] <<= 1;
                    }
                    _ => {}
                }
            }
            0x9000 => {
                // if (Vx != Vy) then pc++
                if self.cpu.v[((opcode & 0x0f00) >> 8) as usize]
                    != self.cpu.v[((opcode & 0x00f0) >> 4) as usize]
                {
                    self.cpu.pc += 2;
                }
            }
            0xa000 => {
                // cpu.i = 0xaNNN
                self.cpu.index = opcode & 0x0fff;
            }
            0xb000 => {
                // unconditional jump
                self.cpu.pc = (self.cpu.v[0] as u16) + (opcode & 0x0fff);
            }
            0xc000 => {
                // generate random number and and it with nn
                let mut rng = rand::thread_rng();
                self.cpu.v[((opcode & 0x0f00) >> 8) as usize] =
                    rng.gen::<u8>() & ((opcode & 0x00ff) as u8);
            }
            0xd000 => {
                // draw sprite
                // Vx and Vy specifies coordinates which the sprite should be drawn
                // Even though the width of a sprite is set to 8bits, height is specified in the N
                // so total bits we will be reading is N * 8
                let x_coord = self.cpu.v[((opcode & 0x0f00) >> 8) as usize];
                let y_coord = self.cpu.v[((opcode & 0x00f0) >> 4) as usize];
                let height = (opcode & 0x000f) as u8; // height
                self.cpu.v[0xf] = 0;
                // loop through each row of the sprite
                for yline in 0..height {
                    // sprite that should be drawn is in row by row at address specified by index
                    // register
                    let pixels = self.memory.memory[(self.cpu.index + yline as u16) as usize];
                    for xline in 0..8 {
                        // if the bit is in memory and corresponding pixel is not 0, we set v[0xf]
                        // = 1
                        if (pixels & (0b1000000 >> xline)) != 0 {
                            if self
                                .media
                                .display
                                .get_screen_pixel_state(x_coord + xline, y_coord + yline)
                            {
                                self.cpu.v[0xf] = 1;
                            }
                            self.media.display.set_screen_pixel_state(
                                x_coord + xline,
                                y_coord,
                                true,
                            )
                        }
                    }
                }
                self.media.display.set_drawflag(true);
            }
            0xe000 => match opcode & 0xe0ff {
                0xe09e => {
                    // skip instruction if key index in Vx is pressed
                    let key_index = self.cpu.v[((opcode & 0x0f00) >> 8) as usize];
                    if self.media.keypad.is_key_pressed(key_index as usize) {
                        self.cpu.pc += 2
                    }
                }
                0xe0a1 => {
                    // skip instruction if key index in Vx is not pressed
                    let key_index = self.cpu.v[((opcode & 0x0f00) >> 8) as usize];
                    if !self.media.keypad.is_key_pressed(key_index as usize) {
                        self.cpu.pc += 2
                    }
                }
                _ => {}
            },
            0xf000 => {
                match opcode & 0xf0ff {
                    0xf007 => {
                        // set Vx = get_delay
                        self.cpu.v[((opcode & 0x0f00) >> 8) as usize] = self.timers.delay;
                    }
                    0xf00a => {
                        // wait for key press
                        // block until key is pressed. if key is pressed, store it in Vx, if
                        // multiple keys are pressed, save the one with lowest index.
                        // we can t use a rust loop or any kind of infinity looops because we want
                        // the cpu to execute key pressed opcode in case of a key press and update
                        // keypad states
                        let mut key_pressed = false;
                        for key_index in 0..16 {
                            if self.media.keypad.is_key_pressed(key_index) {
                                self.cpu.v[((opcode & 0x0f00) >> 4) as usize] = key_index as u8;
                                key_pressed = true;
                                break;
                            }
                        }
                        if !key_pressed {
                            // if key is not pressed, execute this in
                            self.cpu.pc -= 2;
                        }
                    }
                    0xf015 => {
                        self.timers.delay = self.cpu.v[((opcode & 0x0f00) >> 8) as usize];
                    }
                    0xf018 => {
                        self.timers.sound = self.cpu.v[((opcode & 0x0f00) >> 8) as usize];
                    }
                    0xf01e => {
                        self.cpu.index += self.cpu.v[((opcode & 0x0f00) >> 8) as usize] as u16;
                    }
                    0xf029 => {
                        // set index register to the location of a sprite
                        self.cpu.index = self.cpu.v[((opcode & 0x0f00) >> 8) as usize] as u16;
                    }
                    0xf033 => {
                        // convert to BCD
                        let value = self.cpu.v[((opcode & 0x0f00) >> 8) as usize] as f32;
                        let of_hundreds = (value / 100.0).floor() as u8;
                        let of_tens = ((value / 10.0) % 10.0) as u8;
                        let of_ones = (value % 10.0) as u8;

                        self.memory.memory[self.cpu.index as usize] = of_hundreds;
                        self.memory.memory[(self.cpu.index + 1) as usize] = of_tens;
                        self.memory.memory[(self.cpu.index + 2) as usize] = of_ones;
                    }
                    0xf055 => {
                        // store regs until n in memory start by address in index register
                        let n = self.cpu.v[((opcode & 0x0f00) >> 8) as usize];
                        for i in 0..=n {
                            self.memory.memory[(self.cpu.index as usize + i as usize)] =
                                self.cpu.v[(i) as usize];
                        }
                    }
                    0xf065 => {
                        // load to regs until n from memory start by address in index register
                        let n = self.cpu.v[((opcode & 0x0f00) >> 8) as usize];
                        for i in 0..=n {
                            self.cpu.v[(i) as usize] =
                                self.memory.memory[self.cpu.index as usize + i as usize];
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn update_timers(&mut self) {
        if self.timers.delay > 0 {
            self.timers.delay -= 1;
        }
        if self.timers.sound > 0 {
            if self.timers.sound == 1 {
                println!("beep");
            }
            self.timers.sound -= 1;
        }
    }
}
