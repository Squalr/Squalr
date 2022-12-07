namespace Squalr.Engine.Memory.Windows
{
    using Native;
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.DataStructures;
    using Squalr.Engine.Common.Extensions;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Memory.Windows.PEB;
    using Squalr.Engine.Processes;
    using System;
    using System.Collections.Generic;
    using System.Diagnostics;
    using System.Linq;
    using System.Runtime.InteropServices;
    using System.Text;
    using static Native.Enumerations;
    using static Native.Structures;

    /// <summary>
    /// Class for memory editing a remote process.
    /// </summary>
    internal unsafe class WindowsMemoryQuery : IMemoryQueryer
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="WindowsMemoryQuery"/> class.
        /// </summary>
        public WindowsMemoryQuery()
        {
            this.ModuleCache = new TtlCache<Int32, IList<NormalizedModule>>(TimeSpan.FromSeconds(10.0));
        }

        /// <summary>
        /// Gets or sets the module cache of process modules.
        /// </summary>
        private TtlCache<Int32, IList<NormalizedModule>> ModuleCache { get; set; }

        /// <summary>
        /// Gets regions of memory allocated in the remote process based on provided parameters.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="requiredProtection">Protection flags required to be present.</param>
        /// <param name="excludedProtection">Protection flags that must not be present.</param>
        /// <param name="allowedTypes">Memory types that can be present.</param>
        /// <param name="startAddress">The start address of the query range.</param>
        /// <param name="endAddress">The end address of the query range.</param>
        /// <param name="regionBoundsHandling">An enum specifying how to handle any regions that partially fall within the specified range.</param>
        /// <param name="emulatorType">The process emulator type, if applicable. This is used to collect virtual memory pages from the emulated game, rather than the emulator process entirely.</param>
        /// <returns>A collection of pointers to virtual pages in the target process.</returns>
        public IEnumerable<NormalizedRegion> GetVirtualPages(
            Process process,
            MemoryProtectionEnum requiredProtection,
            MemoryProtectionEnum excludedProtection,
            MemoryTypeEnum allowedTypes,
            UInt64 startAddress,
            UInt64 endAddress,
            RegionBoundsHandling regionBoundsHandling,
            EmulatorType emulatorType)
        {
            return this.GetVirtualPages<NormalizedRegion>(process, requiredProtection, excludedProtection, allowedTypes, startAddress, endAddress, regionBoundsHandling, emulatorType);
        }

        /// <summary>
        /// Gets regions of memory allocated in the remote process based on provided parameters.
        /// </summary>
        /// <typeparam name="T">A type inheriting from <see cref="NormalizedRegion"/>.</typeparam>
        /// <param name="process">The target process.</param>
        /// <param name="requiredProtection">Protection flags required to be present.</param>
        /// <param name="excludedProtection">Protection flags that must not be present.</param>
        /// <param name="allowedTypes">Memory types that can be present.</param>
        /// <param name="startAddress">The start address of the query range.</param>
        /// <param name="endAddress">The end address of the query range.</param>
        /// <param name="regionBoundsHandling">An enum specifying how to handle any regions that partially fall within the specified range.</param>
        /// <param name="emulatorType">The process emulator type, if applicable. This is used to collect virtual memory pages from the emulated game, rather than the emulator process entirely.</param>
        /// <returns>A collection of pointers to virtual pages in the target process.</returns>
        public IEnumerable<T> GetVirtualPages<T>(
            Process process, 
            MemoryProtectionEnum requiredProtection, 
            MemoryProtectionEnum excludedProtection, 
            MemoryTypeEnum allowedTypes,
            UInt64 startAddress,
            UInt64 endAddress,
            RegionBoundsHandling regionBoundsHandling,
            EmulatorType emulatorType) where T : NormalizedRegion, new()
        {
            switch (emulatorType)
            {
                case EmulatorType.AutoDetect:
                    throw new NotImplementedException("Auto detect emulator type not yet supported on GetModules()");
                case EmulatorType.Dolphin:
                    return this.GetDolphinVirtualPages<T>(process);
                case EmulatorType.None:
                    break;
                default:
                    throw new NotImplementedException("Provided emulator type not yet supported on GetModules()");
            }

            MemoryProtectionFlags requiredFlags = 0;
            MemoryProtectionFlags excludedFlags = 0;

            if ((requiredProtection & MemoryProtectionEnum.Write) != 0)
            {
                requiredFlags |= MemoryProtectionFlags.ExecuteReadWrite;
                requiredFlags |= MemoryProtectionFlags.ReadWrite;
            }

            if ((requiredProtection & MemoryProtectionEnum.Execute) != 0)
            {
                requiredFlags |= MemoryProtectionFlags.Execute;
                requiredFlags |= MemoryProtectionFlags.ExecuteRead;
                requiredFlags |= MemoryProtectionFlags.ExecuteReadWrite;
                requiredFlags |= MemoryProtectionFlags.ExecuteWriteCopy;
            }

            if ((requiredProtection & MemoryProtectionEnum.CopyOnWrite) != 0)
            {
                requiredFlags |= MemoryProtectionFlags.WriteCopy;
                requiredFlags |= MemoryProtectionFlags.ExecuteWriteCopy;
            }

            if ((excludedProtection & MemoryProtectionEnum.Write) != 0)
            {
                excludedFlags |= MemoryProtectionFlags.ExecuteReadWrite;
                excludedFlags |= MemoryProtectionFlags.ReadWrite;
            }

            if ((excludedProtection & MemoryProtectionEnum.Execute) != 0)
            {
                excludedFlags |= MemoryProtectionFlags.Execute;
                excludedFlags |= MemoryProtectionFlags.ExecuteRead;
                excludedFlags |= MemoryProtectionFlags.ExecuteReadWrite;
                excludedFlags |= MemoryProtectionFlags.ExecuteWriteCopy;
            }

            if ((excludedProtection & MemoryProtectionEnum.CopyOnWrite) != 0)
            {
                excludedFlags |= MemoryProtectionFlags.WriteCopy;
                excludedFlags |= MemoryProtectionFlags.ExecuteWriteCopy;
            }

            IEnumerable<MemoryBasicInformation64> memoryInfo = WindowsMemoryQuery.VirtualPages(process == null ? IntPtr.Zero : process.Handle, startAddress, endAddress, requiredFlags, excludedFlags, allowedTypes, regionBoundsHandling);
            IList<T> regions = new List<T>();

            foreach (MemoryBasicInformation64 next in memoryInfo)
            {
                T newRegion = new T();

                newRegion.GenericConstructor(next.BaseAddress.ToUInt64(), next.RegionSize.ToInt32());
                regions.Add(newRegion);
            }

            return regions;
        }

        /// <summary>
        /// Gets all virtual pages in the opened process.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="emulatorType">The process emulator type, if applicable. This is used to collect virtual memory pages from the emulated game, rather than the emulator process entirely.</param>
        /// <returns>A collection of regions in the process.</returns>
        public IEnumerable<NormalizedRegion> GetAllVirtualPages(Process process, EmulatorType emulatorType)
        {
            return this.GetAllVirtualPages<NormalizedRegion>(process, emulatorType);
        }

        /// <summary>
        /// Gets all virtual pages in the opened process.
        /// </summary>
        /// <typeparam name="T">A type inheriting from <see cref="NormalizedRegion"/>.</typeparam>
        /// <param name="process">The target process.</param>
        /// <param name="emulatorType">The process emulator type, if applicable. This is used to collect virtual memory pages from the emulated game, rather than the emulator process entirely.</param>
        /// <returns>A collection of regions in the process.</returns>
        public IEnumerable<T> GetAllVirtualPages<T>(Process process, EmulatorType emulatorType) where T : NormalizedRegion, new()
        {
            MemoryTypeEnum flags = MemoryTypeEnum.None | MemoryTypeEnum.Private | MemoryTypeEnum.Image | MemoryTypeEnum.Mapped;

            return this.GetVirtualPages<T>(process, 0, 0, flags, 0, this.GetMaximumAddress(process), RegionBoundsHandling.Exclude, emulatorType);
        }

        /// <summary>
        /// Gets a value indicating whether an address is writable.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="address">The address to check for writability.</param>
        /// <returns>A value indicating whether the given address is writable.</returns>
        public bool IsAddressWritable(Process process, UInt64 address)
        {
            MemoryTypeEnum flags = MemoryTypeEnum.None | MemoryTypeEnum.Private | MemoryTypeEnum.Image | MemoryTypeEnum.Mapped;

            IEnumerable<MemoryBasicInformation64> memoryInfo = WindowsMemoryQuery.VirtualPages(process == null ? IntPtr.Zero : process.Handle, address, address + 1, 0, 0, flags);

            if (memoryInfo.Count() > 0)
            {
                return (memoryInfo.First().Protect & (MemoryProtectionFlags.ExecuteReadWrite | MemoryProtectionFlags.ReadWrite)) != 0;
            }

            return false;
        }

        /// <summary>
        /// Gets the maximum address possible in the target process.
        /// </summary>
        /// <returns>The maximum address possible in the target process.</returns>
        public UInt64 GetMaximumAddress(Process process)
        {
            if (IntPtr.Size == Conversions.SizeOf(ScannableType.Int32))
            {
                return unchecked(UInt32.MaxValue);
            }
            else if (IntPtr.Size == Conversions.SizeOf(ScannableType.Int64))
            {
                return unchecked(UInt64.MaxValue);
            }

            throw new Exception("Unable to determine maximum address");
        }

        /// <summary>
        /// Gets the minimum usermode address possible in the target process.
        /// </summary>
        /// <returns>The minimum usermode address possible in the target process.</returns>
        public UInt64 GetMinUsermodeAddress(Process process)
        {
            return UInt16.MaxValue;
        }

        /// <summary>
        /// Gets the maximum usermode address possible in the target process.
        /// </summary>
        /// <returns>The maximum usermode address possible in the target process.</returns>
        public UInt64 GetMaxUsermodeAddress(Process process)
        {
            if (process.Is32Bit())
            {
                return Int32.MaxValue;
            }
            else
            {
                return 0x7FFFFFFFFFF;
            }
        }

        /// <summary>
        /// Gets all modules in the opened process.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="emulatorType">The process emulator type, if applicable. This is used to collect modules from the emulated game, rather than the emulator process entirely.</param>
        /// <returns>A collection of modules in the process.</returns>
        public IEnumerable<NormalizedModule> GetModules(Process process, EmulatorType emulatorType)
        {
            if (process == null)
            {
                return new List<NormalizedModule>();
            }

            switch (emulatorType)
            {
                case EmulatorType.AutoDetect:
                    throw new NotImplementedException("Auto detect emulator type not yet supported on GetModules()");
                case EmulatorType.Dolphin:
                    return this.GetDolphinModules(process);
                case EmulatorType.None:
                    break;
                default:
                    throw new NotImplementedException("Provided emulator type not yet supported on GetModules()");
            }

            if (this.ModuleCache.Contains(process.Id) && this.ModuleCache.TryGetValue(process.Id, out IList<NormalizedModule> outMoudles))
            {
                return outMoudles?.SoftClone() ?? new List<NormalizedModule>();
            }

            IList<NormalizedModule> modules = new List<NormalizedModule>();

            try
            {
                // Query all modules in the target process
                IntPtr[] modulePointers = new IntPtr[0];
                Int32 bytesNeeded;

                // Determine number of modules
                if (!NativeMethods.EnumProcessModulesEx(process.Handle, modulePointers, 0, out bytesNeeded, (UInt32)Enumerations.ModuleFilter.ListModulesAll))
                {
                    // Failure, return our current empty list
                    return modules;
                }

                Int32 totalNumberofModules = bytesNeeded / IntPtr.Size;
                modulePointers = new IntPtr[totalNumberofModules];

                if (NativeMethods.EnumProcessModulesEx(process.Handle, modulePointers, bytesNeeded, out bytesNeeded, (UInt32)Enumerations.ModuleFilter.ListModulesAll))
                {
                    for (Int32 index = 0; index < totalNumberofModules; index++)
                    {
                        StringBuilder moduleFilePath = new StringBuilder(1024);
                        NativeMethods.GetModuleFileNameEx(process.Handle, modulePointers[index], moduleFilePath, (UInt32)moduleFilePath.Capacity);

                        ModuleInformation moduleInformation = new ModuleInformation();
                        NativeMethods.GetModuleInformation(process.Handle, modulePointers[index], out moduleInformation, (UInt32)(IntPtr.Size * modulePointers.Length));

                        // Ignore modules in 64-bit address space for WoW64 processes
                        if (process.Is32Bit() && moduleInformation.ModuleBase.ToUInt64() > Int32.MaxValue)
                        {
                            continue;
                        }

                        // Convert to a normalized module and add it to our list
                        NormalizedModule module = new NormalizedModule(moduleFilePath.ToString(), moduleInformation.ModuleBase.ToUInt64(), (Int32)moduleInformation.SizeOfImage);
                        modules.Add(module);
                    }
                }
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Unable to fetch modules from selected process", ex);
            }

            this.ModuleCache.Add(process.Id, modules);

            return modules;
        }

        /// <summary>
        /// Gets the address of the stacks in the opened process.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="emulatorType">The process emulator type, if applicable. This is used to collect stack addresses from the emulated game, rather than the emulator process entirely.</param>
        /// <returns>A pointer to the stacks of the opened process.</returns>
        public IEnumerable<NormalizedRegion> GetStackAddresses(Process process, EmulatorType emulatorType)
        {
            throw new NotImplementedException();
        }

        /// <summary>
        /// Gets the addresses of the heaps in the opened process.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="emulatorType">The process emulator type, if applicable. This is used to collect heap addresses from the emulated game, rather than the emulator process entirely.</param>
        /// <returns>A collection of pointers to all heaps in the opened process.</returns>
        public IEnumerable<NormalizedRegion> GetHeapAddresses(Process process, EmulatorType emulatorType)
        {
            switch (emulatorType)
            {
                case EmulatorType.AutoDetect:
                    throw new NotImplementedException("Auto detect emulator type not yet supported on GetModules()");
                case EmulatorType.Dolphin:
                    return this.GetDolphinHeaps(process);
                case EmulatorType.None:
                    break;
                default:
                    throw new NotImplementedException("Provided emulator type not yet supported on GetModules()");
            }

            ManagedPeb peb = new ManagedPeb(process == null ? IntPtr.Zero : process.Handle);

            throw new NotImplementedException();
        }

        /// <summary>
        /// Converts an address to a module and an address offset.
        /// </summary>
        /// <param name="address">The original address.</param>
        /// <param name="moduleName">The module name containing this address, if there is one. Otherwise, empty string.</param>
        /// <returns>The module name and address offset. If not contained by a module, the original address is returned.</returns>
        public UInt64 AddressToModule(Process process, UInt64 address, out String moduleName, EmulatorType emulatorType)
        {
            NormalizedModule containingModule = this.GetModules(process, emulatorType)
                .Select(module => module)
                .Where(module => module.ContainsAddress(address))
                .FirstOrDefault();

            moduleName = containingModule?.Name ?? String.Empty;

            return containingModule == null ? address : address - containingModule.BaseAddress;
        }

        /// <summary>
        /// Determines the base address of a module given a module name.
        /// </summary>
        /// <param name="identifier">The module identifier, or name.</param>
        /// <returns>The base address of the module.</returns>
        public UInt64 ResolveModule(Process process, String identifier, EmulatorType emulatorType)
        {
            UInt64 result = 0;

            identifier = identifier?.RemoveSuffixes(true, ".exe", ".dll");
            IEnumerable<NormalizedModule> modules = this.GetModules(process, emulatorType)
                ?.ToList()
                ?.Select(module => module)
                ?.Where(module => module.Name.RemoveSuffixes(true, ".exe", ".dll").Equals(identifier, StringComparison.OrdinalIgnoreCase));

            if (modules.Count() > 0)
            {
                result = modules.First().BaseAddress;
            }

            return result;
        }

        /// <summary>
        /// Retrieves information about a range of pages within the virtual address space of a specified process.
        /// </summary>
        /// <param name="processHandle">A handle to the process whose memory information is queried.</param>
        /// <param name="startAddress">A pointer to the starting address of the region of pages to be queried.</param>
        /// <param name="endAddress">A pointer to the ending address of the region of pages to be queried.</param>
        /// <returns>
        /// A collection of <see cref="MemoryBasicInformation64"/> structures containing info about all virtual pages in the target process.
        /// </returns>
        public static IEnumerable<MemoryBasicInformation64> QueryUnallocatedMemory(IntPtr processHandle, UInt64 startAddress, UInt64 endAddress)
        {
            if (startAddress >= endAddress)
            {
                yield return new MemoryBasicInformation64();
            }

            Boolean wrappedAround = false;
            Int32 queryResult;

            // Enumerate the memory pages
            do
            {
                // Allocate the structure to store information of memory
                MemoryBasicInformation64 memoryInfo = new MemoryBasicInformation64();

                if (!Environment.Is64BitProcess)
                {
                    // 32 Bit struct is not the same
                    MemoryBasicInformation32 memoryInfo32 = new MemoryBasicInformation32();

                    // Query the memory region (32 bit native method)
                    queryResult = NativeMethods.VirtualQueryEx(processHandle, startAddress.ToIntPtr(), out memoryInfo32, Marshal.SizeOf(memoryInfo32));

                    // Copy from the 32 bit struct to the 64 bit struct
                    memoryInfo.AllocationBase = memoryInfo32.AllocationBase;
                    memoryInfo.AllocationProtect = memoryInfo32.AllocationProtect;
                    memoryInfo.BaseAddress = memoryInfo32.BaseAddress;
                    memoryInfo.Protect = memoryInfo32.Protect;
                    memoryInfo.RegionSize = memoryInfo32.RegionSize;
                    memoryInfo.State = memoryInfo32.State;
                    memoryInfo.Type = memoryInfo32.Type;
                }
                else
                {
                    // Query the memory region (64 bit native method)
                    queryResult = NativeMethods.VirtualQueryEx(processHandle, startAddress.ToIntPtr(), out memoryInfo, Marshal.SizeOf(memoryInfo));
                }

                // Increment the starting address with the size of the page
                UInt64 previousFrom = startAddress;
                startAddress = startAddress.Add(memoryInfo.RegionSize);

                if (previousFrom > startAddress)
                {
                    wrappedAround = true;
                }

                if ((memoryInfo.State & MemoryStateFlags.Free) != 0)
                {
                    // Return the unallocated memory page
                    yield return memoryInfo;
                }
                else
                {
                    // Ignore actual memory
                    continue;
                }
            }
            while (startAddress < endAddress && queryResult != 0 && !wrappedAround);
        }

        /// <summary>
        /// Dtermines the real address of an emulator address.
        /// </summary>
        /// <param name="process"></param>
        /// <param name="emulatorAddress"></param>
        /// <param name="emulatorType"></param>
        /// <returns></returns>
        public UInt64 EmulatorAddressToRealAddress(Process process, UInt64 emulatorAddress, EmulatorType emulatorType)
        {
            switch (emulatorType)
            {
                case EmulatorType.Dolphin:
                    const UInt64 MemoryBase = 0x80000000;
                    const UInt64 WiiMemoryBase = 0x90000000;

                    if (emulatorAddress < MemoryBase)
                    {
                        return 0;
                    }

                    bool isWiiExtendedMemory = emulatorAddress >= WiiMemoryBase;
                    UInt64 baseRelativeAddress = emulatorAddress - (isWiiExtendedMemory ? WiiMemoryBase : MemoryBase);
                    IEnumerable<NormalizedRegion> dolphinRegions = this.GetDolphinVirtualPages<NormalizedRegion>(process).OrderByDescending(region => region.BaseAddress);

                    if (isWiiExtendedMemory && dolphinRegions.Count() >= 2)
                    {
                        return dolphinRegions.First().BaseAddress + baseRelativeAddress;
                    }

                    if (dolphinRegions.Count() >= 1)
                    {
                        return dolphinRegions.Last().BaseAddress + baseRelativeAddress;
                    }

                    break;
            }

            return 0;
        }

        /// <summary>
        /// Dtermines the real address of an emulator address.
        /// </summary>
        /// <param name="process"></param>
        /// <param name="realAddress"></param>
        /// <param name="emulatorType"></param>
        /// <returns></returns>
        public UInt64 RealAddressToEmulatorAddress(Process process, UInt64 realAddress, EmulatorType emulatorType)
        {
            switch (emulatorType)
            {
                case EmulatorType.Dolphin:
                    const UInt64 MemoryBase = 0x80000000;
                    const UInt64 WiiMemoryBase = 0x90000000;
                    IEnumerable<NormalizedRegion> dolphinRegions = this.GetDolphinVirtualPages<NormalizedRegion>(process).OrderByDescending(region => region.BaseAddress);

                    if (dolphinRegions.Count() >= 2)
                    {
                        NormalizedRegion region = dolphinRegions.First();
                        if (realAddress >= region.BaseAddress)
                        {
                            return realAddress - region.BaseAddress + WiiMemoryBase;
                        }
                    }

                    if (dolphinRegions.Count() >= 1)
                    {
                        NormalizedRegion region = dolphinRegions.Last();
                        if (realAddress >= region.BaseAddress)
                        {
                            return realAddress - region.BaseAddress + MemoryBase;
                        }
                    }

                    break;
            }

            return 0;
        }

        /// <summary>
        /// Retrieves information about a range of pages within the virtual address space of a specified process.
        /// </summary>
        /// <param name="processHandle">A handle to the process whose memory information is queried.</param>
        /// <param name="startAddress">A pointer to the starting address of the region of pages to be queried.</param>
        /// <param name="endAddress">A pointer to the ending address of the region of pages to be queried.</param>
        /// <param name="requiredProtection">Protection flags required to be present.</param>
        /// <param name="excludedProtection">Protection flags that must not be present.</param>
        /// <param name="allowedTypes">Memory types that can be present.</param>
        /// <returns>
        /// A collection of <see cref="MemoryBasicInformation64"/> structures containing info about all virtual pages in the target process.
        /// </returns>
        private static IEnumerable<MemoryBasicInformation64> VirtualPages(
            IntPtr processHandle,
            UInt64 startAddress,
            UInt64 endAddress,
            MemoryProtectionFlags requiredProtection,
            MemoryProtectionFlags excludedProtection,
            MemoryTypeEnum allowedTypes,
            RegionBoundsHandling regionBoundsHandling = RegionBoundsHandling.Exclude)
        {
            if (startAddress >= endAddress)
            {
                yield return new MemoryBasicInformation64();
            }

            Boolean wrappedAround = false;
            Int32 queryResult;
            UInt64 currentAddress = startAddress;

            // If partial matches are supported, we need to enumerate all memory regions. A small optimization may be possible here if we start from the min(0, startAddress - max page size) instead.
            if (regionBoundsHandling == RegionBoundsHandling.Include || regionBoundsHandling == RegionBoundsHandling.Resize)
            {
                currentAddress = 0;
            }

            // Enumerate the memory pages
            do
            {
                // Allocate the structure to store information of memory
                MemoryBasicInformation64 memoryInfo = new MemoryBasicInformation64();

                if (!Environment.Is64BitProcess)
                {
                    // 32 Bit struct is not the same
                    MemoryBasicInformation32 memoryInfo32 = new MemoryBasicInformation32();

                    // Query the memory region (32 bit native method)
                    queryResult = NativeMethods.VirtualQueryEx(processHandle, currentAddress.ToIntPtr(), out memoryInfo32, Marshal.SizeOf(memoryInfo32));

                    // Copy from the 32 bit struct to the 64 bit struct
                    memoryInfo.AllocationBase = memoryInfo32.AllocationBase;
                    memoryInfo.AllocationProtect = memoryInfo32.AllocationProtect;
                    memoryInfo.BaseAddress = memoryInfo32.BaseAddress;
                    memoryInfo.Protect = memoryInfo32.Protect;
                    memoryInfo.RegionSize = memoryInfo32.RegionSize;
                    memoryInfo.State = memoryInfo32.State;
                    memoryInfo.Type = memoryInfo32.Type;
                }
                else
                {
                    // Query the memory region (64 bit native method)
                    queryResult = NativeMethods.VirtualQueryEx(processHandle, currentAddress.ToIntPtr(), out memoryInfo, Marshal.SizeOf(memoryInfo));
                }

                // Increment the starting address with the size of the page
                UInt64 nextAddress = currentAddress.Add(memoryInfo.RegionSize);

                if (currentAddress > nextAddress)
                {
                    wrappedAround = true;
                }

                currentAddress = nextAddress;

                // Ignore free memory. These are unallocated memory regions.
                if ((memoryInfo.State & MemoryStateFlags.Free) != 0)
                {
                    continue;
                }

                // At least one readable memory flag is required
                if ((memoryInfo.Protect & MemoryProtectionFlags.ReadOnly) == 0 && (memoryInfo.Protect & MemoryProtectionFlags.ExecuteRead) == 0 &&
                    (memoryInfo.Protect & MemoryProtectionFlags.ExecuteReadWrite) == 0 && (memoryInfo.Protect & MemoryProtectionFlags.ReadWrite) == 0)
                {
                    continue;
                }

                // Do not bother with this shit, this memory is not worth scanning
                if ((memoryInfo.Protect & MemoryProtectionFlags.ZeroAccess) != 0 || (memoryInfo.Protect & MemoryProtectionFlags.NoAccess) != 0 || (memoryInfo.Protect & MemoryProtectionFlags.Guard) != 0)
                {
                    continue;
                }

                // Enforce allowed types
                switch (memoryInfo.Type)
                {
                    case MemoryTypeFlags.None:
                        if ((allowedTypes & MemoryTypeEnum.None) == 0)
                        {
                            continue;
                        }

                        break;
                    case MemoryTypeFlags.Private:
                        if ((allowedTypes & MemoryTypeEnum.Private) == 0)
                        {
                            continue;
                        }

                        break;
                    case MemoryTypeFlags.Image:
                        if ((allowedTypes & MemoryTypeEnum.Image) == 0)
                        {
                            continue;
                        }

                        break;
                    case MemoryTypeFlags.Mapped:
                        if ((allowedTypes & MemoryTypeEnum.Mapped) == 0)
                        {
                            continue;
                        }

                        break;
                }

                // Ensure at least one required protection flag is set
                if (requiredProtection != 0 && (memoryInfo.Protect & requiredProtection) == 0)
                {
                    continue;
                }

                // Ensure no ignored protection flags are set
                if (excludedProtection != 0 && (memoryInfo.Protect & excludedProtection) != 0)
                {
                    continue;
                }

                UInt64 regionStartAddress = memoryInfo.BaseAddress.ToUInt64();
                UInt64 regionEndAddress = regionStartAddress + (UInt64)memoryInfo.RegionSize;

                // Ignore regions not in the provided bounds
                if (regionEndAddress < startAddress || regionStartAddress > endAddress)
                {
                    continue;
                }

                // Handle regions that are partially in the provided bounds based on given bounds handling method
                if (regionStartAddress < startAddress || regionEndAddress > endAddress)
                {
                    switch (regionBoundsHandling)
                    {
                        case RegionBoundsHandling.Exclude:
                            continue;
                        case RegionBoundsHandling.Include:
                            break;
                        case RegionBoundsHandling.Resize:
                            UInt64 newStartAddress = Math.Max(startAddress, regionStartAddress);
                            UInt64 newEndAddress = Math.Min(endAddress, regionEndAddress);
                            memoryInfo.BaseAddress = (IntPtr)newStartAddress;
                            memoryInfo.RegionSize = (Int64)(newEndAddress - newStartAddress);
                            break;
                    }
                }

                // Return the memory page
                yield return memoryInfo;
            }
            while (currentAddress < endAddress && queryResult != 0 && !wrappedAround);
        }

        /// <summary>
        /// Gets all modules in the opened Dolphin emulator process.
        /// </summary>
        /// <returns>A collection of Dolphin emulator modules in the process.</returns>
        private IEnumerable<NormalizedModule> GetDolphinModules(Process process)
        {
            List<NormalizedModule> modules = new List<NormalizedModule>();

            // GameCube and Wii memory. See https://wiibrew.org/wiki/Memory_map
            NormalizedModule mem1 = new NormalizedModule("mem1", this.EmulatorAddressToRealAddress(process, 0x80000000, EmulatorType.Dolphin), 0x01330000);

            // TODO: If possible, it would be nice to figure out how to parse all .rel files (which are basically .dlls) and add them to the list of static modules.

            modules.Add(mem1);

            return modules;
        }

        /// <summary>
        /// Gets all heap memory in the opened Dolphin emulator process.
        /// </summary>
        /// <returns>A collection of Dolphin emulator heap memory in the process.</returns>
        private IEnumerable<NormalizedRegion> GetDolphinHeaps(Process process)
        {
            List<NormalizedRegion> regions = new List<NormalizedRegion>();

            // GameCube and Wii memory. See https://wiibrew.org/wiki/Memory_map
            NormalizedRegion mem1Dynamic = new NormalizedRegion(this.EmulatorAddressToRealAddress(process, 0x81330000, EmulatorType.Dolphin), (int)(0x817FFFFF - 0x81330000));
            NormalizedRegion mem2 = new NormalizedRegion(this.EmulatorAddressToRealAddress(process, 0x90000000, EmulatorType.Dolphin), 0x04000000);

            regions.Add(mem1Dynamic);
            regions.Add(mem2);

            return regions;
        }

        /// <summary>
        /// Gets all virtual pages for the target emulator in the opened process.
        /// </summary>
        /// <typeparam name="T">A type inheriting from <see cref="NormalizedRegion"/>.</typeparam>
        /// <param name="process">The process from which virtual memory pages are collected.</param>
        /// <returns>A collection of regions in the process.</returns>
        private IEnumerable<T> GetDolphinVirtualPages<T>(Process process) where T : NormalizedRegion, new()
        {
            IntPtr processHandle = process?.Handle ?? IntPtr.Zero;
            IList<T> regions = new List<T>();
            Byte[] layoutMagicGC = { 0x5D, 0x1C, 0x9E, 0xA3 };
            Byte[] layoutMagicWii = { 0xC2, 0x33, 0x9F, 0x3D };
            Byte[] bootCode = { 0x0D, 0x15, 0xEA, 0x5E };

            IEnumerable<T> mappedRegions = this.GetVirtualPages<T>(process, 0, 0, MemoryTypeEnum.Mapped, 0, this.GetMaximumAddress(process), RegionBoundsHandling.Exclude, EmulatorType.None);

            foreach (T region in mappedRegions)
            {
                // Dolphin stores main memory in a memory mapped region of this exact size.
                if (region.RegionSize == 0x2000000 && this.IsRegionBackedByPhysicalMemory(processHandle, region))
                {
                    // Check to see if there is a game id. This should weed out any false positives.
                    bool readSuccess = false;
                    Byte[] gameId = new WindowsMemoryReader().ReadBytes(process, region.BaseAddress, 6, out readSuccess);

                    if (readSuccess)
                    {
                        String gameIdStr = Encoding.ASCII.GetString(gameId);

                        if ((gameIdStr.StartsWith('G') || gameIdStr.StartsWith('R')) && gameIdStr.All(character => Char.IsLetterOrDigit(character)))
                        {
                            // Oddly Dolphin seems to map multiple main memory regions into RAM. These are identical.
                            // Changing values in one will change the other. This means that we can just take the first one we find.
                            regions.Add(region);
                            break;
                        }
                    }
                }
            }

            bool mem2Found = false;
            foreach (T region in mappedRegions)
            {
                // Dolphin stores wii memory in a memory mapped region of this exact size.
                if (region.RegionSize == 0x4000000 && this.IsRegionBackedByPhysicalMemory(processHandle, region))
                {
                    regions.Add(region);
                    mem2Found = true;
                    break;
                }
            }

            // Try private regions if mapped didn't contain mem2
            if (!mem2Found)
            {
                IEnumerable<T> privateRegions = this.GetVirtualPages<T>(process, 0, 0, MemoryTypeEnum.Private, 0, this.GetMaximumAddress(process), RegionBoundsHandling.Exclude, EmulatorType.None);

                foreach (T region in privateRegions)
                {
                    // Dolphin stores wii memory in a memory mapped region of this exact size.
                    if (region.RegionSize == 0x4000000 && this.IsRegionBackedByPhysicalMemory(processHandle, region))
                    {
                        regions.Add(region);
                        mem2Found = true;
                        break;
                    }
                }
            }

            return regions;
        }

        private Boolean IsRegionBackedByPhysicalMemory(IntPtr processHandle, NormalizedRegion region)
        {
            // Taken from Dolphin Memory Engine, this checks that the region is backed by physical memory, which apparently is required.
            MemoryWorkingSetExInformation[] workingSetExInformation = new MemoryWorkingSetExInformation[1];

            workingSetExInformation[0].VirtualAddress = region.BaseAddress.ToIntPtr();

            if (NativeMethods.QueryWorkingSetEx(processHandle, workingSetExInformation, Marshal.SizeOf<MemoryWorkingSetExInformation>()))
            {
                if (workingSetExInformation[0].VirtualAttributes.Valid)
                {
                    return true;
                }
            }

            return false;
        }
    }
    //// End class
}
//// End namespace