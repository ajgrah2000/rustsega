type SoundPatternType = u16;

pub struct SoundChannel {
    volume: u8,
    playlength: SoundPatternType,
    nextlength: SoundPatternType,
    playpos: SoundPatternType,
    playbuf: Vec<u8>,
    next: Vec<u8>,
    updated: bool,

    r_min: u32,
}

impl SoundChannel {
    const MAX_SOUND_PATTERN: SoundPatternType = 512;

    pub fn new() -> Self {
        Self {
            volume: 0,
            playlength: 0,
            nextlength: 0,
            playpos: 0,
            playbuf: vec![0; SoundChannel::MAX_SOUND_PATTERN as usize],
            next: vec![0; SoundChannel::MAX_SOUND_PATTERN as usize],
            updated: false,

            r_min: 0,
        }
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume / 4;
    }

    pub fn set_frequency(&mut self, freq: u32, sample_rate: u32) {
        // Generate a particular frequency for the channel.
        // Generates a square waves at the specified frequency, for the length
        //'MAX_SOUND_PATTERN'.

        let mut vol = self.volume;

        let d = freq * 2;
        let mut r = self.r_min;
        self.r_min = sample_rate;
        let mut r_min_pos = SoundChannel::MAX_SOUND_PATTERN;

        for nextlength in 0..SoundChannel::MAX_SOUND_PATTERN {
            if r >= sample_rate {
                r = r % sample_rate;
                vol = self.volume - vol;
                if vol == self.volume {
                    if r < self.r_min {
                        self.r_min = r;
                        r_min_pos = nextlength;
                    }
                }
            }

            self.next[nextlength as usize] = vol;
            r += d;
        }

        self.nextlength = r_min_pos;

        self.updated = true;
    }

    pub fn get_wave(&mut self, length: u32) -> Vec<u8> {
        // Generate the 'wave' output buffer.
        // First copy what's left of the current 'play buffer', update to the
        // new buffer, if it's changed and copy that until the wave buffer has
        // been fully populated.

        let mut wave = Vec::with_capacity(length as usize);

        let mut wave_pos = 0;
        while (self.playpos < self.playlength) && (wave_pos < length) {
//            wave[wave_pos as usize] = self.playbuf[self.playpos as usize];
            wave.push(self.playbuf[self.playpos as usize]);

            self.playpos += 1;
            wave_pos += 1;
        }

        if self.playpos >= self.playlength {
            // Swap buffers if updated
            if self.updated {
                self.updated = false;

                std::mem::swap(&mut self.next, &mut self.playbuf);
                std::mem::swap(&mut self.playlength, &mut self.nextlength);
            }
            if self.playlength == 0 {
                while wave_pos < length {
//                    wave[wave_pos as usize] = 0;
                    wave.push(0);
                }
            } else {
                self.playpos = 0;
                while wave_pos < length {
                    self.playpos = 0;
                    while (self.playpos < self.playlength) && (wave_pos < length) {
//                        wave[wave_pos as usize] = self.playbuf[self.playpos as usize];
                        wave.push(self.playbuf[self.playpos as usize]);
                        wave_pos += 1;
                        self.playpos += 1;
                    }
                }
            }
        }

        wave
    }
}
