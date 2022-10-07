use sdl2::pixels;
use sdl2::render;
use sdl2::video;
use sdl2::event;
use sdl2::keyboard;
use sdl2::rect;

use super::clocks;
use super::cpu;
use super::memory;
use super::graphics;
use super::ports;
use super::interruptor;

pub struct Sega {
    core: cpu::core::Core<memory::memory::MemoryAbsolute>,
    debug: bool,
}

impl Sega {
    pub fn build_sega(debug: bool, cartridge_name: String) -> cpu::core::Core<memory::memory::MemoryAbsolute> {
        let mut cartridge = memory::cartridge::Cartridge::new(&cartridge_name);
        match cartridge.load()
        {
            Ok(()) => {println!("Ok");}
            _ => {println!("Error loading cartridge.");}
        }
    
        let clock = clocks::Clock::new();
        let mut memory = memory::memory::MemoryAbsolute::new();
        let pc_state = cpu::pc_state::PcState::new();
        let vdp = graphics::vdp::VDP::new();
        let mut ports = ports::Ports::new();
        let interruptor = interruptor::Interruptor::new();
    
        // Add the graphics device to the list of ports.
        ports.add_device(Box::new(vdp));
    
        memory.set_cartridge(cartridge);
        let core = cpu::core::Core::new(clock, memory, pc_state, ports, interruptor);
        core
    }

    pub fn power_sega(&mut self) -> () {
        const SMS_WIDTH:u16  = 256;
        const SMS_HEIGHT:u16 = 192; // MAX HEIGHT
        const SCALE_X:u8 = 3;
        const SCALE_Y:u8 = 3;
        const FRAME_WIDTH:u16  = SMS_WIDTH  * (SCALE_X as u16);
        const FRAME_HEIGHT:u16 = SMS_HEIGHT * (SCALE_Y as u16);
    
        let mut display_generator = graphics::display::DisplayGenerator::new(FRAME_WIDTH, FRAME_HEIGHT, pixels::PixelFormatEnum::RGB24); 
    
        self.main_loop(FRAME_WIDTH, FRAME_HEIGHT, SCALE_X, SCALE_Y, &mut display_generator);
    }

    pub fn new(debug: bool, cartridge_name: String) -> Self {
        let core = Self::build_sega(debug, cartridge_name);
        Self {
            core: core,
            debug: debug,
        }
    }

    pub fn draw_loop<'a, F: FnMut(&mut [u8], usize)-> () >(&'a mut self, canvas: &mut render::Canvas<video::Window>, pixel_format: pixels::PixelFormatEnum, frame_width:u16, frame_height:u16, pixel_width:u8, pixel_height:u8, generate_display: F, iterations:u32) -> () {
        // Creating the texture creator and texture is slow, so perform multiple display updates per creation.
        let texture_creator = graphics::display::SDLUtility::texture_creator(canvas);
        let mut texture = graphics::display::SDLUtility::create_texture(&texture_creator, pixel_format, frame_width/(pixel_width as u16), frame_height/(pixel_height as u16));

        for k in 0..iterations {

            // Clock the CPU lots per display update.
            for j in 0..1000 {
                self.core.step(self.debug);
            }

            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {self.core.generate_display(buffer, pitch)})
                         .map_err(|e| e.to_string()).unwrap();

            canvas.clear();
            canvas.copy(&texture, None, Some(rect::Rect::new(0, 0, (frame_width/(pixel_width as u16)) as u32, (frame_height/(pixel_width as u16)) as u32))) .map_err(|e| e.to_string()).unwrap();
            canvas.copy_ex(&texture, None, Some(rect::Rect::new(0, 0, frame_width as u32, frame_height as u32)), 0.0, None, false, false);
            canvas.present();
        }

    }

    // Main entry point, intention is to call 'once'. 
    pub fn main_loop<'a>(&'a mut self, frame_width:u16, frame_height:u16, pixel_width:u8, pixel_height:u8, generator: &mut graphics::display::DisplayGenerator) -> () {
        let mut sdl_context = sdl2::init().unwrap();

        let mut canvas = graphics::display::SDLUtility::create_canvas(&mut sdl_context, "rust-sega emulator", frame_width, frame_height);

        let mut event_pump = sdl_context.event_pump().unwrap();

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    event::Event::Quit { .. } | event::Event::KeyDown { keycode: Some(keyboard::Keycode::Escape), ..}
                     => break 'running,
                    _ => {}
                }
            }

            // First loop, draw 30 frames at a time.
            self.draw_loop(&mut canvas, generator.pixel_format, frame_width, frame_height, pixel_width, pixel_height, generator.get_generate_display_closure(), 30);
        }
    }
}
