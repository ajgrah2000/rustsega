use super::super::ports;
use super::super::clocks;
use super::super::interruptor;
use super::display;

#[derive(Clone)]
pub struct TileAttribute {
    priority_cleared:bool,
    priority:u8,
    palette_select:bool, 
    vertical_flip:bool, 
    horizontal_flip:bool, 
    tile_number:u16,
    references:u16,
    changed:bool,
}

impl Default for TileAttribute {
    fn default() -> Self {
        Self {
            priority_cleared:false,
            priority:0,
            palette_select:false, 
            vertical_flip:false, 
            horizontal_flip:false, 
            tile_number:0,
            references:0,
            changed:false,
        }
    }
}

#[derive(Clone)]
pub struct HorizontalScroll {
    column_offset:u8,
    fine_scroll:u8,
    x_offset:u16,
}

impl Default for HorizontalScroll {
    fn default() -> Self {
        Self {
            column_offset:0,
            fine_scroll:0,
            x_offset:0,
        }
    }
}

#[derive(Clone)]
pub struct PatternInfo {
    colour_check:bool,
    colours:u16, // TODO: Check, looks as though this should be a u8.
    changed:bool,
    screen_version_cached:bool,
    references:u16,
}

impl Default for PatternInfo {
    fn default() -> Self {
        Self {
            colour_check:false,
            colours:0,
            changed:false,
            screen_version_cached:false,
            references:0,
        }
    }
}

pub struct Mode1Settings {
    y_scroll:u8,
    x_scroll:u16,
    h_sync_interrupt_enabled:bool,
    start_x:u8, 
    display_mode_1:u8,
}

impl Mode1Settings {
    pub fn new() -> Self {
        Self {
            y_scroll:0,
            x_scroll:0,
            h_sync_interrupt_enabled:false,
            start_x:0, 
            display_mode_1:0,
        }
    }

    pub fn update_mode_1_settings(&mut self, mode_1_input:u8) -> () {
        Mode1Settings::mode_1_settings(mode_1_input, &mut self.y_scroll, &mut self.x_scroll, &mut self.h_sync_interrupt_enabled, &mut self.start_x, &mut self.display_mode_1);
    }

    fn mode_1_settings(mode_1_input:u8, y_scroll:&mut u8, x_scroll:&mut u16, h_sync_interrupt_enabled:&mut bool, start_x:&mut u8, display_mode_1:&mut u8) -> () {
        // Set first scrolling line
        if 0 != (mode_1_input & Constants::VDP0DISHSCROLL) {
            *y_scroll = 16;
        } else {
            *y_scroll = 0;
        }
    
        // Set last scrolling position
        if 0 != (mode_1_input & Constants::VDP0DISHSCROLL) {
            *x_scroll = 192;
        } else {
            *x_scroll = Constants::SMS_WIDTH;
        }
    
        if 0 != (mode_1_input & Constants::VDP0LINEINTENABLE) {
            *h_sync_interrupt_enabled = true;
        } else {
            *h_sync_interrupt_enabled = false;
        }
    
        if 0 != (mode_1_input & Constants::VDP0COL0OVERSCAN) {
            *start_x = Constants::PATTERNWIDTH;
        } else {
            *start_x = 0;
        }
    
            // TODO: Add additional sprite control
//            if (mode_1_input & Constants::VDP0SHIFTSPRITES) {
//                errors.warning("Sprite shift not implemented")
//    
//            if (mode_1_input & Constants::VDP0NOSYNC) {
//                errors.warning("No sync, not implemented")
    
        *display_mode_1 = 0;
        if 0 != (mode_1_input & Constants::VDP0M4) {
            *display_mode_1 |= 8;
        }
        if 0 != (mode_1_input & Constants::VDP0M2) {
            *display_mode_1 |= 2;
        }
    }
}


pub struct Mode2Settings {
    v_sync_interrupt_enabled:bool, 
    enable_display:bool, 
    sprite_height:u8, 
    display_mode_2:u8,
}

impl Mode2Settings {
    fn new() -> Self {
        Self {
            v_sync_interrupt_enabled:false, 
            enable_display:false, 
            sprite_height:0, 
            display_mode_2:0,
        }
    }

    pub fn update_mode_2_settings(&mut self, mode_2_input:u8) -> () {
        Mode2Settings::mode_2_settings(mode_2_input, &mut self.v_sync_interrupt_enabled, &mut self.enable_display, &mut self.sprite_height, &mut self.display_mode_2);
    }

    fn mode_2_settings(mode_2_input:u8, v_sync_interrupt_enabled:&mut bool, enable_display:&mut bool, sprite_height:&mut u8, display_mode_2:&mut u8) -> () {
        if 0 != (mode_2_input & Constants::VDP1VSYNC) {
            *v_sync_interrupt_enabled = true;
        } else {
            *v_sync_interrupt_enabled = false;
        }

        if 0 != (mode_2_input & Constants::VDP1ENABLEDISPLAY) {
            *enable_display = true;
        } else {
            *enable_display = false;
        }
    
        if 0 != (mode_2_input & Constants::VDP1BIGSPRITES) {
            *sprite_height = 16;
        } else {
            *sprite_height = 8;
        }
    
                // TODO: Add double sprites
//            if (mode_2_input & Constants::VDP1DOUBLESPRITES) {
//                errors.warning("Double sprites not implemented")
    
        *display_mode_2 = 0;
        if 0 != (mode_2_input & Constants::VDP1M3) {
            *display_mode_2 |= 4;
        }
        if 0 != (mode_2_input & Constants::VDP1M1) {
            *display_mode_2 |= 1;
        }
    }
}

pub struct Constants {
}

impl Constants {
    const RAMSIZE:u16  = 0x4000;
    const CRAMSIZE:u8  = 0x20;
    // 3Mhz CPU, 50Hz refresh ~= 60000 ticks
    const VSYNCCYCLETIME:u16 = 65232;
    const BLANKTIME:u16      = ((Constants::VSYNCCYCLETIME as u32 * 72)/262) as u16;
    const VFRAMETIME:u16     = ((Constants::VSYNCCYCLETIME as u32 * 192)/262) as u16;
    const HSYNCCYCLETIME:u16 = 216;

    const REGISTERMASK:u8  = 0x0F;
    const REGISTERUPDATEMASK:u8  = 0xF0;
    const REGISTERUPDATEVALUE:u8 = 0x80;
    const NUMVDPREGISTERS:u8 = 16;

    const MAX_MODE_CONTROL_ENTRIES:u16 = 256;

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
    c_ram: Vec<u8>,

    vdp_register: [u8;Constants::NUMVDPREGISTERS as usize],

    horizontal_scroll_info: Vec<HorizontalScroll>,
    vertical_scroll_info: Vec<u8>,
    pattern_info: Vec<PatternInfo>,
    screen_palette: Vec<display::Colour>,

    patterns4: Vec<u8>,
    tile_attributes: Vec<TileAttribute>,

    mode_1_control: Mode1Settings,
    mode_2_control: Mode2Settings,

    // address attributes.
    current_address: u16,
    sprite_attributes_address: u16,
    tile_attributes_address: u16,
    write_bf_low_address: u8, // Holds the 'low' byte of the address when a write to bf occurs.

    border_colour:u8,

    code_register: u8,

    read_be_latch: u8,
    address_latch: bool,

    last_v_sync_clock: clocks::Clock,
    current_y_pos: u16,
    display_mode: u8, 
    y_end: u16,
    
    interrupt_handler: VDPInterrupts,

    sprite_tile_shift:u16,
    horizontal_scroll:u8,
    vertical_scroll:u8,

    // Used internally for debugging/diagnostics
    debug_name_table_offset:u16,
    debug_sprite_information_table_offset:u16,
}

pub struct VDPInterrupts {
    v_sync: u16,
    last_v_sync_clock: clocks::Clock,
    line_int_time:u32,
    line_interrupt:u16,
    line_interrupt_latch:u16,

    h_int_pending:bool,
    v_int_pending:bool,
    v_sync_interrupt_enabled:bool,
    h_sync_interrupt_enabled:bool,
}

impl VDP {
    const SMS_WIDTH:u16  = 256;
    const SMS_HEIGHT:u16 = 192; // MAX HEIGHT

    const FRAME_WIDTH:u16  = Constants::SMS_WIDTH;
    const FRAME_HEIGHT:u16 = Constants::SMS_HEIGHT;
    const PIXEL_WIDTH:u16  = 2;
    const PIXEL_HEIGHT:u16 = 2;

    const START_DRAW_Y:u16 = 0;
    const END_DRAW_Y:u16   = VDP::FRAME_HEIGHT ;


    pub fn new() -> Self {
        Self {
            ram: vec![0; Constants::RAMSIZE as usize],         
            c_ram: vec![0; Constants::CRAMSIZE as usize],
            vdp_register: [0;Constants::NUMVDPREGISTERS as usize],

            // One entry per scan line for horizontal and vertical scroll info.
            horizontal_scroll_info: vec![HorizontalScroll::default(); Constants::SMS_HEIGHT as usize],
            vertical_scroll_info: vec![0;Constants::SMS_HEIGHT as usize],
            pattern_info: vec![PatternInfo::default(); Constants::MAXPATTERNS as usize],
            screen_palette: vec![display::Colour::new(0,0,0);Constants::CRAMSIZE as usize],
            patterns4: vec![0;(Constants::MAXPATTERNS*(Constants::PATTERNSIZE as u16)) as usize],
            tile_attributes: vec![TileAttribute::default();Constants::NUMTILEATTRIBUTES as usize],

            mode_1_control: Mode1Settings::new(),
            mode_2_control: Mode2Settings::new(),
            current_address: 0,
            sprite_attributes_address: 0,
            tile_attributes_address: 0,

            code_register: 0,

            read_be_latch: 0,
            write_bf_low_address: 0,
            border_colour:0,
            address_latch: false,
            last_v_sync_clock: clocks::Clock::new(),
            current_y_pos: 0,
            display_mode: 0, 
            y_end: 0,
            interrupt_handler: VDPInterrupts::new(),

            sprite_tile_shift:0,
            horizontal_scroll:0,
            vertical_scroll:0,

            debug_name_table_offset:0,
            debug_sprite_information_table_offset:0,
        }
    }

    pub fn set_address(&mut self, value: u16) -> () {

    }
    pub fn get_address(&self) -> u16 {
        self.current_address
    }

    pub fn read_port_7e(&mut self, clock: &clocks::Clock) -> u8 {
        self.address_latch = false;  // Address is unlatched during port read
    
        let v_counter:u8 = ((clock.cycles-self.last_v_sync_clock.cycles)/Constants::HSYNCCYCLETIME as u32) as u8;
        self.current_y_pos = (((clock.cycles-self.last_v_sync_clock.cycles)/Constants::HSYNCCYCLETIME as u32)+1) as u16;
    
        // I can't think of an ellegant solution, so this is as good as it gets
        // for now (fudge factor and all)
// TODO: Add joystick (light gun)        self.inputs.joystick.setYpos(vCounter+10)
        v_counter
    }

    pub fn read_port_7f(&mut self, clock: &clocks::Clock) -> u8 {
        self.address_latch = false;  // Address is unlatched during port read
    
        // TODO: Add/fix joystick (light gun)
        // I can't think of an ellegant solution, so this is as good as it gets
        // for now (fudge factor and all)
        // hCounter = ((self.inputs.joystick.getXpos() + 0x28)/2 & 0x7F)
        0
    }

    pub fn read_port_be(&mut self, clock: &clocks::Clock) -> u8 {
        self.address_latch = false; // Address is unlatched during port read
    
        let data = self.read_be_latch;
    
        self.current_address = (self.current_address + 1) & 0x3FFF; // Should be ok without this
        self.read_be_latch = self.ram[self.current_address as usize];
    
        data
    }

    pub fn set_palette(&mut self, address: u16, data:u8) -> () {
        // uint16 colour
        // uint8 r, g, b
    
        let addr = address as u8 % Constants::CRAMSIZE;
    
        if self.c_ram[addr as usize] != data {

            self.c_ram[addr as usize] = data;
    
            // Generate 8-bit RGB components, just to be generic
            let r = ((data as u16 &0x3)*0xFF)/0x3;
            let g = (((data as u16 >>2)&0x3)*0xFF)/0x3;
            let b = (((data as u16 >>4)&0x3)*0xFF)/0x3;
    
            let colour = display::Colour::new(r as u8, g as u8, b as u8);
    
            self.screen_palette[addr as usize] = colour;
    
            // Rough optimisation for palette `rotate' graphics 
            for i in 0..Constants::MAXPATTERNS as usize {
                if self.pattern_info[i].colour_check == false {
                    self.check_pattern_colors(i as u16);
                }
    
                if 0 != self.pattern_info[i].colours & (1<<(addr & 0xF)) {
                    self.pattern_info[i].changed = true;
                    self.pattern_info[i].screen_version_cached = false;
                }
            }
        }
    }

    pub fn check_pattern_colors(&mut self, pattern: u16) -> () {
        self.pattern_info[pattern as usize].colours = 0 ;
        for i in 0..Constants::PATTERNSIZE as u16 {
            self.pattern_info[pattern as usize].colours |= 1 << self.patterns4[(pattern << 6 | i) as usize] ;
        }
    
        self.pattern_info[pattern as usize].colour_check = true;
    }

    pub fn update_tile_attributes(&mut self, address: u16, old_data:u8, data:u8) -> () {
        // Only update if altered
        if old_data != data {
            let tile = (address & Constants::TILEATTRIBUTESTILEMASK) >> Constants::TILESHIFT;
    
            self.pattern_info[self.tile_attributes[tile as usize].tile_number as usize].references -= 1;
    
            // Alteration of the high byte 
            if 0 != address & Constants::TILEATTRIBUTESHMASK {
                if 0 != self.tile_attributes[tile as usize].priority {
                    if 0 == (data >> Constants::TILEPRIORITYSHIFT) {
                        self.tile_attributes[tile as usize].priority_cleared = true;
                    }
                }
    
                self.tile_attributes[tile as usize].priority        =  data >> Constants::TILEPRIORITYSHIFT;
                self.tile_attributes[tile as usize].palette_select  = 0 != (data >> Constants::TILEPALETTESHIFT) & 0x1;
                self.tile_attributes[tile as usize].vertical_flip   = 0 != (data >> Constants::TILEVFLIPSHIFT) & 0x1;
                self.tile_attributes[tile as usize].horizontal_flip = 0 != (data >> Constants::TILEHFLIPSHIFT) & 0x1;
                self.tile_attributes[tile as usize].tile_number     = (self.tile_attributes[tile as usize].tile_number & 0xFF) | (((data as u16) & 0x1) << 8);
            } else {
                self.tile_attributes[tile as usize].tile_number     = (self.tile_attributes[tile as usize].tile_number & 0x100) | (data as u16);
            }
    
            // Flag that the tile referenced is displayed
            // This may `exceed value' (ie 511), but should have no adverse effect
            self.pattern_info[self.tile_attributes[tile as usize].tile_number as usize].references += 1;
    
            self.tile_attributes[tile as usize].changed = true;
        }
    }

    pub fn update_sprite_attributes(&mut self, address: u16, old_data:u8, data:u8) -> () {
            panic!("update_sprite_attributes not implemented");
    }

    pub fn update_horizontal_scroll_info(&mut self) -> () {
        let column_offset = (0x20 - (self.horizontal_scroll >> 3)) & 0x1F;
        let fine_scroll = self.horizontal_scroll & 0x7;
        let x_offset = ((column_offset as u16) * (Constants::PATTERNWIDTH as u16) - (fine_scroll as u16)) % Constants::SMS_WIDTH;
    
        for y in self.current_y_pos as usize ..self.y_end as usize  {
            self.horizontal_scroll_info[y].column_offset = column_offset;
            self.horizontal_scroll_info[y].fine_scroll = fine_scroll;
            self.horizontal_scroll_info[y].x_offset = x_offset;
        }
    }

    pub fn update_vertical_scroll_info(&mut self) -> () {
        for y in self.current_y_pos as usize ..self.y_end as usize  {
            self.vertical_scroll_info[y] = self.vertical_scroll;
        }
    }

    pub fn update_pattern(&mut self, address: u16, old_data:u8, data:u8) -> () {
        let mut change = old_data ^ data; // Flip only the bits that have changed
        if change != 0 {
            let index = (address & 0x3FFC) << 1; // Base index (pattern + row)
    
            let mask = 1 << (address & 0x3);  // Bit position to flip
    
            // Only update if the data has changed
            // From right to left
            let mut x = 7;
            while 0 != change {
                // Flip the bit position if required
                if 0 != change & 0x1 {
                    self.patterns4[(index + x) as usize] ^= mask;
                }
    
                if x > 0 {
                    x -= 1;
                }
                change = change >> 1;
            }
    
            self.pattern_info[(index >> 6) as usize].changed = true;
            self.pattern_info[(index >> 6) as usize].colour_check = false;

            self.pattern_info[(index >> 6) as usize].screen_version_cached = false;
        }
    }

    pub fn write_port_be(&mut self, clock: &clocks::Clock, data: u8) -> () {
        self.address_latch = false;  // Address is unlatched during port read
    
        if self.code_register == 0x3 { // Write to video ram
            self.set_palette(self.current_address, data);
        } else {
            if ((self.current_address & Constants::TILEATTRIBUTESADDRESSMASK) == self.tile_attributes_address) &&
               ((self.current_address & Constants::TILEATTRIBUTEMASK) < Constants::NUMTILEATTRIBUTES) {
                self.update_tile_attributes(self.current_address,self.ram[self.current_address as usize], data);
            } else if ((self.current_address & Constants::SPRITEATTRIBUTESADDRESSMASK) == self.sprite_attributes_address) &&
                      ((self.current_address & Constants::SPRITEATTRIBUTESMASK) < Constants::NUMSPRITEATTRIBUTES) {
                self.update_sprite_attributes(self.current_address,self.ram[self.current_address as usize], data);
            }
            if self.current_address < Constants::PATTERNADDRESSLIMIT {
                self.update_pattern(self.current_address, self.ram[self.current_address as usize], data);
            }
    
            self.ram[self.current_address as usize] = data; // Update after function call
            self.read_be_latch = data;
        }
    
        self.current_address = (self.current_address + 1) & 0x3FFF; // Should be ok without this
    }

    pub fn write_register(&mut self, register_number: u8, data: u8) -> () {

        self.vdp_register[register_number as usize] = data; // Update register data
    
        // Only need to react immediately to some register changes
        match register_number {
            0 => {
                self.update_mode_1_control();
            }
            1 => {
                self.update_mode_2_control();
            }
            2 => {
                self.tile_attributes_address = ((data as u16) & 0xE) << 10;
                self.debug_name_table_offset = self.tile_attributes_address;
            }
            5 => {
                self.sprite_attributes_address = ((data as u16) & 0x7E) << 7;
                self.debug_sprite_information_table_offset = self.sprite_attributes_address;
            }
    
            6 => {
                // self._tileDefinitions = &self._vdpRAM[(data & 0x4) << 11]
                //  Probably should do more when this changes, as all the 
                //  sprite tile numbers should change... maybe later
                self.sprite_tile_shift = ((data as u16) & 0x4) << 6;
            }
    
            7 => {
                println!("Using border colour: {:x}", data);
                self.border_colour = data & 0xf;
            }
    
            8 => {
                self.horizontal_scroll = data;
                self.update_horizontal_scroll_info();
            }
    
            9 => {
                self.vertical_scroll = data;
                self.update_vertical_scroll_info();
            }
    
            10 => {
                self.interrupt_handler.line_interrupt = data as u16;
            }
            _ => { println!("write_register: unsupported write register: {} ", register_number);}
        }
    }

    pub fn write_port_bf(&mut self, clock: &clocks::Clock, data: u8) -> () {
        if false == self.address_latch {
            self.write_bf_low_address = data;
            self.address_latch = true;
        } else {
            if (data & Constants::REGISTERUPDATEMASK) == Constants::REGISTERUPDATEVALUE {
                self.write_register(data & Constants::REGISTERMASK, self.write_bf_low_address)
            }

            self.current_address = ((self.write_bf_low_address as u16) + ((data as u16) << 8)) & 0x3FFF; // Should limit current_address to ram size
            self.code_register = data >> 6; // TODO: Was self._codeRegister = (self._lowaddress + (data << 8)) >> 14
            self.address_latch = false;

            self.read_be_latch = self.ram[self.current_address as usize];
        }
    }

    pub fn read_port_bf(&mut self, clock: &clocks::Clock) -> u8 {
        self.current_address = self.current_address + 1;
        self.current_address as u8
    }




    pub fn update_mode_1_control(&mut self) -> () {
        self.mode_1_control.update_mode_1_settings(self.vdp_register[Constants::MODE_CONTROL_NO_1 as usize]);
        self.interrupt_handler.h_sync_interrupt_enabled = self.mode_1_control.h_sync_interrupt_enabled;
        self.update_display_mode(self.mode_1_control.display_mode_1, self.mode_2_control.display_mode_2);
    }

    pub fn update_mode_2_control(&mut self) -> () {
        self.mode_2_control.update_mode_2_settings(self.vdp_register[Constants::MODE_CONTROL_NO_2 as usize]);
        self.interrupt_handler.v_sync_interrupt_enabled = self.mode_2_control.v_sync_interrupt_enabled;
        self.update_display_mode(self.mode_1_control.display_mode_1, self.mode_2_control.display_mode_2);

    }

    fn update_display_mode(&mut self, display_mode_1: u8, display_mode_2: u8) -> () {
        self.display_mode = display_mode_1 | display_mode_2;

        // Need to see what the modes do/mean.
        if (self.display_mode == 0x8) || (self.display_mode == 0xA) {
            self.y_end = 192;
        } else {
            self.y_end = 0;
            panic!("Mode not supported");
        }
    }

    fn print_debug_info(&mut self) -> () {
        println!("{} {}", self.debug_name_table_offset, self.debug_sprite_information_table_offset); 
    }
}

impl ports::Device for VDP {
    fn port_read(&mut self, clock: &clocks::Clock, port_address: u8) -> u8 {
        match port_address {
            0xbe => {self.read_port_be(clock)}
            0xbf => {self.read_port_bf(clock)}
            // Add the vdp to port `7F' plus all the mirror ports, vdp h_counter
            n if (n & 0xC1 == 0x41) => {self.read_port_7e(clock)}
            // Add the vdp to port `7E' plus all the mirror ports, vdp v_counter
            n if (n & 0xC1 == 0x40) => {self.read_port_7e(clock)}
            _ => {0 /* Unhandled, just return 0 for now */}
        }
    }
    fn port_write(&mut self, clock: &clocks::Clock, port_address: u8, value:u8) -> () {
        match port_address & 0x1 {
            0x0 => {self.write_port_be(clock, value);}
            0x1 => {self.write_port_bf(clock, value);}
            _ => {}
        }
    }

    fn poll_interrupts(&mut self, clock: &clocks::Clock) -> bool {
        self.interrupt_handler.poll_interrupts(clock)
    }
}

impl VDPInterrupts {
    pub fn new() -> Self {
        Self {
            v_sync: 0,
            last_v_sync_clock: clocks::Clock::new(),
            line_int_time: 0,
            line_interrupt: 0,
            line_interrupt_latch: 0,
            h_int_pending: false,
            v_int_pending: false,
            v_sync_interrupt_enabled: true,
            h_sync_interrupt_enabled: true,
        }
    }
}


impl VDPInterrupts {
    fn poll_interrupts(&mut self, clock: &clocks::Clock) -> bool {
        self.v_sync = (clock.cycles - self.last_v_sync_clock.cycles) as u16;
    
        if (self.line_int_time < Constants::VFRAMETIME as u32) &&
            (self.v_sync as u32 >= self.line_int_time) {
            self.line_interrupt_latch = self.line_interrupt + 1;
            self.line_int_time += (self.line_interrupt_latch * Constants::HSYNCCYCLETIME) as u32;
    
            self.h_int_pending = true;
        }
    
        if self.v_sync >= Constants::VFRAMETIME {
            self.v_int_pending = true;
        }
    
        if self.v_sync >= Constants::VSYNCCYCLETIME {
            self.last_v_sync_clock.cycles = clock.cycles;
            self.v_sync = 0;
    
            self.line_interrupt_latch = self.line_interrupt;
            self.line_int_time = (self.line_interrupt_latch * Constants::HSYNCCYCLETIME) as u32;
        }
    
        if (self.v_sync_interrupt_enabled && self.v_int_pending) ||
            (self.h_sync_interrupt_enabled && self.h_int_pending) {
            return true;
        } else {
            return false;
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

            canvas.set_draw_colour(pixels::Color::RGB(0, 0, 0));
            canvas.clear();
            i = (i + 1) % 255;
            canvas.set_draw_colour(pixels::Color::RGB(i, 64, 255 - i));
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

    // set_colour
    // get_colour
    // setCycle
    // pollInterupts
    // readPort7E
    // readPort7F
    // readPortBE
    // readPortBF
    // writePortBF
    // writePortBE
    // updateSpriteAttributes
    // drawSprites
    // removeSpriteFromScanlines
    // addSpriteToScanlines
    // updatePattern
    // updateScreenPattern
    // updateHorizontalScrollInfo
    // updateVerticalScrollInfo
    // updateTileAttributes
    // writeRegister

    // _populateMode1Control
    // _populateMode2Control
    // setInterupt
    // getNextInterupt
    // openDisplay
    // setPalette
    // checkPatternColors
    // drawBuffer
    // drawBackground
    // clearDisplay
    // updateDisplay
    // single_scan


    // drawPatterns
    // printSpriteInformation
    // printNameTable
