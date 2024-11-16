#[cfg(target_family = "linux")]
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
    }

    impl Target for LinuxTTs {
        fn new(device: &str) -> Result<LinuxTTs, Error> {
            Ok(LinuxTTs {
                player: 0,
            })
        }

        fn devices() -> Vec<String> {
            vec!["Linux system is temporarily not supported".to_string()]
        }

        fn speak(&self, context: String) -> Result<(), Error> {
            Ok(())
        }
    }
}