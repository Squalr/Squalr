﻿namespace Squalr.Source.Scanners
{
    using Snapshots;
    using Squalr.Properties;
    using SqualrCore.Source.ActionScheduler;
    using SqualrCore.Source.Engine;
    using SqualrCore.Source.Utils.DataStructures;
    using SqualrCore.Source.Utils.Extensions;
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using System.Threading;
    using System.Threading.Tasks;

    /// <summary>
    /// Class to collect all pointers in the running process.
    /// </summary>
    internal class PointerCollector : ScheduledTask
    {
        /// <summary>
        /// Time in milliseconds between scans.
        /// </summary>
        private const Int32 RescanTime = 256;

        /// <summary>
        /// Limit the number of bytes to read per iteration.
        /// </summary>
        private const UInt64 ByteLimit = 2 << 20; // 2 MB

        /// <summary>
        /// The rounding size for pointer destinations.
        /// </summary>
        private const Int32 ChunkSize = 1024;

        /// <summary>
        /// Gets or sets the number of regions processed by this prefilter.
        /// </summary>
        private Int64 processedCount;

        /// <summary>
        /// Singleton instance of the <see cref="PointerCollector"/> class.
        /// </summary>
        private static Lazy<PointerCollector> pointerCollectorInstance = new Lazy<PointerCollector>(
            () => { return new PointerCollector(); },
            LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// Prevents a default instance of the <see cref="PointerCollector" /> class from being created.
        /// </summary>
        private PointerCollector() : base("Pointer Collector", isRepeated: true, trackProgress: false)
        {
            this.AccessLock = new Object();
            this.FoundPointerDestinations = new ConcurrentHashSet<IntPtr>();
            this.InProgressPointerDestinations = new ConcurrentHashSet<IntPtr>();
        }

        /// <summary>
        /// Gets or sets the pointers found in the target process.
        /// </summary>
        private ConcurrentHashSet<IntPtr> FoundPointerDestinations { get; set; }

        /// <summary>
        /// Gets or sets the new found pointers being constructed, which will replace the found pointers upon snapshot parse completion.
        /// </summary>
        private ConcurrentHashSet<IntPtr> InProgressPointerDestinations { get; set; }

        /// <summary>
        /// Gets or sets the current snapshot being parsed. A new one is collected after the current one is parsed.
        /// </summary>
        private Snapshot CurrentSnapshot { get; set; }

        private Object AccessLock { get; set; }

        public static PointerCollector GetInstance()
        {
            return PointerCollector.pointerCollectorInstance.Value;
        }

        /// <summary>
        /// Gets all found pointers in the external process.
        /// </summary>
        /// <returns>A set of all found pointers.</returns>
        public IEnumerable<IntPtr> GetFoundPointerDestinations()
        {
            lock (this.AccessLock)
            {
                foreach (IntPtr pointerDestination in this.FoundPointerDestinations)
                {
                    yield return pointerDestination;
                }
            }
        }

        protected override void OnBegin()
        {
            this.UpdateInterval = PointerCollector.RescanTime;
        }

        /// <summary>
        /// Called when the scan updates.
        /// </summary>
        /// <param name="cancellationToken">The cancellation token for handling canceled tasks.</param>
        protected override void OnUpdate(CancellationToken cancellationToken)
        {
            this.GatherPointers();
        }

        /// <summary>
        /// Called when the repeated task completes.
        /// </summary>
        protected override void OnEnd()
        {
        }

        /// <summary>
        /// Gradually gathers pointers in the running process.
        /// </summary>
        private void GatherPointers()
        {
            ConcurrentHashSet<UInt64> foundPointerDestinations = new ConcurrentHashSet<UInt64>();

            // Test for conditions where we set the final found set and take a new snapshot to parse
            if (this.CurrentSnapshot == null || this.CurrentSnapshot.RegionCount <= 0 || this.processedCount >= this.CurrentSnapshot.RegionCount)
            {
                this.processedCount = 0;
                this.CurrentSnapshot = SnapshotManager.GetInstance().CreateSnapshotFromUsermodeMemory();

                lock (this.AccessLock)
                {
                    this.FoundPointerDestinations = this.InProgressPointerDestinations;
                }

                this.InProgressPointerDestinations = new ConcurrentHashSet<IntPtr>();
            }

            List<SnapshotRegion> sortedRegions = new List<SnapshotRegion>(this.CurrentSnapshot.GetSnapshotRegions().OrderBy(x => x.GetTimeSinceLastRead()));

            UInt64 total = 0;
            Int32 regionCount = sortedRegions.TakeWhile(x =>
            {
                UInt64 previousTotal = total;
                total += x.RegionSize;
                return previousTotal == 0 || total < PointerCollector.ByteLimit;
            }).Count();

            // Process the allowed amount of chunks from the priority queue
            Parallel.For(
                0,
                Math.Min(sortedRegions.Count, regionCount),
                SettingsViewModel.GetInstance().ParallelSettingsMedium,
                (index) =>
            {
                Interlocked.Increment(ref this.processedCount);

                SnapshotRegion region = sortedRegions[index];
                Boolean success;

                // Set to type of a pointer
                region.ElementType = EngineCore.GetInstance().Processes.IsOpenedProcess32Bit() ? typeof(UInt32) : typeof(UInt64);

                // Enforce 4-byte alignment of pointers
                region.Alignment = sizeof(Int32);

                // Read current page data for chunk
                region.ReadAllRegionMemory(keepValues: true, readSuccess: out success);

                // Read failed; Deallocated page
                if (!success)
                {
                    return;
                }

                if (region.CurrentValues == null || region.CurrentValues.Length <= 0)
                {
                    return;
                }

                if (EngineCore.GetInstance().Processes.IsOpenedProcess32Bit())
                {
                    for (IEnumerator<SnapshotElementIterator> enumerator = region.IterateElements(PointerIncrementMode.CurrentOnly); enumerator.MoveNext();)
                    {
                        SnapshotElementIterator element = enumerator.Current;
                        UInt32 value = unchecked((UInt32)element.GetCurrentValue());

                        // Enforce 4-byte alignment of destination, and filter out small (invalid) pointers
                        if (value < UInt16.MaxValue || value % 4 != 0)
                        {
                            continue;
                        }

                        // Check if it is possible that this pointer is valid, if so keep it
                        if (this.CurrentSnapshot.ContainsAddress(value))
                        {
                            value = value - value % PointerCollector.ChunkSize;
                            foundPointerDestinations.Add(value);
                        }
                    }
                }
                else
                {
                    for (IEnumerator<SnapshotElementIterator> enumerator = region.IterateElements(PointerIncrementMode.CurrentOnly); enumerator.MoveNext();)
                    {
                        SnapshotElementIterator element = enumerator.Current;
                        UInt64 value = unchecked((UInt64)element.GetCurrentValue());

                        // Enforce 4-byte alignment of destination, and filter out small (invalid) pointers
                        if (value < UInt16.MaxValue || value % 4 != 0)
                        {
                            continue;
                        }

                        // Check if it is possible that this pointer is valid, if so keep it
                        if (this.CurrentSnapshot.ContainsAddress(value))
                        {
                            value = value - value % PointerCollector.ChunkSize;
                            foundPointerDestinations.Add(value);
                        }
                    }
                }

                // Clear the saved values, we do not need them now
                region.SetCurrentValues(null);
            });

            lock (this.AccessLock)
            {
                foreach (UInt64 pointerDestination in foundPointerDestinations)
                {
                    IntPtr pointerDestinationPtr = pointerDestination.ToIntPtr();

                    this.InProgressPointerDestinations.Add(pointerDestinationPtr);
                    this.FoundPointerDestinations.Add(pointerDestinationPtr);
                }
            }
        }
    }
    //// End class
}
//// End namespace