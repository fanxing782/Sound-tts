use sound_tts::{SoundTTs, SoundValue};

#[cfg(test)]
mod test {
    use crate::{SoundTTs, SoundValue};

    #[test]
    pub fn test() {
        SoundTTs::init();
        let devices: Vec<String> = SoundTTs::get_devices();
        for x in devices {
            println!("{}", x);
        }

        let device_name = "VG27AQML1A (NVIDIA High Definition Audio)";
        SoundTTs::speak(SoundValue::create("test",device_name));
        SoundTTs::stop(device_name);
        SoundTTs::speak(SoundValue::create("test",device_name));

        loop {
            SoundTTs::speak(SoundValue::create("test",device_name));
        }
    }
}