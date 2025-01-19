use std::io::{BufReader, BufWriter};
use std::process::{Child, ChildStdin, ChildStdout};

// Holds the running `su` child process and buffered IO handles.
pub struct AndroidSuperUserProcess {
    pub child_process: Child,
    pub child_stdin: BufWriter<ChildStdin>,
    pub child_stdout: BufReader<ChildStdout>,
}
