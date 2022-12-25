use sdl2;
use sdl2::audio::{AudioDevice, AudioCallback};

pub struct Audio {
    dev: AudioDevice<SquareWave>
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

impl Audio {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let a_sub = sdl_context.audio().unwrap();
        let spec = sdl2::audio::AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };
        let dev = a_sub.open_playback(None, &spec, |spec| {
            // callback
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        }).unwrap();

        Self { dev }
    }

    pub fn beep(&self) {
        self.dev.resume();
    }

    pub fn beep_stop(&self) {
        self.dev.pause();
    }
}
