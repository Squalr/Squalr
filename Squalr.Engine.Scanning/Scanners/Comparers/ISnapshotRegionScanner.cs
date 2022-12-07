namespace Squalr.Engine.Scanning.Scanners.Comparers
{
    using Squalr.Engine.Scanning.Scanners.Constraints;
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Collections.Generic;

    /// <summary>
    /// An interface that defines the implementation for snapshot scans.
    /// </summary>
    internal unsafe interface ISnapshotRegionScanner : IDisposable
    {
        /// <summary>
        /// Performs a scan over the given element range, returning the elements that match the scan.
        /// </summary>
        /// <param name="elementRange">The element range to scan.</param>
        /// <param name="constraints">The scan constraints.</param>
        /// <returns>The resulting elements, if any.</returns>
        public IList<SnapshotElementRange> ScanRegion(SnapshotElementRange elementRange, ScanConstraints constraints);

        /// <summary>
        /// Sets the action to perform when the scanner is disposed. Note that a disposed scanner is not necessarily destroyed, as these objects may be recycled for future scans.
        /// </summary>
        /// <param name="onDispose">The dispose function callback.</param>
        public void SetDisposeCallback(Action onDispose);
    }
    //// End interface
}
//// End namespace
