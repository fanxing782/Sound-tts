use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
#[cfg(target_family = "unix")]
use crate::mark::linux::linux::LinuxTTs;

#[cfg(target_family = "windows")]
use crate::mark::windows::windows::WindowsTTs;
mod mark;
mod ffi;

#[derive(Debug)]
pub enum Error {
    #[cfg(target_family = "windows")]
    Windows(windows::core::Error),
    #[cfg(target_family = "windows")]
    WinResult(windows_result::Error),
}

lazy_static! {
    static ref SPEAKERS:Arc<RwLock<Vec<Speaker>>> = Arc::new(RwLock::new(Vec::new()));
    static ref DEVICE_DEFAULT:Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
}

pub struct SoundTTs;

impl SoundTTs {
    /// 初始化设备
    /// ```rust
    ///  use sound_tts::SoundTTs;
    ///  SoundTTs::init();
    /// ```
    pub fn init() {
        #[cfg(target_family = "windows")]
        let devices = WindowsTTs::devices();

        #[cfg(target_family = "unix")]
        let devices = LinuxTTs::devices();


        if let Ok(mut speakers) = SPEAKERS.clone().try_write() {
            for x in devices {
                speakers.push(Speaker::new(x))
            }
        }
    }

    /// 获取本机声卡设备列表
    /// ```rust
    ///  use sound_tts::SoundTTs;
    ///  SoundTTs::init();
    ///  let devices:Vec<String> = SoundTTs::get_devices();
    ///  for x in devices {
    ///     println!("{}", x);
    ///  }
    /// ```
    pub fn get_devices() -> Vec<String> {
        if let Ok(speakers) = SPEAKERS.clone().try_read() {
            let names: Vec<String> = speakers.iter()
                .map(|speaker| speaker.name.clone())
                .collect();
            drop(speakers);
            return names;
        }
        vec![]
    }


    /// 输入设备名称，判断是否存在
    ///
    pub fn device_is_exist(device_name: &str) -> bool {
        let devices: Vec<String> = SoundTTs::get_devices();
        devices.contains(&device_name.to_string())
    }

    /// 按顺序播放文本
    /// ```rust
    /// use sound_tts::{SoundTTs, SoundValue};
    /// SoundTTs::init();
    /// SoundTTs::speak(SoundValue::create("文本","设备名称"));
    /// ```
    pub fn speak(value: SoundValue) {
        Self::execute(value, false);
    }

    /// 中断之前顺序播放，从本次开始播放，且丢弃之前队列
    /// ```rust
    /// use sound_tts::{SoundTTs, SoundValue};
    /// SoundTTs::init();
    /// SoundTTs::speak_interrupt(SoundValue::create("文本","设备名称"));
    /// ```
    pub fn speak_interrupt(value: SoundValue) {
        Self::execute(value, true);
    }

    pub fn execute(value: SoundValue, interrupt: bool) {

        if let Ok(guard) = SPEAKERS.clone().try_read() {
            let speaker_option = guard.iter()
                .find(|speaker| speaker.name == value.device_name).clone();
            if let Some(speaker) = speaker_option {
                speaker.speak(value, interrupt);
            }
        }
    }


    /// 当前设备停止播放
    /// ```rust
    /// use sound_tts::SoundTTs;
    /// SoundTTs::init();
    /// SoundTTs::stop("设备名称)");
    /// ```
    pub fn stop(device: &str) {
        if let Ok(guard) = SPEAKERS.clone().try_read() {
            let speaker_option = guard.iter()
                .find(|speaker| speaker.name == device).clone();
            if let Some(speaker) = speaker_option {
                let _ =  speaker.stop();

            }
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

    fn default_device() -> Option<String>
    where
        Self: Sized;

    fn speak(&self, value: SoundValue, interrupt: bool) -> Result<(), Error>;

    #[allow(dead_code)]
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

    fn speak(&self, value: SoundValue, interrupt: bool) {
        let tts = self.tts.clone();
        sleep(Duration::from_millis(10));
        thread::spawn(move || {
            if let Ok(guard) = tts.try_read() {
               let _ =  guard.speak(value, interrupt);
            }
        });
    }

    fn stop(&self) -> Result<(), Error> {
        let tts = self.tts.clone();
        if let Ok(guard) = tts.try_read() {
            guard.stop()?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SoundValue {
    device_name: String,
    str: String,
    play_count: u64,
    play_interval: u64,
}


/// ```
/// use sound_tts::SoundValue;
/// // 创建播放一次的内容
/// SoundValue::create("文本","设备名称");
/// // 自定义播放配置
/// // play_count 播放次数 0 是一直循环播放
/// // play_interval 本次播放完成后，播放下一次之间的间隔时间
/// SoundValue::new("文本",0,1,"设备名称");
/// ```
impl SoundValue {
    pub fn default(str: &str) -> Self {
        #[cfg(target_family = "windows")]
        let default = WindowsTTs::default_device();

        #[cfg(target_family = "unix")]
        let default = LinuxTTs::default_device();

        Self {
            str: String::from(str),
            play_count: 1,
            play_interval: 0,
            device_name:  if let Some(default) = default {
                default
            }else {
                String::new()
            },
        }
    }

    pub fn create(str: &str, device_name: &str) -> Self {
        Self {
            str: String::from(str),
            play_count: 1,
            play_interval: 0,
            device_name: String::from(device_name),
        }
    }

    pub fn new(str: &str, play_count: u64, play_interval: u64, device_name: &str) -> Self {
        Self {
            str: String::from(str),
            play_count,
            play_interval,
            device_name: String::from(device_name),
        }
    }
}


impl Into<String> for SoundValue {
    fn into(self) -> String {
        self.str
    }
}
