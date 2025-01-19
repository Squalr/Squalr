use crate::logging::log_level::LogLevel;
use crate::logging::logger::Logger;
use crate::privileges::android::android_super_user_process::AndroidSuperUserProcess;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Once, RwLock};

pub struct AndroidSuperUser {
    pub child: Option<AndroidSuperUserProcess>,
}

impl AndroidSuperUser {
    /// Construct a new `AndroidSuperUser`, possibly with no child if spawn failed.
    fn new() -> Self {
        Self::spawn_su()
    }

    pub fn get_instance() -> Arc<RwLock<AndroidSuperUser>> {
        static mut INSTANCE: Option<Arc<RwLock<AndroidSuperUser>>> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Arc::new(RwLock::new(AndroidSuperUser::new()));
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked().clone()
        }
    }

    /// Attempt to spawn `su`. If **any** step fails, `child` is `None`.
    /// If successful, wraps everything in `AndroidSuperUserProcess`.
    fn spawn_su() -> Self {
        let child_opt = match Command::new("su")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // .stderr(Stdio::piped()) // Uncomment if you need stderr
            .spawn()
        {
            Ok(mut child_proc) => {
                // Try to take stdin and stdout
                let child_stdin = child_proc.stdin.take().map(BufWriter::new);
                let child_stdout = child_proc.stdout.take().map(BufReader::new);

                if let (Some(stdin), Some(stdout)) = (child_stdin, child_stdout) {
                    Some(AndroidSuperUserProcess {
                        child_process: child_proc,
                        child_stdin: stdin,
                        child_stdout: stdout,
                    })
                } else {
                    // If either is None, log the error
                    Logger::get_instance().log(LogLevel::Error, "Failed to open stdin or stdout for `su`.", None);
                    None
                }
            }
            Err(e) => {
                // Log the spawn failure
                Logger::get_instance().log(LogLevel::Error, "Failed to spawn `su` command.", Some(e.to_string().as_str()));
                None
            }
        };

        Self { child: child_opt }
    }

    /// Execute a command via the running `su` process, if it exists.
    pub fn execute_command(
        &mut self,
        command: &str,
    ) -> std::io::Result<Vec<String>> {
        // Make sure we actually have a running process
        let child = match self.child.as_mut() {
            Some(child) => child,
            None => {
                Logger::get_instance().log(LogLevel::Error, "No `su` process running; cannot execute command.", None);
                return Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "No su process running"));
            }
        };

        // Send the command
        writeln!(child.child_stdin, "{}", command)?;
        writeln!(child.child_stdin, "echo COMMAND_DONE")?;
        child.child_stdin.flush()?;

        // Now read lines until we see COMMAND_DONE
        let mut output_lines = Vec::new();
        loop {
            let mut buf = String::new();
            let bytes_read = child.child_stdout.read_line(&mut buf)?;
            if bytes_read == 0 {
                // `su` closed, or something went wrong
                break;
            }
            if buf.trim() == "COMMAND_DONE" {
                // We’ve reached the marker line, so stop reading
                break;
            }
            output_lines.push(buf);
        }

        Ok(output_lines)
    }

    /// Check if `su` is still alive, returning false if we have no child.
    pub fn is_alive(&mut self) -> bool {
        match self.child.as_mut() {
            Some(child_proc) => match child_proc.child_process.try_wait() {
                // None => child is still running
                Ok(None) => true,
                // Some(status) => child has exited
                Ok(Some(_status)) => false,
                // Error checking => treat as dead
                Err(_) => false,
            },
            None => false,
        }
    }

    /// If `su` is dead or wasn't started, re-spawn it.
    pub fn ensure_alive(&mut self) -> std::io::Result<()> {
        if !self.is_alive() {
            Logger::get_instance().log(LogLevel::Error, "Re-spawning `su` (previous instance dead).", None);
            let new_self = Self::spawn_su();
            self.child = new_self.child;
        }
        Ok(())
    }
}
