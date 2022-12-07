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
    /// A vector scanner implementation that can handle staggered vector scans (alignment less than data type size).
    /// </summary>
    internal unsafe class SnapshotRegionVectorStaggeredScanner : SnapshotRegionVectorScannerBase
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="SnapshotRegionVectorStaggeredScanner" /> class.
        /// </summary>
        /// <param name="region">The parent region that contains this element.</param>
        /// <param name="constraints">The set of constraints to use for the element comparisons.</param>
        public SnapshotRegionVectorStaggeredScanner() : base()
        {
        }

        /// <summary>
        /// Gets a dictionary of staggered mask maps. This only needs to store staggered masks for cases where alignment size is less than the data type size.
        /// </summary>
        private static readonly Dictionary<Int32, Dictionary<MemoryAlignment, Vector<Byte>[]>> StaggeredMaskMap = new Dictionary<Int32, Dictionary<MemoryAlignment, Vector<Byte>[]>>
        {
            //// Data type size 2
            {
                2, new Dictionary<MemoryAlignment, Vector<Byte>[]>
                {
                    //// 1-byte aligned
                    {
                        MemoryAlignment.Alignment1, new Vector<Byte>[2]
                        {
                            Vector.AsVectorByte(new Vector<UInt16>(0x00FF)),
                            Vector.AsVectorByte(new Vector<UInt16>(0xFF00)),
                        }
                    },
                }
            },
            //// Data type size 4
            {
                4, new Dictionary<MemoryAlignment, Vector<Byte>[]>
                {
                    //// 1-byte aligned
                    {
                        MemoryAlignment.Alignment1, new Vector<Byte>[4]
                        {
                            Vector.AsVectorByte(new Vector<UInt32>(0x000000FF)),
                            Vector.AsVectorByte(new Vector<UInt32>(0x0000FF00)),
                            Vector.AsVectorByte(new Vector<UInt32>(0x00FF0000)),
                            Vector.AsVectorByte(new Vector<UInt32>(0xFF000000)),
                        }
                    },
                    //// 2-byte aligned
                    {
                        MemoryAlignment.Alignment2, new Vector<Byte>[2]
                        {
                            Vector.AsVectorByte(new Vector<UInt32>(0x0000FFFF)),
                            Vector.AsVectorByte(new Vector<UInt32>(0xFFFF0000)),
                        }
                    },
                }
            },
            //// Data type size 8
            {
                8, new Dictionary<MemoryAlignment, Vector<Byte>[]>
                {
                    //// 1-byte aligned
                    {
                        MemoryAlignment.Alignment1, new Vector<Byte>[8]
                        {
                            Vector.AsVectorByte(new Vector<UInt64>(0x00000000000000FF)),
                            Vector.AsVectorByte(new Vector<UInt64>(0x000000000000FF00)),
                            Vector.AsVectorByte(new Vector<UInt64>(0x0000000000FF0000)),
                            Vector.AsVectorByte(new Vector<UInt64>(0x00000000FF000000)),
                            Vector.AsVectorByte(new Vector<UInt64>(0x000000FF00000000)),
                            Vector.AsVectorByte(new Vector<UInt64>(0x0000FF0000000000)),
                            Vector.AsVectorByte(new Vector<UInt64>(0x00FF000000000000)),
                            Vector.AsVectorByte(new Vector<UInt64>(0xFF00000000000000)),
                        }
                    },
                    //// 2-byte aligned
                    {
                        MemoryAlignment.Alignment2, new Vector<Byte>[4]
                        {
                            Vector.AsVectorByte(new Vector<UInt64>(0x000000000000FFFF)),
                            Vector.AsVectorByte(new Vector<UInt64>(0x00000000FFFF0000)),
                            Vector.AsVectorByte(new Vector<UInt64>(0x0000FFFF00000000)),
                            Vector.AsVectorByte(new Vector<UInt64>(0xFFFF000000000000)),
                        }
                    },
                    //// 4-byte aligned
                    {
                        MemoryAlignment.Alignment4, new Vector<Byte>[2]
                        {
                            Vector.AsVectorByte(new Vector<UInt64>(0x00000000FFFFFFFF)),
                            Vector.AsVectorByte(new Vector<UInt64>(0xFFFFFFFF00000000)),
                        }
                    },
                }
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

            // This algorithm is mostly the same as SnapshotRegionVectorFastScanner. The primary difference is that instead of doing one vector comparison, multiple scans must be done per vector to
            // scan for mis-aligned values. This adds 2 to 8 additional vector scans, based on the alignment and data type. Each of these sub-scans is masked against a stagger mask to create the scan result.
            // For example, scanning for 4-byte integer 0 with an alignment of 2-bytes against <0, 0, 0, 0, 55, 0, 0, 0, 0, 0> would need to return <255, 0, 0, 0, 255, 255, ..>.
            // This is accomplished by performing a full vector scan, then masking it against the appropriate stagger mask to extract the relevant scan results for that iteration.
            // These sub-scans are OR'd together to get a run-length encoded vector of all scan matches.

            Int32 scanCount = (this.ElementRnage.Range / Vectors.VectorSize) + (this.VectorOverread > 0 ? 1 : 0);
            Int32 scanCountPerVector = unchecked(this.DataTypeSize / (Int32)this.Alignment);
            Int32 offsetVectorIncrementSize = unchecked(Vectors.VectorSize - ((Int32)this.Alignment * scanCountPerVector));
            Vector<Byte> misalignmentMask = this.BuildVectorMisalignmentMask();
            Vector<Byte> overreadMask = this.BuildVectorOverreadMask();
            Vector<Byte>[] staggeredMasks = SnapshotRegionVectorStaggeredScanner.StaggeredMaskMap[this.DataTypeSize][this.Alignment];
            Vector<Byte> scanResults;

            // Perform the first scan (there should always be at least one). Apply the misalignment mask, and optionally the overread mask if this is also the finals scan.
            {
                scanResults = Vector.BitwiseAnd(Vector.BitwiseAnd(misalignmentMask, this.StaggeredVectorScan(ref staggeredMasks)), scanCount == 1 ? overreadMask : Vectors.AllBits);
                this.EncodeScanResults(ref scanResults);
                this.VectorReadOffset += offsetVectorIncrementSize;
            }

            // Perform middle scans
            for (; this.VectorReadOffset < this.ElementRnage.Range - Vectors.VectorSize; this.VectorReadOffset += offsetVectorIncrementSize)
            {
                scanResults = this.StaggeredVectorScan(ref staggeredMasks);
                this.EncodeScanResults(ref scanResults);
            }

            // Perform final scan, applying the overread mask if applicable.
            if (scanCount > 1)
            {
                scanResults = Vector.BitwiseAnd(overreadMask, this.StaggeredVectorScan(ref staggeredMasks));
                this.EncodeScanResults(ref scanResults);
                this.VectorReadOffset += offsetVectorIncrementSize;
            }

            this.RunLengthEncoder.FinalizeCurrentEncodeUnchecked();

            return this.RunLengthEncoder.GetCollectedRegions();
        }

        /// <summary>
        /// Performs a staggered scan of a given vector.
        /// </summary>
        /// <returns>The staggered scan results vector.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        Vector<Byte> StaggeredVectorScan(ref Vector<Byte>[] staggeredMasks)
        {
            Int32 scanCountPerVector = unchecked(this.DataTypeSize / (Int32)this.Alignment);
            Vector<Byte> scanResults = Vector<Byte>.Zero;

            for (Int32 alignmentOffset = 0; alignmentOffset < scanCountPerVector; alignmentOffset++)
            {
                scanResults = Vector.BitwiseOr(scanResults, Vector.BitwiseAnd(this.VectorCompare(), staggeredMasks[alignmentOffset]));

                this.VectorReadOffset += unchecked((Int32)this.Alignment);

                // Advance to the expected vector read offset if we run out of bytes
                if (this.VectorReadOffset >= this.ElementRnage.Range - Vectors.VectorSize)
                {
                    this.VectorReadOffset += unchecked((Int32)this.Alignment * (scanCountPerVector - alignmentOffset - 1));
                    break;
                }
            }

            return scanResults;
        }
    }
    //// End class
}
//// End namespace
