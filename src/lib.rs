use lazy_static::lazy_static;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

#[cfg(target_family = "unix")]
use crate::mark::linux::linux::LinuxTTs;

#[cfg(target_family = "windows")]
use crate::mark::windows::windows::WindowsTTs;
mod mark;


mod test {
    use crate::{SoundTTs,SoundValue};

    #[test]
    pub fn test(){
        SoundTTs::init();
        let devices:Vec<String> = SoundTTs::get_devices();
        for x in devices {
            println!("{}", x);
        }
    }

    #[test]
    pub fn play(){
        SoundTTs::init();
        SoundTTs::speak(SoundValue::create("测试","耳机 (Realtek USB Audio)"));
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


        if let Ok(mut speakers) = SPEAKERS.clone().write() {
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
    /// use sound_tts::{SoundTTs, SoundValue};
    /// SoundTTs::init();
    /// SoundTTs::speak(SoundValue::create("文本","设备名称"));
    /// ```
    pub fn speak(value: SoundValue) {
        Self::execute(value,false);
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

    pub fn execute(value: SoundValue,interrupt: bool) {
        let guard = SPEAKERS.read().unwrap();
        let speaker_option = guard.iter()
            .find(|speaker| speaker.name == value.device_name).clone();
        if let Some(speaker) = speaker_option {
            speaker.speak(value, interrupt);
        }
    }


    /// 当前设备停止播放
    /// ```rust
    /// use sound_tts::SoundTTs;
    /// SoundTTs::init();
    /// SoundTTs::stop("设备名称)");
    /// ```
    pub fn stop(device: &str) {
        let guard = SPEAKERS.read().unwrap();
        let speaker_option = guard.iter()
            .find(|speaker| speaker.name == device).clone();
        if let Some(speaker) = speaker_option {
            speaker.stop().expect("speaker stop failed");
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

    fn default_device()-> Option<String>
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
        // sleep(Duration::from_millis(10));
        thread::spawn(move || {
            let guard = tts.read().expect("The lock for the mark was not obtained when starting to play");
            guard.speak(value, interrupt).expect("Speaking failed");
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
pub struct SoundValue{
    device_name: String,
    str:String,
    play_count:u64,
    play_interval:u64
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

    #[cfg(feature = "default_device")]
    pub fn default(str:&str)->Self{
        #[cfg(target_family = "windows")]
        let default = WindowsTTs::default_device();

        #[cfg(target_family = "unix")]
        let default = LinuxTTs::default_device();

        Self{
            str: String::from(str),
            play_count: 1,
            play_interval: 0,
            device_name:String::from(default.expect("No default device")),
        }
    }

    pub fn create(str:&str, device_name:&str) ->Self{
        Self{
            str: String::from(str),
            play_count: 1,
            play_interval: 0,
            device_name:String::from(device_name)
        }
    }

    pub fn new(str:&str,play_count:u64,play_interval:u64,device_name:&str)->Self{
        Self{
            str: String::from(str),
            play_count,
            play_interval,
            device_name:String::from(device_name)
        }
    }
}


impl Into<String> for SoundValue {
    fn into(self) -> String {
        self.str
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
