namespace Squalr.Engine.Architecture
{
    using Squalr.Engine.Architecture.Assemblers;
    using Squalr.Engine.Architecture.Disassemblers;
    using System;
    using System.Runtime.InteropServices;
    using System.Threading;

    /// <summary>
    /// Factory for obtaining an object that enables debugging of a process.
    /// </summary>
    public class CpuArchitecture : IArchitecture
    {
        /// <summary>
        /// Singleton instance of the <see cref="WindowsMemoryWriter"/> class.
        /// </summary>
        private static readonly Lazy<CpuArchitecture> ArchitectureInstance = new Lazy<CpuArchitecture>(
            () => { return new CpuArchitecture(); },
            LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// Gets a system architecture instance.
        /// </summary>
        /// <returns>A system architecture instance.</returns>
        public static IArchitecture GetInstance()
        {
            return ArchitectureInstance.Value;
        }

        /// <summary>
        /// Gets the architecture of the CPU running Squalr.
        /// </summary>
        /// <returns>The architecture of the CPU running Squalr.</returns>
        public Architecture GetCpuArchitecture()
        {
            return RuntimeInformation.ProcessArchitecture;
        }

        /// <summary>
        /// Gets an instruction assembler for the current CPU architecture.
        /// </summary>
        /// <returns>An instruction assembler for the current CPU architecture.</returns>
        public IAssembler GetAssembler()
        {
            return AssemblerFactory.GetAssembler(this.GetCpuArchitecture());
        }

        /// <summary>
        /// Gets an instruction assembler of the specified architecture.
        /// </summary>
        /// <param name="architecture">The cpu architexture for the assembler.</param>
        /// <returns>An instruction assembler of the specified architecture.</returns>
        public IAssembler GetAssembler(Architecture architecture)
        {
            return AssemblerFactory.GetAssembler(architecture);
        }

        /// <summary>
        /// Gets an instruction disassembler for the current CPU architecture.
        /// </summary>
        /// <returns>An instruction disassembler for the current CPU architecture.</returns>
        public IDisassembler GetDisassembler()
        {
            return DisassemblerFactory.GetDisassembler(this.GetCpuArchitecture());
        }

        /// <summary>
        /// Gets an instruction disassembler of the specified architecture.
        /// </summary>
        /// <param name="architecture">The cpu architexture for the disassembler.</param>
        /// <returns>An instruction disassembler of the specified architecture.</returns>
        public IDisassembler GetDisassembler(Architecture architecture)
        {
            return DisassemblerFactory.GetDisassembler(architecture);
        }
    }
    //// End class
}
//// End namespace