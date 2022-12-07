namespace Squalr.Engine.Processes
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Logging;
    using System.Diagnostics;
    using System.Threading.Tasks;

    /// <summary>
    /// A container for a process to open. This allows multiple systems to easily detect a changed process by sharing an instance of this class.
    /// </summary>
    public class ProcessSession
    {
        /// <summary>
        /// The current opened process.
        /// </summary>
        private Process openedProcess;

        /// <summary>
        /// Initializes a new instance of the <see cref="ProcessSession" /> class.
        /// </summary>
        /// <param name="processToOpen">The optional initial process to open for this session.</param>
        public ProcessSession(Process processToOpen = null)
        {
            if (processToOpen != null)
            {
                Logger.Log(LogLevel.Info, "Attached to process: " + processToOpen.ProcessName + " (" + processToOpen.Id.ToString() + ")");
            }

            this.DetectedEmulator = EmulatorType.None;
            this.OpenedProcess = processToOpen;

            this.ListenForProcessDeath();
        }

        /// <summary>
        /// Gets or sets a reference to the current target process.
        /// </summary>
        public Process OpenedProcess
        {
            get
            {
                return this.openedProcess;
            }

            set
            {
                if (value == DetachProcess.Instance)
                {
                    this.openedProcess = null;
                }
                else
                {
                    this.openedProcess = value;
                }
            }
        }

        /// <summary>
        /// Gets or sets the detected emulator type. This is not automatically set, because the detection could have dependencies on scanning.
        /// It is up to the caller to store and reuse the detected emulator type here.
        /// </summary>
        public EmulatorType DetectedEmulator { get; set; }

        /// <summary>
        /// Listens for process death and detaches from the process if it closes.
        /// </summary>
        private void ListenForProcessDeath()
        {
            Task.Run(async () =>
            {
                while (true)
                {
                    try
                    {
                        if (this.OpenedProcess?.HasExited ?? false)
                        {
                            this.OpenedProcess = null;
                        }
                    }
                    catch
                    {
                    }

                    await Task.Delay(50);
                }
            });
        }
    }
    //// End class
}
//// End namespace
