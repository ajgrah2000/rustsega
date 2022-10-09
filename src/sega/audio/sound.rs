use sdl2::audio;
use super::soundchannel;

pub type SoundQueueType = audio::AudioQueue<soundchannel::PlaybackType>;
pub struct SDLUtility {}

impl SDLUtility {
    // TODO: Fix up values, make them more dynamic, do better comparisons
    // Not sure how they compare on different PCs
    const TARGET_QUEUE_LENGTH:u32 = 2000;// This drives the 'delay' in audio, but too small for the speed and they aren't filled fast enough
    const AUDIO_SAMPLE_SIZE:u16 = 100; // 'Desired' sample size (smaller make sound 'more accurate')
    const FRACTION_FILL:f32 = 0.8; // TODO: FUDGE FACTOR.  Don't completely fill, samples a removed 1 at a time, don't fill them immediately.

    pub fn get_audio_queue (
        sdl_context: &mut sdl2::Sdl,
    ) -> Result<SoundQueueType, String> {
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = audio::AudioSpecDesired {
            freq: Some(Sound::SAMPLERATE as i32),
            channels: Some(1), // mono
            samples: Some(SDLUtility::AUDIO_SAMPLE_SIZE),
        };

        let queue = audio_subsystem.open_queue::<soundchannel::PlaybackType,_>(None, &desired_spec);
        queue
    }

    pub fn top_up_audio_queue<F>(audio_queue: &mut SoundQueueType, mut get_additional_buffer:F)
        where F: FnMut(u32) ->Vec<soundchannel::PlaybackType> {
            assert!(audio_queue.size() <= SDLUtility::TARGET_QUEUE_LENGTH as u32);
            let fill_size = ((SDLUtility::TARGET_QUEUE_LENGTH - audio_queue.size()) as f32 * SDLUtility::FRACTION_FILL) as u32;
            let sound_buffer = get_additional_buffer(fill_size);
            audio_queue.queue_audio(&sound_buffer).unwrap();
    }
}

pub struct Sound {
    volume: Vec<u8>,
    channels: Vec<soundchannel::SoundChannel>,
    freq: Vec<u32>,
    chan_freq: u8,
}

impl Sound {
    const FREQMULTIPLIER: u32 = 125000;
    //    const SAMPLERATE:u32 = 32050;
    const SAMPLERATE: u32 = 22050;
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
        }
    }
    fn get_hertz(frequency: u32) -> u32 {
        Sound::FREQMULTIPLIER / (frequency + 1)
    }


    pub fn get_next_audio_chunk(&mut self, length: u32) -> Vec<soundchannel::PlaybackType> {
        let mut stream = Vec::with_capacity(length as usize);
        for i in 0..length {
            stream.push(0);
        }

        for c in 0..Sound::CHANNELS {
            self.channels[c as usize].set_volume((Sound::MAX_VOLUME_MASK - self.volume[c as usize]) << 4);
            self.channels[c as usize].set_frequency(Sound::get_hertz(self.freq[c as usize]), Sound::SAMPLERATE);
            let channel_wave = self.channels[c as usize].get_wave(length);

            for i in 0..length {
                stream[i as usize] += channel_wave[i as usize];
            }
        }

        stream
    }

    pub fn write_port(&mut self, data:u8) {
        // Dispatch the data to perform the specified audio function (frequency,
        // channel frequency, volume).

        if (data & 0x90) == 0x90 {
            self.volume[((data >> 5) & 0x3) as usize] = data & Sound::MAX_VOLUME_MASK;
        }

        if (data & 0x90) == 0x80 {
            self.chan_freq = data;
        }

        if (data & 0x80) == 0x00 {
            self.freq[((self.chan_freq >> 5) & 0x3) as usize] = (((data & 0x3F) as u32) << 4) | (self.chan_freq & 0xF) as u32;
        }
    }
}

