namespace Squalr.Engine.Common.OS
{
    using Squalr.Engine.Common.Logging;
    using System;
    using System.Diagnostics;
    using System.IO;

    /// <summary>
    /// Utility functions for common OS actions.
    /// </summary>
    public static class OSUtils
    {
        /// <summary>
        /// Opens the given file path in the OS native file explorer.
        /// </summary>
        /// <param name="path">The path to open.</param>
        public static void OpenPathInFileExplorer(String path)
        {
            OperatingSystem os = Environment.OSVersion;
            PlatformID platformid = os.Platform;

            if (Directory.Exists(path))
            {
                try
                {
                    switch (platformid)
                    {
                        case PlatformID.Win32NT:
                        case PlatformID.Win32S:
                        case PlatformID.Win32Windows:
                        case PlatformID.WinCE:
                            Process.Start("explorer.exe", path);
                            break;
                        case PlatformID.Unix:
                            throw new Exception("Unix operating system is not supported");
                        case PlatformID.MacOSX:
                            throw new Exception("MacOSX operating system is not supported");
                        default:
                            throw new Exception("Unknown operating system");
                    }
                }
                catch (Exception ex)
                {
                    Logger.Log(LogLevel.Error, "Error opening file explorer", ex);
                }
            }
            else
            {
                Logger.Log(LogLevel.Error, "Unable to open file explorer. Directory does not exist");
            }
        }
    }
}
