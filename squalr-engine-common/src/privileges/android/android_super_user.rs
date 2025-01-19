use crate::logging::log_level::LogLevel;
use crate::logging::logger::Logger;
use crate::privileges::android::android_super_user_process::AndroidSuperUserProcess;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Once, RwLock};

pub struct AndroidSuperUser {
    /// Shell used for line-based commands (execute_command).
    pub child_text: Option<AndroidSuperUserProcess>,

    /// Shell used for binary reads (read_memory_chunk).
    pub child_binary: Option<AndroidSuperUserProcess>,
}

impl AndroidSuperUser {
    /// Private constructor that spawns both shells.
    fn new() -> Self {
        let child_text = Self::spawn_su_text();
        let child_binary = Self::spawn_su_binary();
        Self { child_text, child_binary }
    }

    /// Singleton accessor
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

    /// Spawns an `su` shell for text commands (with `COMMAND_DONE` markers).
    fn spawn_su_text() -> Option<AndroidSuperUserProcess> {
        match Command::new("su")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // .stderr(Stdio::piped()) // if needed
            .spawn()
        {
            Ok(mut child_proc) => {
                let child_stdin = child_proc.stdin.take().map(BufWriter::new);
                let child_stdout = child_proc.stdout.take().map(BufReader::new);

                if let (Some(stdin), Some(stdout)) = (child_stdin, child_stdout) {
                    Some(AndroidSuperUserProcess {
                        child_process: child_proc,
                        child_stdin: stdin,
                        child_stdout: stdout,
                    })
                } else {
                    Logger::get_instance().log(LogLevel::Error, "Failed to open stdin/stdout for `su` (text).", None);
                    None
                }
            }
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, "Failed to spawn `su` (text).", Some(&e.to_string()));
                None
            }
        }
    }

    /// Spawns an `su` shell for *binary* reads (no line-based “COMMAND_DONE” usage).
    fn spawn_su_binary() -> Option<AndroidSuperUserProcess> {
        match Command::new("su")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(mut child_proc) => {
                let child_stdin = child_proc.stdin.take().map(BufWriter::new);
                let child_stdout = child_proc.stdout.take().map(BufReader::new);

                if let (Some(stdin), Some(stdout)) = (child_stdin, child_stdout) {
                    Some(AndroidSuperUserProcess {
                        child_process: child_proc,
                        child_stdin: stdin,
                        child_stdout: stdout,
                    })
                } else {
                    Logger::get_instance().log(LogLevel::Error, "Failed to open stdin/stdout for `su` (binary).", None);
                    None
                }
            }
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, "Failed to spawn `su` (binary).", Some(&e.to_string()));
                None
            }
        }
    }

    /// Execute a **text command** in the text-based shell.
    /// Uses `"COMMAND_DONE"` sentinel line to mark completion.
    pub fn execute_command(
        &mut self,
        command: &str,
    ) -> std::io::Result<Vec<String>> {
        // Ensure the shell is alive
        self.ensure_text_shell_alive()?;

        let child_text = self
            .child_text
            .as_mut()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotConnected, "Text shell not available"))?;

        // Write the command
        writeln!(child_text.child_stdin, "{}", command)?;
        // Write our sentinel
        writeln!(child_text.child_stdin, "echo COMMAND_DONE")?;
        child_text.child_stdin.flush()?;

        // Read lines until we see the sentinel
        let mut output_lines = Vec::new();
        loop {
            let mut buf = String::new();
            let bytes_read = child_text.child_stdout.read_line(&mut buf)?;
            if bytes_read == 0 {
                // EOF => shell is gone
                break;
            }
            if buf.trim() == "COMMAND_DONE" {
                break;
            }
            output_lines.push(buf);
        }
        Ok(output_lines)
    }

    /// Check if the text shell is alive, else re-spawn it.
    pub fn ensure_text_shell_alive(&mut self) -> std::io::Result<()> {
        if !Self::is_child_alive(self.child_text.as_mut()) {
            Logger::get_instance().log(LogLevel::Error, "Re-spawning text shell (previous instance dead).", None);
            self.child_text = Self::spawn_su_text();
        }
        Ok(())
    }

    /// Check if the binary shell is alive, else re-spawn it.
    pub fn ensure_binary_shell_alive(&mut self) -> std::io::Result<()> {
        if !Self::is_child_alive(self.child_binary.as_mut()) {
            Logger::get_instance().log(LogLevel::Error, "Re-spawning binary shell (previous instance dead).", None);
            self.child_binary = Self::spawn_su_binary();
        }
        Ok(())
    }

    /// Generic function to check if a given child is still alive.
    fn is_child_alive(child_opt: Option<&mut AndroidSuperUserProcess>) -> bool {
        if let Some(child) = child_opt {
            match child.child_process.try_wait() {
                // `Ok(None)` => still running
                Ok(None) => true,
                // `Ok(Some(_status))` => exited
                Ok(Some(_)) => false,
                // Error => treat as dead
                Err(_) => false,
            }
        } else {
            false
        }
    }
}
