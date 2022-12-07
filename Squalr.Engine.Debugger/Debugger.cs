namespace Squalr.Engine.Debuggers
{
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Debuggers.Windows.DebugEngine;
    using System;
    using System.Threading;

    /// <summary>
    /// Factory for obtaining an object that enables debugging of a process.
    /// </summary>
    public static class Debugger
    {
        /// <summary>
        /// Singleton instance of the <see cref="WindowsMemoryWriter"/> class.
        /// </summary>
        private static readonly Lazy<DebugEngine> windowDebuggerInstance = new Lazy<DebugEngine>(
            () => { return new DebugEngine(); },
            LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// Gets a process debugger instance.
        /// </summary>
        /// <returns>A process debugger instance.</returns>
        public static IDebugger GetInstance()
        {
            OperatingSystem os = Environment.OSVersion;
            PlatformID platformid = os.Platform;
            Exception ex;

            switch (platformid)
            {
                case PlatformID.Win32NT:
                case PlatformID.Win32S:
                case PlatformID.Win32Windows:
                case PlatformID.WinCE:
                    return windowDebuggerInstance.Value;
                case PlatformID.Unix:
                    ex = new Exception("Unix operating system is not supported");
                    break;
                case PlatformID.MacOSX:
                    ex = new Exception("MacOSX operating system is not supported");
                    break;
                default:
                    ex = new Exception("Unknown operating system");
                    break;
            }

            Logger.Log(LogLevel.Fatal, "Unsupported Operating System", ex);
            throw ex;
        }
    }
    //// End class
}
//// End namespace