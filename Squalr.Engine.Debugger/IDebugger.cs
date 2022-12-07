namespace Squalr.Engine.Debuggers
{
    using System;
    using System.Diagnostics;
    using System.Threading;

    public delegate void MemoryAccessCallback(CodeTraceInfo codeTraceInfo);

    public delegate Boolean DebugRequestCallback();

    public enum BreakpointSize
    {
        B1 = 1,
        B2 = 2,
        B4 = 4,
        B8 = 8,
    }

    public interface IDebugger
    {
        void SetTargetProcess(Process process);

        CancellationTokenSource FindWhatReads(UInt64 address, BreakpointSize size, MemoryAccessCallback callback);

        CancellationTokenSource FindWhatWrites(UInt64 address, BreakpointSize size, MemoryAccessCallback callback);

        CancellationTokenSource FindWhatAccesses(UInt64 address, BreakpointSize size, MemoryAccessCallback callback);

        void PauseExecution();

        void ResumeExecution();

        void WriteRegister(UInt32 registerId, UInt64 value);

        UInt64 ReadRegister(UInt32 registerId);

        void WriteInstructionPointer(UInt64 value);

        UInt64 ReadInstructionPointer();

        Boolean IsAttached { get; }

        /// <summary>
        /// Gets or sets the debug request callback. If this is set, this callback will be called before the debugger is attached.
        /// The debugger will only perform the attach if the result of the callback is true.
        /// </summary>
        DebugRequestCallback DebugRequestCallback { get; set; }
    }
    //// End interface
}
//// End namespace
