namespace Squalr.Engine.Scanning.Scanners
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Extensions;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Diagnostics;
    using System.Linq;
    using System.Threading;
    using System.Threading.Tasks;
    using static Squalr.Engine.Common.TrackableTask;

    /// <summary>
    /// Collect values for a given snapshot. The values are assigned to a new snapshot.
    /// </summary>
    public static class ValueCollector
    {
        /// <summary>
        /// The name of this scan.
        /// </summary>
        private const String Name = "Value Collector";

        public static TrackableTask<Snapshot> CollectValues(Process process, Snapshot snapshot, String taskIdentifier = null, bool withLogging = true)
        {
            try
            {
                return TrackableTask<Snapshot>
                    .Create(ValueCollector.Name, taskIdentifier, out UpdateProgress updateProgress, out CancellationToken cancellationToken)
                    .With(Task<Snapshot>.Run(
                    () =>
                    {
                        try
                        {
                            Int32 processedRegions = 0;

                            if (withLogging)
                            {
                                Logger.Log(LogLevel.Info, "Reading values from memory...");
                            }

                            Stopwatch stopwatch = new Stopwatch();
                            stopwatch.Start();

                            ParallelOptions options = ParallelSettings.ParallelSettingsFastest;
                            options.CancellationToken = cancellationToken;

                            // Read memory to get current values for each region
                            Parallel.ForEach(
                                snapshot?.ReadOptimizedSnapshotRegions,
                                options,
                                (snapshotRegion) =>
                                {
                                    // Check for canceled scan
                                    cancellationToken.ThrowIfCancellationRequested();

                                    // Read the memory for this region
                                    snapshotRegion.ReadAllMemory(process);

                                    // Update progress every N regions
                                    if (Interlocked.Increment(ref processedRegions) % 32 == 0)
                                    {
                                        // Technically this callback is a data race, but it does not really matter if scan progress is not reported perfectly accurately
                                        updateProgress((float)processedRegions / (float)snapshot.RegionCount * 100.0f);
                                    }
                                });

                            cancellationToken.ThrowIfCancellationRequested();
                            UInt64 byteCount = snapshot?.SnapshotRegions?.Sum(snapshotRegion => unchecked((UInt64)snapshotRegion.RegionSize)) ?? 0;
                            stopwatch.Stop();

                            if (withLogging)
                            {
                                Logger.Log(LogLevel.Info, "Values collected in: " + stopwatch.Elapsed);
                                Logger.Log(LogLevel.Info, Conversions.ValueToMetricSize(byteCount) + " bytes read");
                            }

                            return snapshot;
                        }
                        catch (OperationCanceledException ex)
                        {
                            if (withLogging)
                            {
                                Logger.Log(LogLevel.Warn, "Scan canceled", ex);
                            }

                            return null;
                        }
                        catch (Exception ex)
                        {
                            if (withLogging)
                            {
                                Logger.Log(LogLevel.Error, "Error performing scan", ex);
                            }

                            return null;
                        }
                    },
                    cancellationToken));
            }
            catch (TaskConflictException ex)
            {
                if (withLogging)
                {
                    Logger.Log(LogLevel.Warn, "Unable to start scan. Scan is already queued.");
                }

                throw ex;
            }
        }
    }
    //// End class
}
//// End namespace