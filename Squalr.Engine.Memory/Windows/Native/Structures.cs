namespace Squalr.Engine.Memory.Windows.Native
{
    using Squalr.Engine.Common.Extensions;
    using System;
    using System.Runtime.InteropServices;
    using static Enumerations;

    /// <summary>
    /// Class containing native Windows structures.
    /// </summary>
    internal class Structures
    {
        [StructLayout(LayoutKind.Sequential, Pack = 1)]
        internal struct ProcessBasicInformation
        {
            /// <summary>
            /// The exit status.
            /// </summary>
            public IntPtr ExitStatus;

            /// <summary>
            /// The base address of Process Environment Block.
            /// </summary>
            public IntPtr PebBaseAddress;

            /// <summary>
            /// The affinity mask.
            /// </summary>
            public IntPtr AffinityMask;

            /// <summary>
            /// The base priority.
            /// </summary>
            public IntPtr BasePriority;

            /// <summary>
            /// The process id.
            /// </summary>
            public UIntPtr UniqueProcessId;

            /// <summary>
            /// The process id of the parent process.
            /// </summary>
            public IntPtr InheritedFromUniqueProcessId;

            /// <summary>
            /// Gets the size of this structure.
            /// </summary>
            public Int32 Size
            {
                get
                {
                    return Marshal.SizeOf(typeof(ProcessBasicInformation));
                }
            }
        }

        /// <summary>
        /// Contains information about a module in an external process.
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        internal struct ModuleInformation
        {
            /// <summary>
            /// The base address of the module.
            /// </summary>
            public IntPtr ModuleBase;

            /// <summary>
            /// The size of the module.
            /// </summary>
            public UInt32 SizeOfImage;

            /// <summary>
            /// The entry point of the module.
            /// </summary>
            public IntPtr EntryPoint;
        }

        /// <summary>
        /// Contains information about a range of pages in the virtual address space of a 32 bit process. The VirtualQuery and 
        /// <see cref="NativeMethods.VirtualQueryEx"/> functions use this structure
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        internal struct MemoryBasicInformation32
        {
            /// <summary>
            /// A pointer to the base address of the region of pages
            /// </summary>
            public IntPtr BaseAddress;

            /// <summary>
            /// A pointer to the base address of a range of pages allocated by the VirtualAlloc function.
            /// The page pointed to by the BaseAddress member is contained within this allocation range
            /// </summary>
            public IntPtr AllocationBase;

            /// <summary>
            /// The memory protection option when the region was initially allocated. This member can be one of the memory protection constants or 0 if the caller does not have access
            /// </summary>
            public MemoryProtectionFlags AllocationProtect;

            /// <summary>
            /// The size of the region beginning at the base address in which all pages have identical attributes, in bytes
            /// </summary>
            public Int32 RegionSize;

            /// <summary>
            /// The state of the pages in the region
            /// </summary>
            public MemoryStateFlags State;

            /// <summary>
            /// The access protection of the pages in the region. This member is one of the values listed for the AllocationProtect member
            /// </summary>
            public MemoryProtectionFlags Protect;

            /// <summary>
            /// The type of pages in the region
            /// </summary>
            public MemoryTypeFlags Type;
        }

        /// <summary>
        /// Contains information about a range of pages in the virtual address space of a 64 bit process. The VirtualQuery and 
        /// <see cref="NativeMethods.VirtualQueryEx"/> functions use this structure
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        internal struct MemoryBasicInformation64
        {
            /// <summary>
            /// A pointer to the base address of the region of pages
            /// </summary>
            public IntPtr BaseAddress;

            /// <summary>
            /// A pointer to the base address of a range of pages allocated by the VirtualAlloc function. The page pointed to by the
            /// BaseAddress member is contained within this allocation range
            /// </summary>
            public IntPtr AllocationBase;

            /// <summary>
            /// The memory protection option when the region was initially allocated. This member can be one of the memory 
            /// protection constants or 0 if the caller does not have access
            /// </summary>
            public MemoryProtectionFlags AllocationProtect;

            /// <summary>
            /// Required in the 64 bit struct. Blame Windows
            /// </summary>
            public UInt32 Alignment1;

            /// <summary>
            /// The size of the region beginning at the base address in which all pages have identical attributes, in bytes
            /// </summary>
            public Int64 RegionSize;

            /// <summary>
            /// The state of the pages in the region
            /// </summary>
            public MemoryStateFlags State;

            /// <summary>
            /// The access protection of the pages in the region. This member is one of the values listed for the AllocationProtect member
            /// </summary>
            public MemoryProtectionFlags Protect;

            /// <summary>
            /// The type of pages in the region
            /// </summary>
            public MemoryTypeFlags Type;

            /// <summary>
            /// Required in the 64 bit struct. Blame Windows
            /// </summary>
            public UInt32 Alignment2;
        }

        [StructLayout(LayoutKind.Sequential)]
        internal struct MemoryWorkSetExBlock
        {
            private IntPtr flags;

            public Int64 Flags => this.flags.ToInt64();

            public Boolean Valid => this.Flags.GetBit(0);

            public Int64 ShareCount => this.Flags.GetBits(1, 3);

            public MemoryProtectionFlags Win32Protection => (MemoryProtectionFlags)this.Flags.GetBits(4, 11);

            public Boolean Shared => this.Flags.GetBit(15);

            public Int64 Node => this.Flags.GetBits(16, 6);

            public Boolean Locked => this.Flags.GetBit(22);

            public Boolean Bad => this.Flags.GetBit(31);
        }

        [StructLayout(LayoutKind.Sequential)]
        internal struct MemoryWorkingSetExInformation
        {
            public IntPtr VirtualAddress;
            public MemoryWorkSetExBlock VirtualAttributes;
        }
    }
    //// End class
}
//// End namespace