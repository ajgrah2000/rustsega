use sdl2::audio;
use super::soundchannel;

pub type SoundQueueType = audio::AudioQueue<soundchannel::PlaybackType>;
pub struct SDLUtility {}

impl SDLUtility {
    // TODO: Fix up values, make them more dynamic, do better comparisons
    // Not sure how they compare on different PCs
    const TARGET_QUEUE_LENGTH:u32 = 1000;// This drives the 'delay' in audio, but too small for the speed and they aren't filled fast enough
    const AUDIO_SAMPLE_SIZE:u16 = 100; // 'Desired' sample size (smaller make sound 'more accurate')
    const FRACTION_FILL:f32 = 0.8; // TODO: FUDGE FACTOR.  Don't completely fill, samples a removed 1 at a time, don't fill them immediately.

    const MONO_STERO_FLAG:u8 = 1; // TODO: Make this configurable 1 - mono, 2 - stereo

    pub fn get_audio_queue (
        sdl_context: &mut sdl2::Sdl,
    ) -> Result<SoundQueueType, String> {
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = audio::AudioSpecDesired {
            freq: Some(Sound::SAMPLERATE as i32),
            channels: Some(SDLUtility::MONO_STERO_FLAG), // mono
            samples: Some(SDLUtility::AUDIO_SAMPLE_SIZE),
        };

        audio_subsystem.open_queue::<soundchannel::PlaybackType,_>(None, &desired_spec)
    }

    pub fn top_up_audio_queue<F>(audio_queue: &mut SoundQueueType, mut get_additional_buffer:F)
        where F: FnMut(u32) ->Vec<soundchannel::PlaybackType> {
            assert!(audio_queue.size() <= SDLUtility::TARGET_QUEUE_LENGTH as u32);
            let fill_size = ((SDLUtility::TARGET_QUEUE_LENGTH - audio_queue.size()) as f32 * SDLUtility::FRACTION_FILL) as u32;
            // If 'stereo' the buffer is twice as large, so just as for half as much.
            let sound_buffer = get_additional_buffer(fill_size/(SDLUtility::MONO_STERO_FLAG as u32));
            audio_queue.queue_audio(&sound_buffer).unwrap();
    }
}

pub struct Sound {
    volume: Vec<u8>,
    channels: Vec<soundchannel::SoundChannel>,
    freq: Vec<u32>,
    chan_freq: u8,

    current_channel: u8,
    current_type: u8,

    shift_rate: u16, // 0b00 - N/512, 0b01 - N/1024, 0b10 - N/2048, 0b11 - Tone Generator #3 Output
    noise_period_select: bool, // true - noise
}

impl Sound {
    const FREQMULTIPLIER: u32 = 125000;
    //    const SAMPLERATE:u32 = 32050;
    const SAMPLERATE: u32 = 44100;
    const CHANNELS: u8 = 4;
    const BITS: u8 = 8;
    const MAX_VOLUME_MASK: u8 = 0xF;
    pub fn new() -> Self {
        Self {
            volume: vec![Sound::MAX_VOLUME_MASK; Sound::CHANNELS as usize],
            channels: vec![
                soundchannel::SoundChannel::new(),
                soundchannel::SoundChannel::new(),
                soundchannel::SoundChannel::new(),
                soundchannel::SoundChannel::new(),
            ],
            freq: vec![0x0; Sound::CHANNELS as usize],
            chan_freq: 0,
            current_channel: 0,
            current_type: 0,
            shift_rate: 0,
            noise_period_select: false,
        }
    }
    fn get_hertz(frequency: u32) -> u32 {
        Sound::FREQMULTIPLIER / (frequency + 1)
    }


    pub fn get_next_audio_chunk(&mut self, length: u32) -> Vec<soundchannel::PlaybackType> {
        let mut stream = Vec::with_capacity((2*length) as usize);
        if length > 0 {
            for i in 0..(length * (SDLUtility::MONO_STERO_FLAG as u32)) {
                stream.push(0x0); // Neutral volume
            }

            for c in 0..Sound::CHANNELS {
                self.channels[c as usize].set_volume((Sound::MAX_VOLUME_MASK - self.volume[c as usize]) << 4);
                self.channels[c as usize].set_frequency(Sound::get_hertz(self.freq[c as usize]), Sound::SAMPLERATE);
                let mut channel_wave = self.channels[c as usize].get_wave(length);

                if 3 == c {
                    for i in 0..length {
                        // Channel 4:
                        channel_wave[i as usize] = self.channels[c as usize].get_shiff_register_output(Sound::get_hertz(self.freq[c as usize]), self.noise_period_select, Sound::SAMPLERATE);
                    }
                }

                if c % SDLUtility::MONO_STERO_FLAG == 0 {
                    for i in 0..length {
                        stream[(i * (SDLUtility::MONO_STERO_FLAG as u32)) as usize] += channel_wave[i as usize];
                    }
                } else {
                    // This will only be called if 'MONO_STEREO_FLAG' is set to '2'
                    for i in 0..length {
                        stream[(i * (SDLUtility::MONO_STERO_FLAG as u32) + 1) as usize] += channel_wave[i as usize];
                    }
                }
            }

        }

        stream
    }

    pub fn write_port(&mut self, data:u8) {
        // Dispatch the data to perform the specified audio function (frequency,
        // channel frequency, volume).

        if (data & 0x80) == 0x80 {
            self.current_channel = (data >> 5) & 0x3;
            self.current_type    = (data >> 4) & 0x1;
        }

        if (data & 0x90) == 0x90 {
            self.volume[((data >> 5) & 0x3) as usize] = data & Sound::MAX_VOLUME_MASK;
        }

        if (data & 0x90) == 0x80 {
            self.chan_freq = data;
        }

        // For the 'noise' channel, the same setting appear from LATCH/DATA or DATA. 
        if 3 == self.current_channel {
            if (data & 0x3) < 3 {
                self.freq[self.current_channel as usize] = match data & 0x3
                {
                    0 => {0x10},
                    1 => {0x20},
                    2 => {0x40},
                    3 => {self.freq[2 as usize]}, // TODO: Not sure how this works
                    _ => {panic!("Match for noise frequency not possible");}
                };

                // TODO: Not sure how the 'periodic' should sound.
                // Superficially, it sounds better if noise is forced to 'true'
//                self.noise_period_select = 0x1 == (data >> 2) & 0x1; // If (---trr) -> t = 1 -> white noise
                self.noise_period_select = true; // Disable 'periodic'

                // Reset the noise shift register:
                self.channels[self.current_channel as usize].ch4_shift_register = 0; // Clear the register (will be set on first get).
            } else {
                // TODO: Figure out what 'Tone 3' means
                self.freq[self.current_channel as usize] = 0;
            }
        }

        if (data & 0x80) == 0x00 {
            if 3 != self.current_channel || 3 == (data & 0x3) {
                self.freq[((self.chan_freq >> 5) & 0x3) as usize] = (((data & 0x3F) as u32) << 4) | (self.chan_freq & 0xF) as u32;
            }
        }
    }
}

