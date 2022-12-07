namespace Squalr.Engine.Architecture.Assemblers
{
    using System;

    /// <summary>
    /// The Fasm assembler for x86/64.
    /// </summary>
    internal class FasmAssembler : IAssembler
    {
        /// <summary>
        /// Assemble the specified assembly code.
        /// </summary>
        /// <param name="assembly">The assembly code.</param>
        /// <param name="isProcess32Bit">Whether or not the assembly is in the context of a 32 bit program.</param>
        /// <returns>An array of bytes containing the assembly code.</returns>
        public AssemblerResult Assemble(String assembly, Boolean isProcess32Bit)
        {
            // Assemble and return the code
            return this.Assemble(assembly, isProcess32Bit, 0);
        }

        /// <summary>
        /// Assemble the specified assembly code at a base address.
        /// </summary>
        /// <param name="assembly">The assembly code.</param>
        /// <param name="isProcess32Bit">Whether or not the assembly is in the context of a 32 bit program.</param>
        /// <param name="baseAddress">The address where the code is rebased.</param>
        /// <returns>An array of bytes containing the assembly code.</returns>
        public AssemblerResult Assemble(String assembly, Boolean isProcess32Bit, UInt64 baseAddress)
        {
            AssemblerResult result = null;

            return result;
        }
    }
    //// End class
}
//// End namespace