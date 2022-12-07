namespace Squalr.Engine.Scanning.Scanners.Comparers.Vectorized
{
    using Squalr.Engine.Common.Hardware;
    using Squalr.Engine.Scanning.Scanners.Constraints;
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Collections.Generic;
    using System.Numerics;

    /// <summary>
    /// A fast vectorized snapshot region scanner that is optimized for snapshot regions that can be chunked to fit entirely in a hardware vector.
    /// This scan only works when the alignment size is equal to the data type size.
    /// </summary>
    internal unsafe class SnapshotRegionVectorFastScanner : SnapshotRegionVectorScannerBase
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="SnapshotRegionVectorFastScanner" /> class.
        /// </summary>
        /// <param name="region">The parent region that contains this element.</param>
        /// <param name="constraints">The set of constraints to use for the element comparisons.</param>
        public SnapshotRegionVectorFastScanner() : base()
        {
        }

        /// <summary>
        /// Performs a scan over the given element range, returning the elements that match the scan.
        /// </summary>
        /// <param name="elementRange">The element range to scan.</param>
        /// <param name="constraints">The scan constraints.</param>
        /// <returns>The resulting elements, if any.</returns>
        public override IList<SnapshotElementRange> ScanRegion(SnapshotElementRange elementRange, ScanConstraints constraints)
        {
            this.Initialize(elementRange: elementRange, constraints: constraints);

            // This algorithm has three stages:
            // 1) Scan the first vector of memory, which may contain elements we do not care about. ie <x, x, x, x ... y, y, y, y>,
            //      where x is data outside the element range (but within the snapshot region), and y is within the region we are scanning.
            //      to solve this, we mask out the x values such that these will always be considered false by our scan
            // 2) Scan the middle parts of. These will all fit perfectly into vectors
            // 3) Scan the final vector, if it exists. This may spill outside of the element range (but within the snapshot region).
            //      This works exactly like the first scan, but reversed. ie <y, y, y, y, ... x, x, x, x>, where x values are masked to be false.
            //      Note: This mask may also be applied to the first scan, if it is also the last scan (ie only 1 scan total for this region).

            Int32 scanCount = this.ElementRnage.Range / Vectors.VectorSize + (this.VectorOverread > 0 ? 1 : 0);
            Vector<Byte> misalignmentMask = this.BuildVectorMisalignmentMask();
            Vector<Byte> overreadMask = this.BuildVectorOverreadMask();
            Vector<Byte> scanResults;

            // Perform the first scan (there should always be at least one). Apply the misalignment mask, and optionally the overread mask if this is also the finals scan.
            {
                scanResults = Vector.BitwiseAnd(Vector.BitwiseAnd(misalignmentMask, this.VectorCompare()), scanCount == 1 ? overreadMask : Vectors.AllBits);
                this.EncodeScanResults(ref scanResults);
                this.VectorReadOffset += Vectors.VectorSize;
            }

            // Perform middle scans
            for (; this.VectorReadOffset < this.ElementRnage.Range - Vectors.VectorSize; this.VectorReadOffset += Vectors.VectorSize)
            {
                scanResults = this.VectorCompare();
                this.EncodeScanResults(ref scanResults);
            }

            // Perform final scan, applying the overread mask if applicable.
            if (scanCount > 1)
            {
                scanResults = Vector.BitwiseAnd(overreadMask, this.VectorCompare());
                this.EncodeScanResults(ref scanResults);
                this.VectorReadOffset += Vectors.VectorSize;
            }

            this.RunLengthEncoder.FinalizeCurrentEncodeUnchecked();

            return this.RunLengthEncoder.GetCollectedRegions();
        }
    }
    //// End class
}
//// End namespace
