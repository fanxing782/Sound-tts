#[cfg(target_family = "unix")]
pub mod linux {
    use crate::{Error, Target};
    use lazy_static::lazy_static;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    lazy_static! {
        static ref DEVICE_CACHE_INFO:Arc<RwLock<HashMap<String, u64>>> = Arc::new(RwLock::new(HashMap::new()));
    }
    #[derive(Debug)]
    pub(crate) struct LinuxTTs {
        player: u8,
        device_name: String,
    }

    impl Target for LinuxTTs {
        fn new(device: &str) -> Result<LinuxTTs, Error> {
            Ok(LinuxTTs {
                player: 0,
                device_name: String::from(device),
            })
        }

        fn devices() -> Vec<String> {
            vec!["Linux system is temporarily not supported".to_string()]
        }

        fn speak(&self, _context: String, _interrupt: bool) -> Result<(), Error> {
            Ok(())
        }


        fn is_playing(&self) -> Result<bool, Error> {
            Ok(true)
        }

        fn stop(&self) -> Result<(), Error> {
            Ok(())
        }
    }
}