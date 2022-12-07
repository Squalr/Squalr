namespace Squalr.Engine.Scanning.Scanners.Comparers
{
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Collections.Generic;
    using System.Runtime.CompilerServices;

    /// <summary>
    /// A class for producing snapshot regions from a scanned snapshot region via run length encoded scan matches.
    /// This is one of the magic tricks Squalr uses for fast scans. Each scan thread uses run length encoding to track the number of consecutive
    /// successful scan matches. Once a non-matching element is found, a snapshot region is created containing the contiguous block of successful results.
    /// </summary>
    internal unsafe class SnapshotRegionRunLengthEncoder : IDisposable
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="SnapshotRegionRunLengthEncoder" /> class.
        /// </summary>
        public SnapshotRegionRunLengthEncoder()
        {
        }

        /// <summary>
        /// Gets or sets the current base address offset from which the run length encoding has started.
        /// </summary>
        private Int32 RunLengthEncodeOffset { get; set; }

        /// <summary>
        /// Gets or sets a value indicating whether we are currently encoding a new result region.
        /// </summary>
        private Boolean IsEncoding { get; set; }

        /// <summary>
        /// Gets or sets the current run length for run length encoded current scan results.
        /// </summary>
        private Int32 RunLength { get; set; }

        /// <summary>
        /// Gets or sets the parent snapshot region.
        /// </summary>
        private SnapshotElementRange ElementRange { get; set; }

        /// <summary>
        /// Gets or sets the list of discovered result regions.
        /// </summary>
        private IList<SnapshotElementRange> ResultRegions { get; set; }

        /// <summary>
        /// Initializes this run legnth encoder for a given region being scanned and a set of scan constraints.
        /// </summary>
        /// <param name="region">The parent region that contains this element.</param>
        public void Initialize(SnapshotElementRange region)
        {
            this.ElementRange = region;
            this.ResultRegions = new List<SnapshotElementRange>();
            this.RunLengthEncodeOffset = region?.RegionOffset ?? 0;
        }

        /// <summary>
        /// Perform cleanup and release references, this run length encoder may still be referenced by cached scanner instances.
        /// </summary>
        public void Dispose()
        {
            this.ElementRange = null;
            this.ResultRegions = null;
        }

        /// <summary>
        /// Finalizes any leftover snapshot regions and returns them.
        /// </summary>
        public IList<SnapshotElementRange> GetCollectedRegions()
        {
            return this.ResultRegions;
        }

        public void AdjustForMisalignment(Int32 misalignmentOffset)
        {
            this.RunLengthEncodeOffset -= misalignmentOffset;
        }

        /// <summary>
        /// Increases the run length encode by the provided byte count. If not currently encoding a region, encoding will begin.
        /// </summary>
        /// <param name="advanceByteCount">The byte count by which the run length encode is incremented.</param>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void EncodeRange(Int32 advanceByteCount)
        {
            this.RunLength += advanceByteCount;
            this.IsEncoding = true;
        }

        /// <summary>
        /// Encodes the current scan results if possible. This finalizes the current run-length encoded scan results to a snapshot region.
        /// This check performs bounds checking to ensure that no run-length encoding captured extra bytes outside of the scan range.
        /// </summary>
        /// <param name="advanceByteCount">The number of failed bytes (ie values that did not match scans) to increment by.</param>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void FinalizeCurrentEncodeChecked(Int32 advanceByteCount = 0)
        {
            // Create the final region if we are still encoding
            if (this.IsEncoding)
            {
                // Run length is in bytes, but snapshot regions need to know total number of elements, which depends on the data type and alignment
                UInt64 absoluteAddressStart = this.ElementRange.ParentRegion.BaseAddress + (UInt64)this.RunLengthEncodeOffset;
                UInt64 absoluteAddressEnd = absoluteAddressStart + (UInt64)this.RunLength;

                // Vector comparisons can produce some false positives since vectors can load values outside of the original snapshot range. This can result in next scans actually increasing the result count.
                // This is particularly true in "next scans". This check catches any potential errors introduced this way.
                if (absoluteAddressStart >= this.ElementRange.BaseElementAddress && absoluteAddressEnd <= this.ElementRange.EndElementAddress)
                {
                    this.ResultRegions.Add(new SnapshotElementRange(this.ElementRange.ParentRegion, this.RunLengthEncodeOffset, this.RunLength));
                }

                this.RunLengthEncodeOffset += this.RunLength;
                this.RunLength = 0;
                this.IsEncoding = false;
            }

            this.RunLengthEncodeOffset += advanceByteCount;
        }

        /// <summary>
        /// Encodes the current scan results if possible. This finalizes the current run-length encoded scan results to a snapshot region.
        /// </summary>
        /// <param name="advanceByteCount">The number of failed bytes (ie values that did not match scans) to increment by.</param>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void FinalizeCurrentEncodeUnchecked(Int32 advanceByteCount = 0)
        {
            // Create the final region if we are still encoding
            if (this.IsEncoding)
            {
                this.ResultRegions.Add(new SnapshotElementRange(this.ElementRange.ParentRegion, this.RunLengthEncodeOffset, this.RunLength));
                this.RunLengthEncodeOffset += this.RunLength;
                this.RunLength = 0;
                this.IsEncoding = false;
            }

            this.RunLengthEncodeOffset += advanceByteCount;
        }
    }
    //// End class
}
//// End namespace
