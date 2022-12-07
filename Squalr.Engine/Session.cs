namespace Squalr.Engine
{
    using Squalr.Engine.Processes;
    using Squalr.Engine.Scanning.Snapshots;
    using System.Diagnostics;

    /// <summary>
    /// Contains session information, including the target process in addition to snapshot history.
    /// </summary>
    public class Session : ProcessSession
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="Session" /> class.
        /// </summary>
        /// <param name="processToOpen">The process to open for this session.</param>
        public Session(Process processToOpen) : base(processToOpen)
        {
            this.SnapshotManager = new SnapshotManager();
        }

        /// <summary>
        /// Gets a snapshot manager for managing scan history.
        /// </summary>
        public SnapshotManager SnapshotManager { get; private set; }
    }
    //// End class
}
//// End namespace
