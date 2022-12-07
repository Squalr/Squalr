namespace Squalr.Engine.Scanning.Scanners.Comparers.Standard
{
    using Squalr.Engine.Scanning.Scanners.Constraints;
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Collections.Generic;

    /// <summary>
    /// A scanner that works by looping over each element of the snapshot individually. Much slower than the vectorized version.
    /// </summary>
    internal class SnapshotRegionSingleElementScanner : SnapshotRegionStandardScannerBase
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="SnapshotRegionSingleElementScanner" /> class.
        /// </summary>
        public unsafe SnapshotRegionSingleElementScanner() : base()
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
            this.InitializeNoPinning(region: elementRange, constraints: constraints);

            fixed (Byte* currentValuePtr = &elementRange.ParentRegion.CurrentValues[elementRange.RegionOffset])
            {
                if (elementRange.ParentRegion.PreviousValues != null && elementRange.ParentRegion.PreviousValues.Length > 0)
                {
                    fixed (Byte* previousValuePtr = &elementRange.ParentRegion.PreviousValues[elementRange.RegionOffset])
                    {
                        this.CurrentValuePointer = currentValuePtr;
                        this.PreviousValuePointer = previousValuePtr;

                        if (this.ElementCompare())
                        {
                            return new List<SnapshotElementRange>()
                            {
                                new SnapshotElementRange(elementRange.ParentRegion, elementRange.RegionOffset, elementRange.Range)
                            };
                        }
                    }
                }
                else
                {
                    this.CurrentValuePointer = currentValuePtr;

                    if (this.ElementCompare())
                    {
                        return new List<SnapshotElementRange>()
                        {
                            new SnapshotElementRange(elementRange.ParentRegion, elementRange.RegionOffset, elementRange.Range)
                        };
                    }
                }
            }

            return null;
        }
    }
    //// End class
}
//// End namespace
