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
    stop_clock: clocks::ClockType,
    fullscreen: bool,
}

impl Sega {

    const DISPLAY_UPDATES_PER_KEY_EVENT: u32 = 1; // Number of display updates per key press event. (reduces texture creation overhead).
    const CPU_STEPS_PER_AUDIO_UPDATE:    u32 = 50; // Number of times to step the CPU before updating the audio.

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
        let mut frame_width = graphics::vdp::Constants::SMS_WIDTH;
        // If not in full screen, default to using a bigger window.
        if !self.fullscreen {frame_width *= 3;}
        let frame_height = ((frame_width as u32) * (graphics::vdp::Constants::SMS_HEIGHT as u32) / (graphics::vdp::Constants::SMS_WIDTH as u32)) as u16;

        println!("powering on Sega Emulator.");
        inputs::Input::print_keys();

        let window_size = graphics::display::WindowSize::new(frame_width, frame_height, graphics::vdp::Constants::SMS_WIDTH as u16, graphics::vdp::Constants::SMS_HEIGHT as u16, self.fullscreen);

        self.main_loop(window_size, graphics::display::SDLUtility::PIXEL_FORMAT);
    }

    pub fn new(debug: bool, realtime: bool, stop_clock:clocks::ClockType, cartridge_name: String, fullscreen: bool) -> Self {
    
        let core = Self::build_sega(cartridge_name);
        Self { core, debug, realtime, stop_clock, fullscreen }
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
        let mut texture;
        texture = graphics::display::SDLUtility::create_texture(
            &texture_creator,
            pixel_format,
            window_size.console_width,
            window_size.console_height,
            );

        let mut audio_steps = 0;
        let mut display_refreshes = 0;
        while display_refreshes < iterations {

            if self.stop_clock > 0 && self.core.clock.cycles > self.stop_clock {
                return false;
            }
            self.core.step(self.debug, self.realtime);

            if 0 == audio_steps % Sega::CPU_STEPS_PER_AUDIO_UPDATE {
                // Top-up the audio queue
                sound::SDLUtility::top_up_audio_queue(audio_queue, |fill_size| {self.core.ports.audio.get_next_audio_chunk(fill_size)});
            }
            audio_steps += 1;

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
            window_size.fullscreen,
        );
        canvas.set_integer_scale(true).unwrap();
        canvas.set_logical_size(window_size.console_width as u32, window_size.console_height as u32).unwrap();

            canvas.info().texture_formats.iter().for_each(|x|
                                          match x {
    pixels::PixelFormatEnum::Unknown        => {println!("Unknown");},
    pixels::PixelFormatEnum::Index1LSB      => {println!("Index1LSB");},
    pixels::PixelFormatEnum::Index1MSB      => {println!("Index1MSB");},
    pixels::PixelFormatEnum::Index4LSB      => {println!("Index4LSB");},
    pixels::PixelFormatEnum::Index4MSB      => {println!("Index4MSB");},
    pixels::PixelFormatEnum::Index8         => {println!("Index8");},
    pixels::PixelFormatEnum::RGB332         => {println!("RGB332");},
    pixels::PixelFormatEnum::RGB444         => {println!("RGB444");},
    pixels::PixelFormatEnum::RGB555         => {println!("RGB555");},
    pixels::PixelFormatEnum::BGR555         => {println!("BGR555");},
    pixels::PixelFormatEnum::ARGB4444       => {println!("ARGB4444");},
    pixels::PixelFormatEnum::RGBA4444       => {println!("RGBA4444");},
    pixels::PixelFormatEnum::ABGR4444       => {println!("ABGR4444");},
    pixels::PixelFormatEnum::BGRA4444       => {println!("BGRA4444");},
    pixels::PixelFormatEnum::ARGB1555       => {println!("ARGB1555");},
    pixels::PixelFormatEnum::RGBA5551       => {println!("RGBA5551");},
    pixels::PixelFormatEnum::ABGR1555       => {println!("ABGR1555");},
    pixels::PixelFormatEnum::BGRA5551       => {println!("BGRA5551");},
    pixels::PixelFormatEnum::RGB565         => {println!("RGB565");},
    pixels::PixelFormatEnum::BGR565         => {println!("BGR565");},
    pixels::PixelFormatEnum::RGB24          => {println!("RGB24");},
    pixels::PixelFormatEnum::BGR24          => {println!("BGR24");},
    pixels::PixelFormatEnum::RGB888         => {println!("RGB888");},
    pixels::PixelFormatEnum::RGBX8888       => {println!("RGBX8888");},
    pixels::PixelFormatEnum::BGR888         => {println!("BGR888");},
    pixels::PixelFormatEnum::BGRX8888       => {println!("BGRX8888");},
    pixels::PixelFormatEnum::ARGB8888       => {println!("ARGB8888");},
    pixels::PixelFormatEnum::RGBA8888       => {println!("RGBA8888");},
    pixels::PixelFormatEnum::ABGR8888       => {println!("ABGR8888");},
    pixels::PixelFormatEnum::BGRA8888       => {println!("BGRA8888");},
    pixels::PixelFormatEnum::ARGB2101010    => {println!("ARGB2101010");},
    pixels::PixelFormatEnum::YV12           => {println!("YV12");},
    pixels::PixelFormatEnum::IYUV           => {println!("IYUV");},
    pixels::PixelFormatEnum::YUY2           => {println!("YUY2");},
    pixels::PixelFormatEnum::UYVY           => {println!("UYVY");},
    pixels::PixelFormatEnum::YVYU           => {println!("YVYU");},
                                          });

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
