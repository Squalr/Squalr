namespace Squalr.Engine.Scanning.Scanners.Comparers.Vectorized
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Hardware;
    using Squalr.Engine.Scanning.Scanners.Constraints;
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Collections.Generic;
    using System.Numerics;
    using System.Runtime.CompilerServices;

    /// <summary>
    /// A vector scanner implementation that can handle sparse vector scans (alignment greater than data type size).
    /// </summary>
    internal unsafe class SnapshotRegionVectorSparseScanner : SnapshotRegionVectorScannerBase
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="SnapshotRegionVectorFastScanner" /> class.
        /// </summary>
        /// <param name="region">The parent region that contains this element.</param>
        /// <param name="constraints">The set of constraints to use for the element comparisons.</param>
        public SnapshotRegionVectorSparseScanner() : base()
        {
        }

        /// <summary>
        /// Gets a dictionary of sparse masks by scan alignment. This is used for scans where alignment is greater than the data type size.
        /// </summary>
        private static readonly Dictionary<MemoryAlignment, Vector<Byte>> SparseMasks = new Dictionary<MemoryAlignment, Vector<Byte>>
        {
            // 2-byte aligned
            {
                // This will produce a byte pattern of <0x00, 0xFF...> once reinterpreted as a byte vector.
                MemoryAlignment.Alignment2, Vector.AsVectorByte(new Vector<UInt16>(0xFF00))
            },
            // 4-byte aligned
            {
                // This will produce a byte pattern of <0x00, 0x00, 0x00, 0xFF...> once reinterpreted as a byte vector.
                MemoryAlignment.Alignment4, Vector.AsVectorByte(new Vector<UInt32>(0xFF000000))
            },
            // 8-byte aligned
            {
                // This will produce a byte pattern of <0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF...> once reinterpreted as a byte vector.
                MemoryAlignment.Alignment8, Vector.AsVectorByte(new Vector<UInt64>(0xFF00000000000000))
            },
        };

        /// <summary>
        /// Performs a scan over the given element range, returning the elements that match the scan.
        /// </summary>
        /// <param name="elementRange">The element range to scan.</param>
        /// <param name="constraints">The scan constraints.</param>
        /// <returns>The resulting elements, if any.</returns>
        public override IList<SnapshotElementRange> ScanRegion(SnapshotElementRange elementRange, ScanConstraints constraints)
        {
            this.Initialize(elementRange: elementRange, constraints: constraints);

            // This algorithm is mostly the same as SnapshotRegionVectorFastScanner. The only difference is that scans are compared against a sparse mask,
            // This mask automatically captures all in-between elements. For example, scanning for Byte 0 with an alignment of 2-bytes against <0, 24, 0, 43> would all return true, due to this mask of <0, 255, 0, 255>.
            // Scan results will automatically skip over the unwanted elements based on alignment. In fact, we do NOT want to break this into two separate snapshot regions, since this would be incredibly inefficient.
            // So in this example, we would return a single snapshot region of size 4, and the scan results would iterate by 2.

            Int32 scanCount = this.ElementRnage.Range / Vectors.VectorSize + (this.VectorOverread > 0 ? 1 : 0);
            Vector<Byte> misalignmentMask = this.BuildVectorMisalignmentMask();
            Vector<Byte> overreadMask = this.BuildVectorOverreadMask();
            Vector<Byte> sparseMask = SnapshotRegionVectorSparseScanner.SparseMasks[this.Alignment];
            Vector<Byte> scanResults;

            // Perform the first scan (there should always be at least one). Apply the misalignment mask, and optionally the overread mask if this is also the finals scan.
            {
                scanResults = Vector.BitwiseAnd(Vector.BitwiseAnd(misalignmentMask, this.VectorCompare()), scanCount == 1 ? overreadMask : Vectors.AllBits);
                this.EncodeScanResults(ref scanResults, ref sparseMask);
                this.VectorReadOffset += Vectors.VectorSize;
            }

            // Perform middle scans
            for (; this.VectorReadOffset < this.ElementRnage.Range - Vectors.VectorSize; this.VectorReadOffset += Vectors.VectorSize)
            {
                scanResults = this.VectorCompare();
                this.EncodeScanResults(ref scanResults, ref sparseMask);
            }

            // Perform final scan, applying the overread mask if applicable.
            if (scanCount > 1)
            {
                scanResults = Vector.BitwiseAnd(overreadMask, this.VectorCompare());
                this.EncodeScanResults(ref scanResults, ref sparseMask);
                this.VectorReadOffset += Vectors.VectorSize;
            }

            this.RunLengthEncoder.FinalizeCurrentEncodeUnchecked();

            return this.RunLengthEncoder.GetCollectedRegions();
        }

        /// <summary>
        /// Run-length encodes the given scan results into snapshot regions, accounting for sparse scan result mask.
        /// </summary>
        /// <param name="scanResults">The scan results to encode.</param>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        protected void EncodeScanResults(ref Vector<Byte> scanResults, ref Vector<Byte> sparseMask)
        {
            scanResults = Vector.BitwiseOr(scanResults, sparseMask);

            // Optimization: check all vector results true
            if (Vector.GreaterThanAll(scanResults, Vector<Byte>.Zero))
            {
                this.RunLengthEncoder.EncodeRange(Vectors.VectorSize);
            }
            // Optimization: check all vector results false (ie equal to the sparse mask)
            else if (Vector.EqualsAll(scanResults, sparseMask))
            {
                this.RunLengthEncoder.FinalizeCurrentEncodeUnchecked(Vectors.VectorSize);
            }
            else
            {
                // Otherwise the vector contains a mixture of true and false
                for (Int32 resultIndex = 0; resultIndex < Vectors.VectorSize; resultIndex += unchecked((Int32)this.Alignment))
                {
                    if (scanResults[resultIndex] != 0)
                    {
                        this.RunLengthEncoder.EncodeRange(unchecked((Int32)this.Alignment));
                    }
                    else
                    {
                        this.RunLengthEncoder.FinalizeCurrentEncodeUnchecked(unchecked((Int32)this.Alignment));
                    }
                }
            }
        }
    }
    //// End class
}
//// End namespace
