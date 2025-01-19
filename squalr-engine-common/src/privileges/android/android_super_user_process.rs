use std::io::{BufReader, BufWriter};
use std::process::{Child, ChildStdin, ChildStdout};

/// Holds a single, running `su` child process, plus buffered stdin/stdout.
pub struct AndroidSuperUserProcess {
    pub child_process: Child,
    pub child_stdin: BufWriter<ChildStdin>,
    pub child_stdout: BufReader<ChildStdout>,
}
