namespace Squalr.Engine.Scanning.Scanners.Pointers
{
    using SharpDX;
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Scanning.Scanners.Constraints;
    using Squalr.Engine.Scanning.Scanners.Pointers.SearchKernels;
    using Squalr.Engine.Scanning.Scanners.Pointers.Structures;
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Collections.Generic;
    using System.Diagnostics;
    using System.Linq;
    using System.Threading;
    using System.Threading.Tasks;
    using static Squalr.Engine.Common.TrackableTask;

    /// <summary>
    /// Scans for pointers in the target process.
    /// </summary>
    public static class PointerRebase
    {
        /// <summary>
        /// The name of this scan.
        /// </summary>
        private const String Name = "Pointer Rescan";

        /// <summary>
        /// 
        /// </summary>
        /// <param name="process"></param>
        /// <param name="previousPointerBag"></param>
        /// <param name="readMemory"></param>
        /// <param name="taskIdentifier"></param>
        /// <returns>Atrackable task that returns the scan results.</returns>
        public static TrackableTask<PointerBag> Scan(Process process, PointerBag previousPointerBag, Boolean readMemory, String taskIdentifier = null)
        {
            try
            {
                TrackableTask<PointerBag> pointerScanTask = TrackableTask<PointerBag>.Create(PointerRebase.Name, taskIdentifier, out UpdateProgress updateProgress, out CancellationToken cancellationToken);

                return pointerScanTask.With(Task<PointerBag>.Run(
                    () =>
                    {
                        try
                        {
                            cancellationToken.ThrowIfCancellationRequested();

                            Stopwatch stopwatch = new Stopwatch();
                            stopwatch.Start();

                            const MemoryAlignment alignment = MemoryAlignment.Alignment4;

                            IList<Level> oldLevels = previousPointerBag.Levels;
                            IList<Level> newLevels = new List<Level>();

                            for (Int32 levelIndex = 0; levelIndex < oldLevels.Count; levelIndex++)
                            {
                                Snapshot updatedStaticPointers = oldLevels[levelIndex].StaticPointers;
                                Snapshot updatedHeapPointers = oldLevels[levelIndex].HeapPointers;

                                // Step 1) Re-read values of all pointers
                                if (readMemory)
                                {
                                    TrackableTask<Snapshot> staticValueCollector = ValueCollector.CollectValues(process, updatedStaticPointers);

                                    // Does not apply to target address
                                    if (levelIndex > 0)
                                    {
                                        TrackableTask<Snapshot> heapValueCollector = ValueCollector.CollectValues(process, updatedHeapPointers);
                                        updatedHeapPointers = heapValueCollector.Result;
                                    }

                                    updatedStaticPointers = staticValueCollector.Result;
                                    updatedStaticPointers.SetAlignmentCascading(previousPointerBag.PointerSize.ToSize(), alignment);
                                }

                                Stopwatch levelStopwatch = new Stopwatch();
                                levelStopwatch.Start();

                                // Step 2) Rebase new heap on to previous heap
                                if (levelIndex > 0)
                                {
                                    IVectorPointerSearchKernel heapSearchKernel = PointerSearchKernelFactory.GetSearchKernel(newLevels.Last().HeapPointers, previousPointerBag.MaxOffset, previousPointerBag.PointerSize);
                                    TrackableTask<Snapshot> heapFilterTask = PointerFilter.Filter(pointerScanTask, updatedHeapPointers, heapSearchKernel, previousPointerBag.PointerSize, newLevels.Last().HeapPointers, previousPointerBag.MaxOffset);

                                    updatedHeapPointers = heapFilterTask.Result;
                                    updatedHeapPointers.SetAlignmentCascading(previousPointerBag.PointerSize.ToSize(), alignment);
                                }

                                // Step 3) Filter static pointers that still point into the updated heap
                                IVectorPointerSearchKernel staticSearchKernel = PointerSearchKernelFactory.GetSearchKernel(updatedHeapPointers, previousPointerBag.MaxOffset, previousPointerBag.PointerSize);
                                TrackableTask<Snapshot> staticFilterTask = PointerFilter.Filter(pointerScanTask, updatedStaticPointers, staticSearchKernel, previousPointerBag.PointerSize, updatedHeapPointers, previousPointerBag.MaxOffset);

                                updatedStaticPointers = staticFilterTask.Result;
                                updatedStaticPointers.SetAlignmentCascading(previousPointerBag.PointerSize.ToSize(), alignment);

                                levelStopwatch.Stop();
                                Logger.Log(LogLevel.Info, "Pointer rebase from level " + levelIndex + " => " + (levelIndex + 1) + " completed in: " + levelStopwatch.Elapsed);

                                newLevels.Add(new Level(updatedHeapPointers, updatedStaticPointers));
                            }

                            // Exit if canceled
                            cancellationToken.ThrowIfCancellationRequested();

                            PointerBag pointerBag = new PointerBag(newLevels, previousPointerBag.MaxOffset, previousPointerBag.PointerSize);

                            stopwatch.Stop();
                            Logger.Log(LogLevel.Info, "Pointer rebase complete in: " + stopwatch.Elapsed);

                            return pointerBag;
                        }
                        catch (OperationCanceledException ex)
                        {
                            Logger.Log(LogLevel.Warn, "Pointer rebase canceled", ex);
                        }
                        catch (Exception ex)
                        {
                            Logger.Log(LogLevel.Error, "Error performing pointer rebase", ex);
                        }

                        return null;
                    },
                    cancellationToken));
            }
            catch (TaskConflictException ex)
            {
                Logger.Log(LogLevel.Warn, "A pointer scan is already scheduled.");
                throw ex;
            }
        }
    }
    //// End class
}
//// End namespace