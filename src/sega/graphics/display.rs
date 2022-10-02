use sdl2::pixels;
use sdl2::render;
use sdl2::video;
use sdl2::rect;
use sdl2::event;
use sdl2::keyboard;
use std::iter::FromIterator;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

type ColourType = Vec<u8>;

#[derive(Clone)]
pub struct Colour {
    // Simple RGB store and conversion at a per colour level.
    r: u8, 
    g: u8, 
    b: u8,
}

impl Colour {
    pub fn new(r: u8, g: u8, b: u8) -> Self  {
        Self {
            r: r, 
            g: g, 
            b: b,
        }
    }

    pub fn convert(&self, pixel_format: pixels::PixelFormatEnum) -> ColourType {
        match pixel_format {
            pixels::PixelFormatEnum::RGB24 => {let mut v = Vec::new();
                                               v.push(self.r);
                                               v.push(self.g);
                                               v.push(self.b);
                                               v
                                               },
            _ => panic!("unrecognised pixel format"),
        }
    }
}

struct Colours {
    pub colour_lookup: Vec<Colour>,
}

impl Colours {
    // The colours palette is specified by a file with 128 lines.
    // R G B # Comment
    // The index in the display is then used with this as a lookup for the raw
    // RGB used for the video mode that's been set.
    const PALETTE_SIZE: u16 = 128;

    pub fn new() -> Self {
        Self {
            colour_lookup: Vec::new(),
        }
    }

    fn read_palette(&mut self, pixel_format: pixels::PixelFormatEnum, filename: &str) -> ()
    {
        let file = match File::open(filename) {
            Err(why) => panic!("couldn't open {}: {}", filename, why),
            Ok(file) => file,
        };
        let reader = BufReader::new(file);
        for line in reader.lines() {
            match line {
                Ok(line) => {let values: Vec<&str> = line.split(' ').collect();
                             let r = values[0].parse::<u8>().unwrap();
                             let g = values[1].parse::<u8>().unwrap();
                             let b = values[2].parse::<u8>().unwrap();
                             // Ignore additional split (likely comment)
                             self.colour_lookup.push(Colour::new(r, g, b));},
                Err(why) => panic!("parsing {}: {}", filename, why),
            }
        }

        if self.colour_lookup.len() != Colours::PALETTE_SIZE as usize {
            panic!("Incorrect number of colours in palette file {}", filename);
        }
    }
}

pub struct SDLUtility
{
}

impl SDLUtility {
    pub fn create_canvas(sdl_context: &mut sdl2::Sdl, name: &str, frame_width:u16, frame_height:u16) -> render::Canvas<video::Window> {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(name, frame_width as u32, frame_height as u32)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string()).unwrap();

        window.into_canvas().build().map_err(|e| e.to_string()).unwrap()
    }

    pub fn texture_creator(canvas: &render::Canvas<video::Window>) -> render::TextureCreator<video::WindowContext> {
        canvas.texture_creator()
    }

    pub fn create_texture<'l>(texture_creator: &'l render::TextureCreator<video::WindowContext>, pixel_format: pixels::PixelFormatEnum, frame_width:u16, frame_height:u16) -> render::Texture<'l> {
        texture_creator
            .create_texture_streaming(pixel_format, frame_width as u32, frame_height as u32)
            .map_err(|e| e.to_string()).unwrap()
    }
}

pub struct DisplayGenerator {
    current_k: u16,
    pub pixel_format: pixels::PixelFormatEnum,
    pitch: u16,
    width: u16,
    height: u16,
    display: Vec<u8>,
}

impl DisplayGenerator {
    pub fn new(width:u16, height:u16, pixel_format: pixels::PixelFormatEnum) -> Self {
        let pitch = match pixel_format {pixels::PixelFormatEnum::RGB24 => {width * 3},
                                       _ => {0}};
        Self {
            current_k: 0,
            pixel_format: pixel_format,
            pitch: pitch,
            width: width,
            height: height,
            display: vec![0;((height as u32)*(pitch as u32)) as usize]
        }
    }

    pub fn update_display(&mut self) -> () {
        // Clear the buffer
        self.display.iter_mut().for_each(|x| *x = 0);

        // Draw the display
        for i in 0..0xFF {
            for j in 0..0xFF {
                let offset = (i + 100 + (self.current_k as usize %200)) * (self.pitch as usize) + (j + 100 + (self.current_k as usize %200)) * 3 as usize;
                self.display[offset] = 0xFF * (self.current_k as usize & 0x0) as u8;
                self.display[offset + 1] = j as u8;
                self.display[offset + 2] = i as u8;
            }
        }
        self.current_k += 1;
    }

    pub fn new_generate_display(&mut self, buffer: &mut [u8], pitch: usize) -> () {
        assert_eq!(self.pitch as usize, pitch);
        // Update the graphics.
        self.update_display();

        // Copy the graphics to the buffer
        buffer.clone_from_slice(self.display.as_slice());
    }

    pub fn get_generate_display_closure<'l>(&'l mut self) -> impl FnMut(&mut [u8], usize) -> () + 'l{
        |buffer, pitch| {self.new_generate_display(buffer, pitch)}
    }
}

pub struct SDLDisplay {
}

impl SDLDisplay {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn draw_loop<'a, F: FnMut(&mut [u8], usize)-> () >(&'a mut self, canvas: &mut render::Canvas<video::Window>, pixel_format: pixels::PixelFormatEnum, frame_width:u16, frame_height:u16, pixel_width:u8, pixel_height:u8, mut generate_display: F, iterations:u32) -> () {
        // Creating the texture creator and texture is slow, so perform multiple display updates per creation.
        let texture_creator = SDLUtility::texture_creator(canvas);
        let mut texture = SDLUtility::create_texture(&texture_creator, pixel_format, frame_width, frame_height);

        for k in 0..iterations {
            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {generate_display(buffer, pitch)})
                         .map_err(|e| e.to_string()).unwrap();

            canvas.clear();
            canvas.copy(&texture, None, Some(rect::Rect::new(0, 0, frame_width as u32, frame_height as u32))) .map_err(|e| e.to_string()).unwrap();
            canvas.present();
        }

    }

    // Main entry point, intention is to call 'once'. 
    pub fn main_loop<'a>(&'a mut self, frame_width:u16, frame_height:u16, pixel_width:u8, pixel_height:u8, generator: &mut DisplayGenerator) -> () {
        let mut sdl_context = sdl2::init().unwrap();

        let mut canvas = SDLUtility::create_canvas(&mut sdl_context, "rust-sdl2 demo: Video", frame_width, frame_height);

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

#[cfg(test)]
mod tests {
    use sdl2::pixels;
    use crate::sega::graphics::display;
    use super::*;

    #[test]
    fn test_open_display() -> () {
        const SMS_WIDTH:u16  = 256;
        const SMS_HEIGHT:u16 = 192; // MAX HEIGHT
        const window_width:u16  = 800;
        const window_height:u16 = 600; // MAX HEIGHT

        let mut display_generator = display::DisplayGenerator::new(window_width, window_height, pixels::PixelFormatEnum::RGB24); 

        let mut sdl_display = display::SDLDisplay::new();
        sdl_display.main_loop(window_width, window_height, 2, 2, &mut display_generator);
    }

    #[test]
    fn test_read_palette() -> () {
        let mut colours = Colours::new();
        colours.read_palette(pixels::PixelFormatEnum::RGB24, "palette.ntsc.dat");
    }
}

