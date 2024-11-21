#[cfg(target_family = "windows")]
pub mod windows {
    use crate::{Error, QueueStack, Target};
    use lazy_static::lazy_static;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, RwLock};
    use std::thread::sleep;
    use std::time::Duration;
    use windows::core::HSTRING;
    use windows::Devices::Enumeration::{DeviceClass, DeviceInformation};
    use windows::Media::Core::MediaSource;
    use windows::Media::Playback::{MediaPlaybackState, MediaPlayer};
    use windows::Media::SpeechSynthesis::SpeechSynthesizer;
    use windows_result::HRESULT;

    lazy_static! {
        static ref DEVICE_CACHE_INFO:Arc<RwLock<HashMap<String, DeviceInformation>>> = Arc::new(RwLock::new(HashMap::new()));
    }

    #[derive(Debug)]
    pub(crate) struct WindowsTTs {
        player: Arc<MediaPlayer>,
        queue: Arc<Mutex<QueueStack<String>>>,
        state: Arc<Mutex<u8>>,
    }
    impl WindowsTTs {
        fn play(&self) -> Result<(), Error> {
            let queue_guard = self.queue.clone();
            let queue = queue_guard.lock().unwrap();
            if let Some(next_text) = queue.pop() {
                drop(queue);
                if next_text.is_empty() {
                    return Ok(());
                }
                let player = self.player.clone();
                let str = HSTRING::from(&next_text);
                let synthesizer = SpeechSynthesizer::new()?;
                let stream = synthesizer.SynthesizeTextToStreamAsync(&str)?.get()?;
                let media_source = MediaSource::CreateFromStream(&stream, &stream.ContentType()?)?;
                stream.Close()?;
                player.SetSource(&media_source)?;
                let session = player.PlaybackSession()?;
                player.Play()?;
                loop {
                    sleep(Duration::from_millis(100));
                    if let Ok(state) = &session.PlaybackState() {
                        if state == &MediaPlaybackState::Paused {
                            media_source.Close()?;
                            break;
                        }
                    }
                }
                self.play()?;
            }
            let state_guard = self.state.clone();
            if let Ok(mut state) = state_guard.lock() {
                *state = 0;
            }
            Ok(())
        }
    }
    impl Target for WindowsTTs {
        fn new(device: &str) -> Result<WindowsTTs, Error> {
            if let Ok(cache) = DEVICE_CACHE_INFO.clone().read() {
                let device = cache.get(device).unwrap();
                let player = MediaPlayer::new().expect("MediaPlayer::new failed");
                let _ = player.SetAudioDevice(device);
                return Ok(Self {
                    player: Arc::new(player),
                    queue: Arc::new(Mutex::new(QueueStack::<String>::new())),
                    state: Arc::new(Mutex::new(0)),
                });
            }
            Err(Error::from(windows::core::Error::new(HRESULT::from_win32(0), "")))
        }

        fn devices() -> Vec<String> {
            if let Ok(mut cache) = DEVICE_CACHE_INFO.clone().write() {
                if cache.is_empty() {
                    let async_operation = DeviceInformation::FindAllAsyncDeviceClass(DeviceClass::AudioRender).expect("DeviceInformation::FindAllAsyncDeviceClass failed -- FIND").get().expect("DeviceInformation::FindAllAsyncDeviceClass failed");
                    for x in async_operation {
                        let name = x.Name().expect("Abnormal device name retrieval").to_string();
                        cache.insert(name, x);
                    }
                }
                return cache.keys().map(|k| k.to_string()).collect::<Vec<String>>();
            }
            vec![]
        }

        fn speak(&self, context: String, interrupt: bool) -> Result<(), Error> {
            if context.is_empty() {
                return Ok(());
            }
            if interrupt {
                self.stop()?;
            }

            let queue_guard = self.queue.clone();
            let mut guard = queue_guard.lock().unwrap();
            guard.push(context.clone());
            let state_guard = self.state.clone();
            if let Ok(mut state) = state_guard.lock() {
                if *state == 1u8 {
                    return Ok(());
                } else {
                    *state = 1;
                }
            }
            drop(guard);
            self.play().expect("TODO: panic message");
            Ok(())
        }
        fn is_playing(&self) -> Result<bool, Error> {
            let state_guard = self.state.clone();
            if let Ok(state) = state_guard.lock() {
                return Ok(*state > 0);
            }
            Ok(false)
        }


        fn stop(&self) -> Result<(), Error> {
            let state_guard = self.state.clone();
            if let Ok(state) = state_guard.lock() {
                if *state == 1u8 {
                    let queue_guard = self.queue.clone();
                    let mut guard = queue_guard.lock().unwrap();
                    guard.clear();
                    drop(guard);
                    self.player.clone().Pause()?;
                }
            }
            println!("停止");
            Ok(())
        }
    }


    impl Drop for WindowsTTs {
        fn drop(&mut self) {
            self.player.Close().expect("Player close failed");
        }
    }


    impl From<windows_result::Error> for Error {
        fn from(value: windows_result::Error) -> Self {
            Error::Windows(value)
        }
    }
}