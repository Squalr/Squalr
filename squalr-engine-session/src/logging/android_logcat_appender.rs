use crate::logging::log_record_filter::should_suppress_record;
use android_log_sys::{__android_log_write, LogPriority};
use log::{Level, Record};
use log4rs::append::Append;
use std::{ffi::CString, fmt};

pub struct AndroidLogcatAppender {
    tag: CString,
}

impl AndroidLogcatAppender {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: Self::sanitize_c_string(tag),
        }
    }

    fn priority_for_level(level: Level) -> i32 {
        match level {
            Level::Error => LogPriority::ERROR as i32,
            Level::Warn => LogPriority::WARN as i32,
            Level::Info => LogPriority::INFO as i32,
            Level::Debug => LogPriority::DEBUG as i32,
            Level::Trace => LogPriority::VERBOSE as i32,
        }
    }

    fn sanitize_c_string(value: &str) -> CString {
        CString::new(value.replace('\0', " ")).unwrap_or_default()
    }
}

impl fmt::Debug for AndroidLogcatAppender {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        formatter.debug_struct("AndroidLogcatAppender").finish()
    }
}

impl Append for AndroidLogcatAppender {
    fn append(
        &self,
        record: &Record,
    ) -> anyhow::Result<()> {
        if should_suppress_record(record) {
            return Ok(());
        }

        let priority = Self::priority_for_level(record.level());
        let message = Self::sanitize_c_string(&format!("{}", record.args()));

        unsafe {
            __android_log_write(priority, self.tag.as_ptr(), message.as_ptr());
        }

        Ok(())
    }

    fn flush(&self) {}
}
