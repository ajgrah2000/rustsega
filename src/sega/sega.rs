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
        const scale_x:u8 = 2;
        const scale_y:u8 = 2;
    
        let mut display_generator = graphics::display::DisplayGenerator::new(SMS_WIDTH, SMS_HEIGHT, pixels::PixelFormatEnum::RGB24); 
    
        self.main_loop(SMS_WIDTH, SMS_HEIGHT, scale_x, scale_y, &mut display_generator);
    }

    pub fn new(debug: bool, cartridge_name: String) -> Self {
        let core = Self::build_sega(debug, cartridge_name);
        Self {
            core: core,
        }
    }

    pub fn draw_loop<'a, F: FnMut(&mut [u8], usize)-> () >(&'a mut self, canvas: &mut render::Canvas<video::Window>, pixel_format: pixels::PixelFormatEnum, frame_width:u16, frame_height:u16, pixel_width:u8, pixel_height:u8, generate_display: F, iterations:u32) -> () {
        // Creating the texture creator and texture is slow, so perform multiple display updates per creation.
        let texture_creator = graphics::display::SDLUtility::texture_creator(canvas);
        let mut texture = graphics::display::SDLUtility::create_texture(&texture_creator, pixel_format, frame_width, frame_height);

        for k in 0..iterations {

            self.core.step(false);

            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {self.core.generate_display(buffer, pitch)})
                         .map_err(|e| e.to_string()).unwrap();

//            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {generate_display(buffer, pitch)})
//                         .map_err(|e| e.to_string()).unwrap();

            canvas.clear();
            canvas.copy(&texture, None, Some(rect::Rect::new(0, 0, frame_width as u32, frame_height as u32))) .map_err(|e| e.to_string()).unwrap();
            canvas.present();
        }

    }

    // Main entry point, intention is to call 'once'. 
    pub fn main_loop<'a>(&'a mut self, frame_width:u16, frame_height:u16, pixel_width:u8, pixel_height:u8, generator: &mut graphics::display::DisplayGenerator) -> () {
        let mut sdl_context = sdl2::init().unwrap();

        let mut canvas = graphics::display::SDLUtility::create_canvas(&mut sdl_context, "rust-sdl2 demo: Video", frame_width, frame_height);

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
