namespace Squalr.Engine.Memory
{
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Memory.Windows;
    using System;
    using System.Threading;

    /// <summary>
    /// Instantiates the proper memory writer based on the host OS.
    /// </summary>
    public static class MemoryWriter
    {
        /// <summary>
        /// Singleton instance of the <see cref="WindowsMemoryWriter"/> class.
        /// </summary>
        private static readonly Lazy<WindowsMemoryWriter> WindowsMemoryWriterInstance = new Lazy<WindowsMemoryWriter>(
            () => { return new WindowsMemoryWriter(); },
            LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// Gets a <see cref="IMemoryWriter"/> object instance for the current operating system.
        /// </summary>
        /// <returns>An instance of a memory writer.</returns>
        public static IMemoryWriter Instance
        {
            get
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
                        return MemoryWriter.WindowsMemoryWriterInstance.Value;
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
    }
    //// End class
}
//// End namespace