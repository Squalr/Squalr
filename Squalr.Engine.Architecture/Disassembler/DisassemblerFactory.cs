namespace Squalr.Engine.Architecture.Disassemblers
{
    using System;
    using System.Runtime.InteropServices;

    /// <summary>
    /// A factory that returns a disassembler based on the system architecture.
    /// </summary>
    internal class DisassemblerFactory
    {
        /// <summary>
        /// Gets a disassembler based on the system architecture.
        /// </summary>
        /// <param name="architecture">The system architecture.</param>
        /// <returns>An object implementing IDisassembler based on the system architecture.</returns>
        public static IDisassembler GetDisassembler(Architecture architecture)
        {
            switch (architecture)
            {
                case Architecture.X86:
                case Architecture.X64:
                    return new SharpDisassembler();
                default:
                    throw new Exception("Assembler not supported for specified architecture");
            }
        }
    }
    //// End class
}
//// End namespace