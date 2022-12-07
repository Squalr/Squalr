﻿namespace Squalr.Cli.CommandHandlers.Process
{
    using CommandLine;
    using Squalr.Engine;
    using Squalr.Engine.Common;
    using Squalr.Engine.Processes;
    using Squalr.Engine.Scanning;
    using Squalr.Engine.Scanning.Scanners;
    using System;
    using System.Diagnostics;
    using System.Linq;

    [Verb("open", HelpText = "Opens a running process.")]
    public class ProcessOpenOptions
    {
        public Int32 Handle()
        {
            if (SessionManager.Session != null)
            {
                Console.WriteLine("[Warn] - A session is already open, no action taken.");

                return -1;
            }

            if (String.IsNullOrWhiteSpace(this.ProcessTerm))
            {
                Console.WriteLine("[Error] - Please specify a process term.");

                return -1;
            }

            Process process = null;

            if (Int32.TryParse(this.ProcessTerm, out Int32 result))
            {
                process = ProcessQuery.Instance.GetProcesses().Where(x => x.Id == result).FirstOrDefault();
            }
            else
            {
                // Try exact match
                process = ProcessQuery.Instance.GetProcesses().Where(x => x.ProcessName.Equals(this.ProcessTerm)).FirstOrDefault();

                // Try non-case sensitive match
                if (process == null)
                {
                    process = ProcessQuery.Instance.GetProcesses().Where(x => x.ProcessName.Equals(this.ProcessTerm, StringComparison.OrdinalIgnoreCase)).FirstOrDefault();
                }

                // Try contains (match case)
                if (process == null)
                {
                    process = ProcessQuery.Instance.GetProcesses().Where(x => x.ProcessName.Contains(this.ProcessTerm)).FirstOrDefault();
                }

                // Try contains (no match case)
                if (process == null)
                {
                    process = ProcessQuery.Instance.GetProcesses().Where(x => x.ProcessName.Contains(this.ProcessTerm, StringComparison.OrdinalIgnoreCase)).FirstOrDefault();
                }
            }

            if (process == null)
            {
                Console.WriteLine("[Error] - Unable to find specified process.");

                return -1;
            }

            SessionManager.Session = new Session(process);

            if (ScanSettings.EmulatorType == EmulatorType.AutoDetect)
            {
                TrackableTask<EmulatorType> emulatorDector = EmulatorDetector.DetectEmulator(process);

                emulatorDector.OnCompletedEvent += (completedTask) =>
                {
                    SessionManager.Session.DetectedEmulator = emulatorDector.Result;
                };
            }
            else
            {
                SessionManager.Session.DetectedEmulator = ScanSettings.EmulatorType;
            }

            return 0;
        }

        [Option('n', "non-invasive", Required = false, HelpText = "Non-invasive attach")]
        public Boolean NonInvasive { get; set; }

        [Value(0, MetaName = "process term", HelpText = "A term to identify the process. This can be a pid, or a string in the process name.")]
        public String ProcessTerm { get; set; }
    }
    //// End class
}
//// End namespace
