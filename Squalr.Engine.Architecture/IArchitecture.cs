namespace Squalr.Engine.Architecture
{
    using Assemblers;
    using Disassemblers;
    using System.Runtime.InteropServices;

    /// <summary>
    /// An interface defining an object that can assemble and disassemble instructions.
    /// </summary>
    public interface IArchitecture
    {
        /// <summary>
        /// Gets the architecture of the CPU running Squalr.
        /// </summary>
        /// <returns>The architecture of the CPU running Squalr.</returns>
        Architecture GetCpuArchitecture();

        /// <summary>
        /// Gets an instruction assembler for the current CPU architecture.
        /// </summary>
        /// <returns>An instruction assembler for the current CPU architecture.</returns>
        IAssembler GetAssembler();

        /// <summary>
        /// Gets an instruction assembler of the specified architecture.
        /// </summary>
        /// <param name="architecture">The cpu architexture for the assembler.</param>
        /// <returns>An instruction assembler of the specified architecture.</returns>
        IAssembler GetAssembler(Architecture architecture);

        /// <summary>
        /// Gets an instruction disassembler for the current CPU architecture.
        /// </summary>
        /// <returns>An instruction disassembler for the current CPU architecture.</returns>
        IDisassembler GetDisassembler();

        /// <summary>
        /// Gets an instruction disassembler of the specified architecture.
        /// </summary>
        /// <param name="architecture">The cpu architexture for the disassembler.</param>
        /// <returns>An instruction disassembler of the specified architecture.</returns>
        IDisassembler GetDisassembler(Architecture architecture);
    }
    //// End architecture
}
//// End namespace