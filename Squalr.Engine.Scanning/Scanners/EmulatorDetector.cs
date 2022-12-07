namespace Squalr.Engine.Scanning.Scanners
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Logging;
    using System;
    using System.Diagnostics;
    using System.Threading;
    using System.Threading.Tasks;
    using static Squalr.Engine.Common.TrackableTask;

    /// <summary>
    /// Detects whether the target process is an emulator.
    /// </summary>
    public static class EmulatorDetector
    {
        /// <summary>
        /// The name of this scan.
        /// </summary>
        private const String Name = "Emulator Detector";

        /// <summary>
        /// Creates a task to detect whether the specified process is an emulator.
        /// </summary>
        /// <param name="process">The candidate emulator process.</param>
        /// <param name="taskIdentifier">An optional unique identifier for this task to prevent duplicate task creation.</param>
        /// <returns>A task to detect whether the specified process is an emulator.</returns>
        public static TrackableTask<EmulatorType> DetectEmulator(Process process, String taskIdentifier = null)
        {
            try
            {
                return TrackableTask<EmulatorType>
                    .Create(EmulatorDetector.Name, taskIdentifier, out UpdateProgress updateProgress, out CancellationToken cancellationToken)
                    .With(Task<EmulatorType>.Run(() =>
                    {
                        try
                        {
                            Stopwatch stopwatch = new Stopwatch();
                            stopwatch.Start();

                            EmulatorType detectedEmulator = EmulatorType.None;

                            // TODO: something a bit more accurate.
                            if (process?.MainWindowTitle?.StartsWith("Dolphin") ?? false)
                            {
                                detectedEmulator = EmulatorType.Dolphin;
                            }

                            // Exit if canceled
                            cancellationToken.ThrowIfCancellationRequested();
                            stopwatch.Stop();

                            switch(detectedEmulator)
                            {
                                case EmulatorType.Dolphin:
                                {
                                    Logger.Log(LogLevel.Info, "Dolphin Emulator detected. Scans will only scan GameCube/Wii memory. This can be disabled from emulator settings.");
                                    break;
                                }
                            }

                            return detectedEmulator;
                        }
                        catch (OperationCanceledException ex)
                        {
                            Logger.Log(LogLevel.Warn, "Emulator detector canceled. Target process assumed to not be an emulator.", ex);
                            return EmulatorType.None;
                        }
                        catch (Exception ex)
                        {
                            Logger.Log(LogLevel.Error, "Error detecting emulator. Target process assumed to not be an emulator.", ex);
                            return EmulatorType.None;
                        }
                    }, cancellationToken));
            }
            catch (TaskConflictException ex)
            {
                Logger.Log(LogLevel.Warn, "Unable to start emulator detection. This task is already queued.");
                throw ex;
            }
        }
    }
    //// End class
}
//// End namespace