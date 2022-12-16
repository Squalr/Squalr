namespace Squalr.Engine.Memory
{
    using Squalr.Engine.Common;
    using System;
    using System.Collections.Generic;
    using System.Diagnostics;

    /// <summary>
    /// An enum specifying how to handle any regions that partially fall within the specified range.
    /// </summary>
    public enum RegionBoundsHandling
    {
        /// <summary>
        /// Exclude the entire region that is partially outside of the specified range.
        /// </summary>
        Exclude,

        /// <summary>
        /// Include the entire region that is partially outside of the specified range.
        /// </summary>
        Include,

        /// <summary>
        /// Resize region that is partially outside of the specified range to fit within the range.
        /// </summary>
        Resize,
    }

    /// <summary>
    /// An interface for querying virtual memory.
    /// </summary>
    public interface IMemoryQueryer
    {
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
        IEnumerable<NormalizedRegion> GetVirtualPages(
            Process process,
            MemoryProtectionEnum requiredProtection,
            MemoryProtectionEnum excludedProtection,
            MemoryTypeEnum allowedTypes,
            UInt64 startAddress,
            UInt64 endAddress,
            RegionBoundsHandling regionBoundsHandling = RegionBoundsHandling.Exclude,
            EmulatorType emulatorType = EmulatorType.None);

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
        IEnumerable<T> GetVirtualPages<T>(
            Process process,
            MemoryProtectionEnum requiredProtection,
            MemoryProtectionEnum excludedProtection,
            MemoryTypeEnum allowedTypes,
            UInt64 startAddress,
            UInt64 endAddress,
            RegionBoundsHandling regionBoundsHandling = RegionBoundsHandling.Exclude,
            EmulatorType emulatorType = EmulatorType.None) where T : NormalizedRegion, new();

        /// <summary>
        /// Gets all virtual pages in the opened process.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="emulatorType">The process emulator type, if applicable. This is used to collect virtual memory pages from the emulated game, rather than the emulator process entirely.</param>
        /// <returns>A collection of regions in the process.</returns>
        IEnumerable<NormalizedRegion> GetAllVirtualPages(Process process, EmulatorType emulatorType = EmulatorType.None);

        /// <summary>
        /// Gets all virtual pages in the opened process.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="emulatorType">The process emulator type, if applicable. This is used to collect virtual memory pages from the emulated game, rather than the emulator process entirely.</param>
        /// <typeparam name="T">A type inheriting from <see cref="NormalizedRegion"/>.</typeparam>
        /// <returns>A collection of regions in the process.</returns>
        IEnumerable<T> GetAllVirtualPages<T>(Process process, EmulatorType emulatorType = EmulatorType.None) where T : NormalizedRegion, new();

        /// <summary>
        /// Gets a value indicating whether an address is writable.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="address">The address to check for writability.</param>
        /// <returns>A value indicating whether the given address is writable.</returns>
        Boolean IsAddressWritable(Process process, UInt64 address);

        /// <summary>
        /// Gets the maximum address possible in the target process.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <returns>The maximum address possible in the target process.</returns>
        UInt64 GetMaximumAddress(Process process);

        /// <summary>
        /// Gets the maximum usermode address possible in the target process.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <returns>The maximum usermode address possible in the target process.</returns>
        UInt64 GetMinUsermodeAddress(Process process);

        /// <summary>
        /// Gets the maximum usermode address possible in the target process.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <returns>The maximum usermode address possible in the target process.</returns>
        UInt64 GetMaxUsermodeAddress(Process process);

        /// <summary>
        /// Gets all modules in the opened process.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="emulatorType">The process emulator type, if applicable. This is used to collect modules from the emulated game, rather than the emulator process entirely.</param>
        /// <returns>A collection of modules in the process.</returns>
        IEnumerable<NormalizedModule> GetModules(Process process, EmulatorType emulatorType = EmulatorType.None);

        /// <summary>
        /// Gets the address of the stacks in the opened process.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="emulatorType">The process emulator type, if applicable. This is used to collect stack addresses from the emulated game, rather than the emulator process entirely.</param>
        /// <returns>A pointer to the stacks of the opened process.</returns>
        IEnumerable<NormalizedRegion> GetStackAddresses(Process process, EmulatorType emulatorType = EmulatorType.None);

        /// <summary>
        /// Gets the addresses of the heaps in the opened process.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="emulatorType">The process emulator type, if applicable. This is used to collect heap addresses from the emulated game, rather than the emulator process entirely.</param>
        /// <returns>A collection of pointers to all heaps in the opened process.</returns>
        IEnumerable<NormalizedRegion> GetHeapAddresses(Process process, EmulatorType emulatorType = EmulatorType.None);

        /// <summary>
        /// Converts an address to a module and an address offset.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="address">The original address.</param>
        /// <param name="moduleName">The module name containing this address, if there is one. Otherwise, empty string.</param>
        /// <returns>The module name and address offset. If not contained by a module, the original address is returned.</returns>
        UInt64 AddressToModule(Process process, UInt64 address, out String moduleName, EmulatorType emulatorType = EmulatorType.None);

        /// <summary>
        /// Determines the base address of a module given a module name.
        /// </summary>
        /// <param name="process">The target process.</param>
        /// <param name="identifier">The module identifier, or name.</param>
        /// <returns>The base address of the module.</returns>
        UInt64 ResolveModule(Process process, String identifier, EmulatorType emulatorType = EmulatorType.None);
    }
    //// End interface
}
//// End namespace