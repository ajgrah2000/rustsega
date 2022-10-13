type SoundPatternType = u16;

pub struct SoundChannel {
    volume: u8,
    playlength: SoundPatternType,
    nextlength: SoundPatternType,
    playpos: SoundPatternType,
    playbuf: Vec<PlaybackType>,
    next: Vec<PlaybackType>,
    updated: bool,

    r_min: u32,
    play_count: u32,
}

pub type PlaybackType = u8;  // Only 8-bit playback is currently supported.
impl SoundChannel {
    const MAX_SOUND_PATTERN: SoundPatternType = 512;

    const MAX_PLAY_FADE_COUNT: u32 = 100;
    const NUM_CHANNELS: u8 = 4;
    const NEUTRAL_SOUND_LEVEL: u32 = 0x7F/SoundChannel::NUM_CHANNELS as u32;

    pub fn new() -> Self {
        Self {
            volume: 0,
            playlength: 0, //SoundChannel::MAX_SOUND_PATTERN,
            nextlength: 0, //SoundChannel::MAX_SOUND_PATTERN,
            playpos: 0,
            playbuf: vec![0; SoundChannel::MAX_SOUND_PATTERN as usize],
            next: vec![0; SoundChannel::MAX_SOUND_PATTERN as usize],
            updated: false,

            r_min: 0,
            play_count: SoundChannel::MAX_PLAY_FADE_COUNT,
        }
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume / SoundChannel::NUM_CHANNELS;
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
                r %= sample_rate;
                vol = self.volume - vol;
                if vol == self.volume  && r < self.r_min {
                    self.r_min = r;
                    r_min_pos = nextlength;
                }
            }

            self.next[nextlength as usize] = vol as PlaybackType;
            r += d;
        }

        self.nextlength = r_min_pos;

        self.updated = true;
    }

    pub fn get_wave(&mut self, length: u32) -> Vec<PlaybackType> {
        // Generate the 'wave' output buffer.
        // First copy what's left of the current 'play buffer', update to the
        // new buffer, if it's changed and copy that until the wave buffer has
        // been fully populated.

        let mut wave = Vec::with_capacity(length as usize);

        while (self.playpos < self.playlength) && (wave.len() < length as usize) {
            wave.push(self.playbuf[self.playpos as usize]);

            self.playpos += 1;
        }

        if self.playpos >= self.playlength {
            // Swap buffers if updated
            if self.updated {
                self.updated = false;

                std::mem::swap(&mut self.next, &mut self.playbuf);
                std::mem::swap(&mut self.playlength, &mut self.nextlength);
            }
            if self.playlength == 0 {
                while wave.len() < length as usize {
                    wave.push(0);
                }
            } else {
                self.playpos = 0;
                while wave.len() < length as usize {
                    self.playpos = 0;
                    while (self.playpos < self.playlength) && (wave.len() < length as usize) {
                        wave.push(self.playbuf[self.playpos as usize]);
                        self.playpos += 1;
                    }
                }
            }
        }

        // TODO: Improve 'fade in' of audio, to avoid 'snap/crackle/pop'
        if self.play_count > 0 { 
            wave.iter_mut().for_each(|x| {if self.play_count > 0 {self.play_count -= 1;} 
                                                *x = (((self.play_count * SoundChannel::NEUTRAL_SOUND_LEVEL) + 
                                                     (*x as u32 * (SoundChannel::MAX_PLAY_FADE_COUNT - self.play_count)))/SoundChannel::MAX_PLAY_FADE_COUNT) as u8});
        }
        wave
    }
}
