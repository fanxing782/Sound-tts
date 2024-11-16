use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

#[cfg(target_family = "unix")]
use crate::mark::linux::linux::LinuxTTs;

#[cfg(target_family = "windows")]
use crate::mark::windows::windows::WindowsTTs;


mod mark;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        SoundTTs::init();
        let devices:Vec<String> = SoundTTs::get_devices();
        for x in devices {
            SoundTTs::speak(x.clone(), x);
        }
        thread::sleep(Duration::from_secs(5));
    }
}


#[derive(Debug)]
pub enum Error {
    #[cfg(target_family = "windows")]
    Windows(windows::core::Error),
    #[cfg(target_family = "windows")]
    WinResult(windows_result::Error),
}


lazy_static! {
    static ref SPEAKERS:Arc<RwLock<Vec<Speaker>>> = Arc::new(RwLock::new(Vec::new()));
}



struct SoundTTs;

impl SoundTTs {
    fn init() {
        #[cfg(target_family = "windows")]
        let devices = WindowsTTs::devices();
        #[cfg(target_family = "unix")]
        let devices = LinuxTTs::devices();

        if let Ok(mut speakers) = SPEAKERS.clone().write() {
            for x in devices {
                speakers.push(Speaker::new(x))
            }
        }
    }

    fn get_devices() -> Vec<String> {
        if let Ok(speakers) = SPEAKERS.clone().read() {
            let names: Vec<String> = speakers.iter()
                .map(|speaker| speaker.name.clone())
                .collect();
            drop(speakers);
            return names;
        }
        vec![]
    }

    fn speak(text: String, device: String) {
        let guard = SPEAKERS.read().unwrap();
        let speaker_option = guard.iter()
            .find(|speaker| speaker.name == device).clone();
        if let Some(speaker) = speaker_option {
            speaker.speak(text);
        }
    }
}

trait Target {
    fn new(device: &str) -> Result<Self, Error>
    where
        Self: Sized;

    fn devices() -> Vec<String>
    where
        Self: Sized;


    fn speak(&self, context: String) -> Result<(), Error>;
}


#[derive(Debug)]
struct Speaker {
    name: String,
    #[cfg(target_family = "windows")]
    tts: Arc<RwLock<WindowsTTs>>,
    #[cfg(target_family = "unix")]
    tts: Arc<RwLock<LinuxTTs>>,
}


impl Speaker {
    fn new(name: String) -> Speaker {
        Speaker {
            #[cfg(target_family = "windows")]
            tts: Arc::new(RwLock::new(WindowsTTs::new(name.as_str()).unwrap())),
            #[cfg(target_family = "unix")]
            tts: Arc::new(RwLock::new(LinuxTTs::new(name.as_str()).unwrap())),
            name,
        }
    }

    fn speak(&self, text: String) {
        let tts = self.tts.clone();
        thread::spawn(move || {
            tts.read().expect("Lock Faild").speak(text.clone()).expect("Speaking failed");
            thread::sleep(Duration::from_secs(1));
        });
    }
}



