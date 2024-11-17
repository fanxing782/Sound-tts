use lazy_static::lazy_static;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

#[cfg(target_family = "unix")]
use crate::mark::linux::linux::LinuxTTs;

#[cfg(target_family = "windows")]
use crate::mark::windows::windows::WindowsTTs;
mod mark;


mod test {
    use std::thread::sleep;
    use std::time::Duration;
    use crate::SoundTTs;

    #[test]
    pub fn test(){
        SoundTTs::init();
        let devices:Vec<String> = SoundTTs::get_devices();
        for x in devices {
            println!("{}", x);
            SoundTTs::speak(&x, &x);
        }
        sleep(Duration::from_secs(5));
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

pub struct SoundTTs;

impl SoundTTs {
    /// 初始化设备
    /// ```rust
    ///  use Sound_tts::SoundTTs;
    ///  SoundTTs::init();
    /// ```
    pub fn init() {
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

    /// 获取本机声卡设备列表
    /// ```rust
    ///  use Sound_tts::SoundTTs;
    ///  SoundTTs::init();
    ///  let devices:Vec<String> = SoundTTs::get_devices();
    ///  for x in devices {
    ///     println!("{}", x);
    ///  }
    /// ```
    pub fn get_devices() -> Vec<String> {
        if let Ok(speakers) = SPEAKERS.clone().read() {
            let names: Vec<String> = speakers.iter()
                .map(|speaker| speaker.name.clone())
                .collect();
            drop(speakers);
            return names;
        }
        vec![]
    }

    /// 按顺序播放文本
    /// ```rust
    /// use Sound_tts::SoundTTs;
    /// SoundTTs::init();
    /// SoundTTs::speak("文本", "设备名称)");
    /// ```
    pub fn speak(text: &str, device: &str) {
        Self::execute(text, device, false);
    }

    /// 中断之前顺序播放，从本次开始播放，且丢弃之前队列
    /// ```rust
    /// use Sound_tts::SoundTTs;
    /// SoundTTs::init();
    /// SoundTTs::speak_interrupt("文本", "设备名称)");
    /// ```
    pub fn speak_interrupt(text: &str, device: &str) {
        Self::execute(text, device, true);
    }

    pub fn execute(text: &str, device: &str, interrupt: bool) {
        let guard = SPEAKERS.read().unwrap();
        let speaker_option = guard.iter()
            .find(|speaker| speaker.name == device).clone();
        if let Some(speaker) = speaker_option {
            speaker.speak(text.to_string(), interrupt);
        }
    }


    /// 当前设备停止播放
    /// ```rust
    /// use Sound_tts::SoundTTs;
    /// SoundTTs::init();
    /// SoundTTs::stop("设备名称)");
    /// ```
    pub fn stop(device: &str) {
        let guard = SPEAKERS.read().unwrap();
        let speaker_option = guard.iter()
            .find(|speaker| speaker.name == device).clone();
        if let Some(speaker) = speaker_option {
            speaker.stop().expect("TODO: panic message");
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


    fn speak(&self, context: String, interrupt: bool) -> Result<(), Error>;


    fn is_playing(&self) -> Result<bool, Error>;

    fn stop(&self) -> Result<(), Error>;
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

    fn speak(&self, text: String, interrupt: bool) {
        let tts = self.tts.clone();
        sleep(Duration::from_millis(10));
        thread::spawn(move || {
            let guard = tts.read().expect("The lock for the mark was not obtained when starting to play");
            guard.speak(text.clone(), interrupt).expect("Speaking failed");
        });
    }

    fn stop(&self) -> Result<(), Error> {
        let tts = self.tts.clone();
        let guard = tts.read().expect("Unable to obtain lock for mark when stopping playback");
        guard.stop()?;
        Ok(())
    }
}


#[derive(Debug)]
pub(crate) struct QueueStack<T> {
    data: Arc<Mutex<Vec<T>>>,
}

impl<T> QueueStack<T> {
    fn new() -> QueueStack<T> {
        Self { data: Arc::new(Mutex::new(Vec::new())) }
    }
    fn push(&mut self, item: T) {
        let stack = self.data.clone();
        stack.lock().unwrap().push(item);
    }
    fn pop(&self) -> Option<T> {
        let stack = self.data.clone();
        let mut stack = stack.lock().unwrap();
        if stack.is_empty() {
            None
        } else {
            Some(stack.remove(0))
        }
    }
    fn clear(&mut self) {
        let stack = self.data.clone();
        let mut stack = stack.lock().unwrap();
        if !stack.is_empty() {
            stack.clear();
        }
    }
}
