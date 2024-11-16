#[cfg(target_family = "windows")]
pub mod windows {
    use crate::{Error, Target};
    use lazy_static::lazy_static;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use std::thread::sleep;
    use std::time::Duration;
    use windows::core::HSTRING;
    use windows::Devices::Enumeration::{DeviceClass, DeviceInformation};
    use windows::Foundation::TypedEventHandler;
    use windows::Media::Core::MediaSource;
    use windows::Media::Playback::{MediaPlaybackState, MediaPlayer};
    use windows::Media::SpeechSynthesis::SpeechSynthesizer;
    use windows_result::HRESULT;

    lazy_static! {
        static ref DEVICE_CACHE_INFO:Arc<RwLock<HashMap<String, DeviceInformation>>> = Arc::new(RwLock::new(HashMap::new()));
    }

    #[derive(Debug)]
    pub(crate) struct WindowsTTs {
        player: MediaPlayer,
    }

    impl Target for WindowsTTs {
        fn new(device: &str) -> Result<WindowsTTs, Error> {
            if let Ok(cache) = DEVICE_CACHE_INFO.clone().read() {
                let player = MediaPlayer::new().expect("MediaPlayer::new failed");
                let device = cache.get(device).unwrap();
                let player = MediaPlayer::new().expect("MediaPlayer::new failed");
                player.MediaEnded(&TypedEventHandler::new(
                    move |sender: &Option<MediaPlayer>, _args| {
                        Ok(())
                    })).expect("MediaPlayer end method creation failed");
                let _ = player.SetAudioDevice(device);
                return Ok(Self {
                    player
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

        fn speak(&self, context: String) -> Result<(), Error> {
            let str = HSTRING::from(&context);
            let synthesizer = SpeechSynthesizer::new()?;
            let stream = synthesizer.SynthesizeTextToStreamAsync(&str)?.get()?;
            let media_source = MediaSource::CreateFromStream(&stream, &stream.ContentType()?)?;
            stream.Close()?;
            self.player.SetSource(&media_source)?;
            let session = self.player.PlaybackSession()?;
            self.player.Play()?;
            loop {
                if let Ok(state) = &session.PlaybackState() {
                    if state == &MediaPlaybackState::Paused {
                        media_source.Close()?;
                        break;
                    }
                }
                sleep(Duration::from_millis(100));
            }
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