namespace Squalr.Engine.Processes.OS.Windows.Native
{
    using System;
    using System.Runtime.InteropServices;

    /// <summary>
    /// Static class referencing all P/Invoked functions used by the library.
    /// </summary>
    internal static class NativeMethods
    {
        /// <summary>
        /// A callback function to be used in the EnumWindows() Windows API call.
        /// </summary>
        /// <param name="hwnd">A handle to the window.</param>
        /// <param name="lParam">An application-defined value to be passed to the callback function.</param>
        /// <returns>A value indicating whether enumeration should continue.</returns>
        public delegate Boolean EnumWindowsProc(IntPtr hwnd, Int32 lParam);

        /// <summary>
        /// Enumerates all top-level windows on the screen by passing the handle to each window, in turn, to an application-defined callback function.
        /// </summary>
        /// <param name="enumFunc">A pointer to an application-defined callback function.</param>
        /// <param name="lParam">An application-defined value to be passed to the callback function.</param>
        /// <returns>A value indicating whether the enumeration was successful. For more info, call GetLastError().</returns>
        [DllImport("user32")]
        public static extern Boolean EnumWindows(EnumWindowsProc enumFunc, Int32 lParam);

        /// <summary>
        /// Retrieves the identifier of the thread that created the specified window and, optionally, the identifier of the process that created the window.
        /// </summary>
        /// <param name="handle">A handle to the window.</param>
        /// <param name="processId">A reference to a variable that receives the process identifier.</param>
        /// <returns>The return value is the identifier of the thread that created the window.</returns>
        [DllImport("user32")]
        public static extern Int32 GetWindowThreadProcessId(IntPtr handle, out Int32 processId);

        /// <summary>
        /// Determines the visibility state of the specified window.
        /// </summary>
        /// <param name="hWnd">A handle to the window to be tested.</param>
        /// <returns>Return true if the specified window or any of its parents are visibile.</returns>
        [DllImport("user32")]
        public static extern Boolean IsWindowVisible(IntPtr hWnd);

        /// <summary>
        /// Extracts the icon from a running process
        /// </summary>
        /// <param name="hInst">Handle to the process</param>
        /// <param name="lpszExeFileName">Executable file name</param>
        /// <param name="nIconIndex">Index of the icon</param>
        /// <returns>A handle to the icon in the target process</returns>
        [DllImport("shell32.dll", SetLastError = true)]
        public static extern IntPtr ExtractIcon(IntPtr hInst, String lpszExeFileName, Int32 nIconIndex);

        /// <summary>
        /// Determines whether the specified process is running under WOW64
        /// </summary>
        /// <param name="processHandle">A handle to the running process</param>
        /// <param name="wow64Process">Whether or not the process is 64 bit</param>
        /// <returns>
        /// A pointer to a value that is set to TRUE if the process is running under WOW64.
        /// If the process is running under 32-bit Windows, the value is set to FALSE.
        /// If the process is a 64-bit application running under 64-bit Windows, the value is also set to FALSE.
        /// </returns>
        [DllImport("kernel32.dll", SetLastError = true, CallingConvention = CallingConvention.Winapi)]
        [return: MarshalAs(UnmanagedType.Bool)]
        public static extern Boolean IsWow64Process([In] IntPtr processHandle, [Out, MarshalAs(UnmanagedType.Bool)] out Boolean wow64Process);
    }
    //// End class
}
//// End namespace