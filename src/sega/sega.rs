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

pub struct Sega {
    core: cpu::core::Core<memory::memory::MemoryAbsolute>,
    debug: bool,
    realtime: bool,
    stop_clock: u32,
}

impl Sega {

    const DISPLAY_UPDATES_PER_KEY_EVENT: u32 = 10; // Number of display updates per key press event. (reduces texture creation overhead).

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
        const FRAME_WIDTH: u16 = 800;
        const FRAME_HEIGHT: u16 = ((FRAME_WIDTH as u32) * (SMS_HEIGHT as u32) / (SMS_WIDTH as u32)) as u16;

        println!("powering on Sega Emulator.");
        inputs::Input::print_keys();

        let window_size = graphics::display::WindowSize::new(FRAME_WIDTH, FRAME_HEIGHT, SMS_WIDTH as u16, SMS_HEIGHT as u16);

        self.main_loop(window_size, pixels::PixelFormatEnum::RGB24);
    }

    pub fn new(debug: bool, realtime: bool, stop_clock:u32, cartridge_name: String) -> Self {
        let core = Self::build_sega(cartridge_name);
        Self { core, debug, realtime, stop_clock }
    }

    pub fn draw_loop(
        &mut self,
        canvas: &mut render::Canvas<video::Window>,
        pixel_format: pixels::PixelFormatEnum,
        window_size: &graphics::display::WindowSize,
        iterations: u32,
        audio_queue: &mut sound::SoundQueueType,
    ) -> bool {
        // Number of iterations to do before getting a new texture.
        // These loops will update the display, but currently events aren't checked in this time.
        
        // Creating the texture creator and texture is slow, so perform multiple display updates per creation.
        let texture_creator = graphics::display::SDLUtility::texture_creator(canvas);
        let mut texture = graphics::display::SDLUtility::create_texture(
            &texture_creator,
            pixel_format,
            window_size.console_width,
            window_size.console_height,
            );

        let mut display_refreshes = 0;
        while display_refreshes < iterations {

            if self.stop_clock > 0 && self.core.clock.cycles > self.stop_clock {
                return false;
            }
            self.core.step(self.debug, self.realtime);

            // Top-up the audio queue
            sound::SDLUtility::top_up_audio_queue(audio_queue, |fill_size| {self.core.ports.audio.get_next_audio_chunk(fill_size)});

            // If an 'export' occurred (buffer was draw), then update the texture.
            if self.core.export() {
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
                                window_size.console_width as u32,
                                window_size.console_height as u32,
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

                display_refreshes += 1;
            }
        }
        true
    }

    // Main entry point, intention is to call 'once'.
    pub fn main_loop(&mut self, mut window_size: graphics::display::WindowSize, pixel_format: pixels::PixelFormatEnum) {
        let mut sdl_context = sdl2::init().unwrap();

        let mut canvas = graphics::display::SDLUtility::create_canvas(
            &mut sdl_context,
            "rust-sega emulator",
            window_size.frame_width,
            window_size.frame_height,
        );

        let mut audio_queue = sound::SDLUtility::get_audio_queue(&mut sdl_context).unwrap();

        audio_queue.clear(); 
        audio_queue.resume(); // Start the audio (nothing in the queue at this point).

        let mut event_pump = sdl_context.event_pump().unwrap();

        'running: loop {
            for event in event_pump.poll_iter() {

                graphics::display::SDLUtility::handle_events(&event, &mut window_size);

                if !inputs::Input::handle_events(event, &mut self.core.ports.joysticks) {
                    break 'running;
                };
            }

            // First loop, draw FRAMES_PER_KEY_EVENT frames at a time.
            if !self.draw_loop(&mut canvas, pixel_format, &window_size, Sega::DISPLAY_UPDATES_PER_KEY_EVENT, &mut audio_queue) {
                break 'running;
            }
        }
    }
}
