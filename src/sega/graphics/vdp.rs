use super::super::ports;

// Create a dummy VDP, to try out hooking into ports.
pub struct VDP {
    ram: Vec<u8>,
    current_address: u32,
}

impl VDP {
    const SMS_WIDTH:u16  = 256;
    const SMS_HEIGHT:u16 = 192; // MAX HEIGHT

    const FRAME_WIDTH:u16  = VDP::SMS_WIDTH;
    const FRAME_HEIGHT:u16 = VDP::SMS_HEIGHT;
    const PIXEL_WIDTH:u16  = 2;
    const PIXEL_HEIGHT:u16 = 2;

    pub fn new() -> Self {
        Self {
            ram: Vec::new(),
            current_address: 0,
        }
    }

    pub fn set_address(&mut self, value: u32) -> () {

    }
    pub fn get_address(&self) -> u32 {
        self.current_address
    }

    pub fn write_port_be(&mut self, data: u8) -> () {
        self.current_address = self.current_address + 1;
    }

    pub fn read_port_be(&mut self) -> u8 {
        self.current_address = self.current_address + 1;
        self.current_address as u8
    }

    pub fn write_port_bf(&mut self, data: u8) -> () {
        self.current_address = self.current_address + 1;
    }

    pub fn read_port_bf(&mut self) -> u8 {
        self.current_address = self.current_address + 1;
        self.current_address as u8
    }
}

impl ports::Device for VDP {
    fn port_read(&mut self, port_address: u8) -> u8 {
        match port_address & 0x1 {
            0x0 => {self.read_port_be()}
            0x1 => {self.read_port_bf()}
            _ => {0 /* Unhandled, just return 0 for now */}
        }
    }
    fn port_write(&mut self, port_address: u8, value:u8) -> () {
        match port_address & 0x1 {
            0x0 => {self.write_port_be(value);}
            0x1 => {self.write_port_bf(value);}
            _ => {}
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::sega::graphics::vdp;
    use sdl2::event;
    use sdl2::keyboard; // Keycode
    use sdl2::pixels;
    use sdl2::rect;

    impl vdp::VDP {
        pub fn driver_open_display(&mut self) -> () {
            use rand::Rng;

            let sdl_context = sdl2::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();

            let window = video_subsystem
                .window("Rusty Sega", (vdp::VDP::FRAME_WIDTH * vdp::VDP::PIXEL_WIDTH) as u32, (vdp::VDP::FRAME_HEIGHT * vdp::VDP::PIXEL_HEIGHT) as u32)
                .position_centered()
                .build()
                .unwrap();

            let mut canvas = window.into_canvas().build().unwrap();

            let mut event_pump = sdl_context.event_pump().unwrap();
            let mut i =0;
            let mut rng = rand::thread_rng();

            canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
            canvas.clear();
            i = (i + 1) % 255;
            canvas.set_draw_color(pixels::Color::RGB(i, 64, 255 - i));
            let (w, h) = canvas.output_size().unwrap();
            let mut points = [rect::Point::new(0, 0); 256];

            'running: loop {
                for event in event_pump.poll_iter() {
                    match event {
                        event::Event::Quit { .. } => break 'running,
                            event::Event::KeyDown { keycode: Some(keyboard::Keycode::Q), repeat: false, .. } => break 'running,
                            event::Event::KeyDown { ..  } => 
                            {
                                points.fill_with(|| rect::Point::new(rng.gen_range(0..w as i32), rng.gen_range(0..h as i32)));
                                canvas.draw_points(points.as_slice()).unwrap();
                                canvas.present();
                            }
                        event::Event::KeyUp { ..  } => {}
                        _ => {}
                    }
                }
            }

        }
    }

    #[test]
    #[ignore]
    fn test_open_display() {
        let mut vdp = vdp::VDP::new();

        vdp.driver_open_display();
    }

}
