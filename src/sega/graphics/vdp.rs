use super::super::ports;

pub struct Constants {
}

impl Constants {
    const RAMSIZE:u16  = 0x4000;
    const CRAMSIZE:u8  = 0x20;
    // 3Mhz CPU, 50Hz refresh ~= 60000 ticks
    const VSYNCCYCLETIME:u32 = 65232;
    const BLANKTIME:u16      = ((Constants::VSYNCCYCLETIME as u32 * 72)/262) as u16;
    const VFRAMETIME:u16     = ((Constants::VSYNCCYCLETIME as u32 * 192)/262) as u16;
    const HSYNCCYCLETIME:u16 = 216;

    const REGISTERMASK:u8  = 0x0F;
    const REGISTERUPDATEMASK:u8  = 0xF0;
    const REGISTERUPDATEVALUE:u8 = 0x80;
    const NUMVDPREGISTERS:u8 = 16;

    // VDP status register
    const VSYNCFLAG:u8   = 0x80;

    // VDP register 0
    const MODE_CONTROL_NO_1:u8 = 0x0;
    const VDP0DISVSCROLL:u8    = 0x80;
    const VDP0DISHSCROLL:u8    = 0x40;
    const VDP0COL0OVERSCAN:u8  = 0x20;
    const VDP0LINEINTENABLE:u8 = 0x10;
    const VDP0SHIFTSPRITES:u8  = 0x08;
    const VDP0M4:u8            = 0x04;
    const VDP0M2:u8            = 0x02;
    const VDP0NOSYNC:u8        = 0x01;

    // VDP register 1
    const MODE_CONTROL_NO_2:u8 = 0x1;
    const VDP1BIT7:u8          = 0x80;
    const VDP1ENABLEDISPLAY:u8 = 0x40;
    const VDP1VSYNC:u8         = 0x20;
    const VDP1M1:u8            = 0x10;
    const VDP1M3:u8            = 0x08;
    const VDP1BIGSPRITES:u8    = 0x02;
    const VDP1DOUBLESPRITES:u8 = 0x01;

    const NAMETABLEPRIORITY:u8 = 0x10;
    const NUMSPRITES:u8 = 64;

    const DMM4:u8 = 0x8;
    const DMM3:u8 = 0x4;
    const DMM2:u8 = 0x2;
    const DMM1:u8 = 0x1;

    const PALETTE_ADDRESS:u16  = 0xC000;

    const SMS_WIDTH:u16  = 256;
    const SMS_HEIGHT:u16 = 192; // MAX HEIGHT
    const SMS_COLOR_DEPTH:u8 = 16;

    const MAXPATTERNS:u16 = 512;
    const PATTERNWIDTH:u8  = 8;
    const PATTERNHEIGHT:u8 = 8;
    const PATTERNSIZE:u8 = 64;

    const MAXPALETTES:u8 = 2;

    const NUMTILEATTRIBUTES:u16 = 0x700;
    const TILEATTRIBUTEMASK:u16     = 0x7FF;
    const TILEATTRIBUTESADDRESSMASK:u16 = 0x3800;
    const TILEATTRIBUTESTILEMASK:u16 = 0x07FE;
    const TILESHIFT:u8 = 1;
    const TILEATTRIBUTESHMASK:u16    = 0x0001;
    const TILEPRIORITYSHIFT:u8 = 4;
    const TILEPALETTESHIFT:u8 = 3;
    const TILEVFLIPSHIFT:u8 = 2;
    const TILEHFLIPSHIFT:u8 = 1;

    const YTILES:u8 = 28;
    const XTILES:u8 = 32;
    const NUMTILES:u16 = Constants::XTILES as u16 * Constants::YTILES as u16 ;

    const SPRITEATTRIBUTESADDRESSMASK:u16 = 0x3F00;
    const SPRITEATTRIBUTESMASK:u16 = 0x00FF;
    const NUMSPRITEATTRIBUTES:u16 = 0x00FF;

    const SPRITETILEMASK:u16 = 0x0001;

    const LASTSPRITETOKEN:u16 = 0xD0;
    const SPRITEXNMASK:u16 = 0x0080;
    const MAXSPRITES:u8 = 64;
    const NOSPRITE:u8 = Constants::MAXSPRITES;
    const MAXSPRITESPERSCANLINE:u8 = 8;

    const PATTERNADDRESSLIMIT:u16 = 0x4000;
}

// Create a dummy VDP, to try out hooking into ports.
pub struct VDP {
    ram: Vec<u8>,
    current_address: u32,
}

impl VDP {
    const SMS_WIDTH:u16  = 256;
    const SMS_HEIGHT:u16 = 192; // MAX HEIGHT

    const FRAME_WIDTH:u16  = Constants::SMS_WIDTH;
    const FRAME_HEIGHT:u16 = Constants::SMS_HEIGHT;
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

    #[test]
    fn test_check_constants() {
        assert_eq!(vdp::Constants::NUMTILES, 896);
        assert_eq!(vdp::Constants::BLANKTIME, 17926);
        assert_eq!(vdp::Constants::VFRAMETIME, 47803);
    }

}
