namespace Squalr.Engine.Architecture.Assemblers
{
    using System;
    using System.Runtime.InteropServices;

    /// <summary>
    /// A factory that returns an assembler based on the system architecture.
    /// </summary>
    internal class AssemblerFactory
    {
        /// <summary>
        /// Gets an assembler based on the system architecture.
        /// </summary>
        /// <param name="architecture">The system architecture.</param>
        /// <returns>An object implementing IAssembler based on the system architecture.</returns>
        public static IAssembler GetAssembler(Architecture architecture)
        {
            switch (architecture)
            {
                case Architecture.X86:
                case Architecture.X64:
                    return new NasmAssembler();
                default:
                    throw new Exception("Assembler not supported for specified architecture");
            }
        }
    }
    //// End class
}
//// End namespace