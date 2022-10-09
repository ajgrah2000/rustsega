use sdl2::pixels;
use sdl2::rect;
use sdl2::render;
use sdl2::video;

use super::clocks;
use super::cpu;
use super::graphics;
use super::audio::sound;
use super::inputs;
use super::interruptor;
use super::memory;
use super::ports;

pub struct WindowSize {
    frame_width: u16,
    frame_height: u16,
    pixel_width: u8,
    pixel_height: u8,
}

impl WindowSize {
    fn new(frame_width: u16, frame_height: u16, pixel_width: u8, pixel_height: u8) -> Self {
        Self {
            frame_width,
            frame_height,
            pixel_width,
            pixel_height,
        }
    }
}

pub struct Sega {
    core: cpu::core::Core<memory::memory::MemoryAbsolute>,
    debug: bool,
}

impl Sega {

    const FIXED_AUDIO_QUEUE_LENGTH: u32 = 3000; // TODO: dynamically set the length based on speed/previous queue lengths, to 'minimise sound delay'

    pub fn build_sega(cartridge_name: String) -> cpu::core::Core<memory::memory::MemoryAbsolute> {
        let mut cartridge = memory::cartridge::Cartridge::new(&cartridge_name);
        match cartridge.load() {
            Ok(()) => {
                println!("Ok");
            }
            _ => {
                println!("Error loading cartridge.");
            }
        }

        let clock = clocks::Clock::new();
        let mut memory = memory::memory::MemoryAbsolute::new();
        let pc_state = cpu::pc_state::PcState::new();
        let vdp = graphics::vdp::Vdp::new();
        let mut ports = ports::Ports::new();
        let interruptor = interruptor::Interruptor::new();

        // Add the graphics device to the list of ports.
        // Joysticks are held directly, not as a 'device' (don't need to pass to ports).
        ports.add_device(Box::new(vdp));

        memory.set_cartridge(cartridge);

        cpu::core::Core::new(clock, memory, pc_state, ports, interruptor)
    }

    pub fn power_sega(&mut self) {
        const SMS_WIDTH: u16 = 256;
        const SMS_HEIGHT: u16 = 192; // MAX HEIGHT
        const SCALE_X: u8 = 3;
        const SCALE_Y: u8 = 3;
        const FRAME_WIDTH: u16 = SMS_WIDTH * (SCALE_X as u16);
        const FRAME_HEIGHT: u16 = SMS_HEIGHT * (SCALE_Y as u16);

        let window_size = WindowSize::new(FRAME_WIDTH, FRAME_HEIGHT, SCALE_X, SCALE_Y);

        self.main_loop(window_size, pixels::PixelFormatEnum::RGB24);
    }

    pub fn new(debug: bool, cartridge_name: String) -> Self {
        let core = Self::build_sega(cartridge_name);
        Self { core, debug }
    }

    pub fn draw_loop(
        &mut self,
        canvas: &mut render::Canvas<video::Window>,
        pixel_format: pixels::PixelFormatEnum,
        window_size: &WindowSize,
        iterations: u32,
        audio_queue: &mut sound::SoundQueueType,
    ) {
        // Creating the texture creator and texture is slow, so perform multiple display updates per creation.
        let texture_creator = graphics::display::SDLUtility::texture_creator(canvas);
        let mut texture = graphics::display::SDLUtility::create_texture(
            &texture_creator,
            pixel_format,
            window_size.frame_width / (window_size.pixel_width as u16),
            window_size.frame_height / (window_size.pixel_height as u16),
        );

        // Number of iterations to do before getting a new texture.
        // These loops will update the display, but currently events aren't checked in this time.
        for _k in 0..iterations {
            // Clock the CPU lots per display update.
            for _j in 0..500 {
                self.core.step(self.debug);
            }

            self.core.export();

            assert!(audio_queue.size() <= Sega::FIXED_AUDIO_QUEUE_LENGTH);
            let fill_size = Sega::FIXED_AUDIO_QUEUE_LENGTH - audio_queue.size();
            let sound_buffer = self.core.ports.audio.get_next_audio_chunk(fill_size);
            audio_queue.queue_audio(&sound_buffer).unwrap();

            texture
                .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                    self.core.generate_display(buffer)
                })
                .unwrap();

            canvas.clear();
            canvas
                .copy(
                    &texture,
                    None,
                    Some(rect::Rect::new(
                        0,
                        0,
                        (window_size.frame_width / (window_size.pixel_width as u16)) as u32,
                        (window_size.frame_height / (window_size.pixel_height as u16)) as u32,
                    )),
                )
                .unwrap();
            match canvas.copy_ex(
                &texture,
                None,
                Some(rect::Rect::new(
                    0,
                    0,
                    window_size.frame_width as u32,
                    window_size.frame_height as u32,
                )),
                0.0,
                None,
                false,
                false,
            ) {
                Ok(()) => {}
                _ => {
                    println!("Error translating texture.");
                }
            }
            canvas.present();
        }
    }

    // Main entry point, intention is to call 'once'.
    pub fn main_loop(&mut self, window_size: WindowSize, pixel_format: pixels::PixelFormatEnum) {
        let mut sdl_context = sdl2::init().unwrap();

        let mut canvas = graphics::display::SDLUtility::create_canvas(
            &mut sdl_context,
            "rust-sega emulator",
            window_size.frame_width,
            window_size.frame_height,
        );

        let mut audio_queue = sound::SDLUtility::get_audio_queue(&mut sdl_context).unwrap();

        audio_queue.resume(); // Start the audio (nothing in the queue at this point).

        let mut event_pump = sdl_context.event_pump().unwrap();

        'running: loop {
            for event in event_pump.poll_iter() {
                if !inputs::Input::handle_events(event, &mut self.core.ports.joysticks) {
                    break 'running;
                };
            }

            // First loop, draw 30 frames at a time.
            self.draw_loop(&mut canvas, pixel_format, &window_size, 30, &mut audio_queue);
        }
    }
}
