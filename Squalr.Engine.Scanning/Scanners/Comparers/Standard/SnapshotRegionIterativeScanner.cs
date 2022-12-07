namespace Squalr.Engine.Scanning.Scanners.Comparers.Standard
{
    using Squalr.Engine.Scanning.Scanners.Constraints;
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Collections.Generic;

    /// <summary>
    /// A scanner that works by looping over each element of the snapshot individually. Much slower than the vectorized version.
    /// </summary>
    internal class SnapshotRegionIterativeScanner : SnapshotRegionStandardScannerBase
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="SnapshotRegionIterativeScanner" /> class.
        /// </summary>
        /// <param name="region">The parent region that contains this element.</param>
        /// <param name="constraints">The constraints to use for the element comparisons.</param>
        public unsafe SnapshotRegionIterativeScanner() : base()
        {
        }

        /// <summary>
        /// Performs a scan over the given element range, returning the elements that match the scan.
        /// </summary>
        /// <param name="elementRange">The element range to scan.</param>
        /// <param name="constraints">The scan constraints.</param>
        /// <returns>The resulting elements, if any.</returns>
        public unsafe override IList<SnapshotElementRange> ScanRegion(SnapshotElementRange elementRange, ScanConstraints constraints)
        {
            this.Initialize(elementRange: elementRange, constraints: constraints);

            Int32 alignedElementCount = elementRange.GetAlignedElementCount(constraints.Alignment);

            for (Int32 index = 0; index < alignedElementCount; index++)
            {
                if (this.ElementCompare())
                {
                    this.RunLengthEncoder.EncodeRange((Int32)constraints.Alignment);
                }
                else
                {
                    this.RunLengthEncoder.FinalizeCurrentEncodeUnchecked((Int32)constraints.Alignment);
                }

                this.CurrentValuePointer += (Int32)constraints.Alignment;
                this.PreviousValuePointer += (Int32)constraints.Alignment;
            }

            this.RunLengthEncoder.FinalizeCurrentEncodeUnchecked();

            return this.RunLengthEncoder.GetCollectedRegions();
        }
    }
    //// End class
}
//// End namespace
