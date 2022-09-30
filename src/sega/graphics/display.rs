use sdl2::pixels;
use sdl2::render;
use sdl2::video;
use sdl2::rect;

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

    pub fn create_texture<'l>(canvas: &render::Canvas<video::Window>, texture_creator: &'l render::TextureCreator<video::WindowContext>, frame_width:u16, frame_height:u16) -> render::Texture<'l> {
        texture_creator
            .create_texture_streaming(pixels::PixelFormatEnum::RGB24, frame_width as u32, frame_height as u32)
            .map_err(|e| e.to_string()).unwrap()
    }
}

pub struct DisplayGenerator {
    current_k: u16,
}

impl DisplayGenerator {
    pub fn new() -> Self {
        Self {
            current_k: 0,
        }
    }

    pub fn new_generate_display(&mut self, buffer: &mut [u8], pitch: usize) -> () {
        // Clear the buffer
        buffer.iter_mut().for_each(|x| *x = 0);

        // Draw the display
        for i in 0..0xFF {
            for j in 0..0xFF {
                let offset = (i + 100 + (self.current_k as usize %200)) * pitch + (j + 100 + (self.current_k as usize %200)) * 3 as usize;
                buffer[offset] = 0xFF * (self.current_k as usize & 0x1) as u8;
                buffer[offset + 1] = j as u8;
                buffer[offset + 2] = i as u8;
            }
        }
        self.current_k += 1;
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

    pub fn draw_loop<'a, F: FnMut(&mut [u8], usize)-> () >(&'a mut self, canvas: &mut render::Canvas<video::Window>, frame_width:u16, frame_height:u16, pixel_width:u8, pixel_height:u8, mut generate_display: F, iterations:u32) -> () {
        // Creating the texture creator and texture is slow, so perform multiple display updates per creation.
        let texture_creator = SDLUtility::texture_creator(canvas);
        let mut texture = SDLUtility::create_texture(canvas, &texture_creator, frame_width, frame_height);

        for k in 0..iterations {
            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {generate_display(buffer, pitch)})
                         .map_err(|e| e.to_string()).unwrap();

            canvas.clear();
            canvas.copy(&texture, None, Some(rect::Rect::new(0, 0, frame_width as u32, frame_height as u32)))
                         .map_err(|e| e.to_string()).unwrap();
            canvas.present();
        }

    }

    // Main entry point, intention is to call 'once'. 
    pub fn main_loop<'a>(&'a mut self, frame_width:u16, frame_height:u16, pixel_width:u8, pixel_height:u8, generator: &mut DisplayGenerator) -> () {
        let mut sdl_context = sdl2::init().unwrap();
        let mut canvas = SDLUtility::create_canvas(&mut sdl_context, "rust-sdl2 demo: Video", frame_width, frame_height);

        // First loop.
        self.draw_loop(&mut canvas, frame_width, frame_height, pixel_width, pixel_height, generator.get_generate_display_closure(), 500);

        // Continue running it again.
        self.draw_loop(&mut canvas, frame_width, frame_height, pixel_width, pixel_height, generator.get_generate_display_closure(), 500);
    }
}

#[cfg(test)]
mod tests {
    use crate::sega::graphics::display;

    #[test]
    fn test_open_display() -> () {
        const SMS_WIDTH:u16  = 256;
        const SMS_HEIGHT:u16 = 192; // MAX HEIGHT

        let mut display_generator = display::DisplayGenerator::new(); 

        let mut sdl_display = display::SDLDisplay::new();
        sdl_display.main_loop(800, 600, 2, 2, &mut display_generator);
    }
}

