use crate::logging::log_level::LogLevel;
use crate::logging::logger::Logger;
use jni::JNIEnv;
use jni::JavaVM;
use std::sync::Once;

static mut INSTANCE: Option<AndroidGlobals> = None;
static INIT: Once = Once::new();

pub struct AndroidGlobals {
    java_vm: JavaVM,
}

impl AndroidGlobals {
    pub fn init(java_vm: JavaVM) {
        unsafe {
            INIT.call_once(|| {
                INSTANCE = Some(AndroidGlobals::new(java_vm));
            });
        }
    }

    pub fn get_instance() -> &'static AndroidGlobals {
        unsafe {
            // If init() has never been called before, panic.
            if !INIT.is_completed() {
                panic!("Attempted to use engine before it was initialized.");
            }

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap()
        }
    }

    fn new(java_vm: JavaVM) -> Self {
        Self { java_vm }
    }

    // Get a JNIEnv for the current thread.
    pub fn get_env(&self) -> Result<JNIEnv, String> {
        match self.java_vm.get_env() {
            Ok(env) => {
                // Already attached, return the existing JNIEnv.
                Ok(env)
            }
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Unexpected JNI error: {}", e), None);
                Err(format!("Unexpected JNI error: {}", e))
            }
        }
    }

    // Get the JavaVM instance.
    pub fn get_java_vm(&self) -> &JavaVM {
        &self.java_vm
    }
}
