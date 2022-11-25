﻿namespace Squalr.Engine.Scanning.Scanners.Comparers.Vectorized
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Scanning.Scanners.Constraints;
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using System.Numerics;

    /// <summary>
    /// A faster version of SnapshotElementComparer that takes advantage of vectorization/SSE instructions.
    /// </summary>
    internal unsafe class SnapshotRegionVectorAoBScanner : SnapshotRegionScannerBase
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="SnapshotRegionVectorAoBScanner" /> class.
        /// </summary>
        /// <param name="region">The parent region that contains this element.</param>
        /// <param name="constraints">The set of constraints to use for the element comparisons.</param>
        public SnapshotRegionVectorAoBScanner(SnapshotRegion region, ScanConstraints constraints) : base(region, constraints)
        {
            this.SetConstraintFunctions();
            this.VectorCompare = this.BuildCompareActions(constraints?.RootConstraint);
        }

        /// <summary>
        /// Gets an action based on the element iterator scan constraint.
        /// </summary>
        private Func<Vector<Byte>> VectorCompare { get; set; }

        /// <summary>
        /// Gets a function which determines if this element has changed.
        /// </summary>
        private Func<Vector<Byte>> Changed { get; set; }

        /// <summary>
        /// Gets a function which determines if this element has not changed.
        /// </summary>
        private Func<Vector<Byte>> Unchanged { get; set; }

        /// <summary>
        /// Gets a function which determines if this array of bytes has a value equal to the given array of bytes.
        /// </summary>
        private Func<Object, Vector<Byte>, Vector<Byte>> EqualToValue { get; set; }

        /// <summary>
        /// Gets a function which determines if this array of bytes has a value not equal to the given array of bytes.
        /// </summary>
        private Func<Object, Vector<Byte>, Vector<Byte>> NotEqualToValue { get; set; }

        /// <summary>
        /// Performs a scan over the given region, returning the discovered regions.
        /// </summary>
        /// <param name="region">The region to scan.</param>
        /// <param name="constraints">The scan constraints.</param>
        /// <returns>The resulting regions, if any.</returns>
        public override IList<SnapshotRegion> ScanRegion(SnapshotRegion region, ScanConstraints constraints)
        {
            /*
            Int32 ByteArraySize = (this.DataType as ByteArrayType)?.Length ?? 0;
            Byte[] Mask = (this.DataType as ByteArrayType)?.Mask;

            if (ByteArraySize <= 0)
            {
                return new List<SnapshotRegion>();
            }

            // Note that array of bytes must increment by 1 per iteration, unlike data type scans which can increment by vector size
            for (; this.VectorReadOffset <= this.Region.RegionSize - ByteArraySize; this.VectorReadOffset++)
            {
                Vector<Byte> scanResults = this.VectorCompare();

                // Optimization: check all vector results true (vector of 0xFF's, which is how SSE/AVX instructions store true)
                if (Vector.GreaterThanAll(scanResults, Vector<Byte>.Zero))
                {
                    this.RunLength += this.VectorSize;
                    this.Encoding = true;
                    continue;
                }

                // Optimization: check all vector results false
                else if (Vector.EqualsAll(scanResults, Vector<Byte>.Zero))
                {
                    this.EncodeCurrentResults(0, ByteArraySize);
                    continue;
                }

                // Otherwise the vector contains a mixture of true and false
                for (Int32 index = 0; index < this.VectorSize; index += this.DataTypeSize)
                {
                    // Vector result was false
                    if (scanResults[unchecked(index)] == 0)
                    {
                        this.EncodeCurrentResults(index, ByteArraySize);
                    }
                    // Vector result was true
                    else
                    {
                        this.RunLength += this.DataTypeSize;
                        this.Encoding = true;
                    }
                }
            }

            return this.GatherCollectedRegions();
            */

            throw new NotImplementedException();
        }

        /// <summary>
        /// Initializes all constraint functions for value comparisons.
        /// </summary>
        private unsafe void SetConstraintFunctions()
        {
            this.Changed = () => new Vector<Byte>(Convert.ToByte(!Vector.EqualsAll(this.CurrentValuesArrayOfBytes, this.PreviousValuesArrayOfBytes)));
            this.Unchanged = () => new Vector<Byte>(Convert.ToByte(Vector.EqualsAll(this.CurrentValuesArrayOfBytes, this.PreviousValuesArrayOfBytes)));
            this.EqualToValue = (value, mask) => Vector.BitwiseOr(Vector.Equals(this.CurrentValuesArrayOfBytes, unchecked((Vector<Byte>)value)), mask);
            this.NotEqualToValue = (value, mask) => Vector.BitwiseOr(Vector.OnesComplement(Vector.Equals(this.CurrentValuesArrayOfBytes, unchecked((Vector<Byte>)value))), mask);
        }

        /// <summary>
        /// Sets the default compare action to use for this element.
        /// </summary>
        /// <param name="constraint">The constraint(s) to use for the scan.</param>
        /// <param name="compareActionValue">The value to use for the scan.</param>
        private Func<Vector<Byte>> BuildCompareActions(Constraint constraint)
        {
            switch (constraint)
            {
                case OperationConstraint operationConstraint:
                    if (operationConstraint.Left == null || operationConstraint.Right == null)
                    {
                        throw new ArgumentException("An operation constraint must have both a left and right child");
                    }

                    switch (operationConstraint.BinaryOperation)
                    {
                        case OperationConstraint.OperationType.AND:
                            return () =>
                            {
                                Vector<Byte> resultLeft = this.BuildCompareActions(operationConstraint.Left).Invoke();

                                // Early exit mechanism to prevent extra comparisons
                                if (resultLeft.Equals(Vector<Byte>.Zero))
                                {
                                    return Vector<Byte>.Zero;
                                }

                                Vector<Byte> resultRight = this.BuildCompareActions(operationConstraint.Right).Invoke();

                                return Vector.BitwiseAnd(resultLeft, resultRight);
                            };
                        case OperationConstraint.OperationType.OR:
                            return () =>
                            {
                                Vector<Byte> resultLeft = this.BuildCompareActions(operationConstraint.Left).Invoke();

                                // Early exit mechanism to prevent extra comparisons
                                if (resultLeft.Equals(Vector<Byte>.One))
                                {
                                    return Vector<Byte>.One;
                                }

                                Vector<Byte> resultRight = this.BuildCompareActions(operationConstraint.Right).Invoke();

                                return Vector.BitwiseOr(resultLeft, resultRight);
                            };
                        case OperationConstraint.OperationType.XOR:
                            return () =>
                            {
                                Vector<Byte> resultLeft = this.BuildCompareActions(operationConstraint.Left).Invoke();
                                Vector<Byte> resultRight = this.BuildCompareActions(operationConstraint.Right).Invoke();

                                return Vector.Xor(resultLeft, resultRight);
                            };
                        default:
                            throw new ArgumentException("Unkown operation type");
                    }
                case ScanConstraint scanConstraint:
                    /*
                     * Array of bytes scan works as such:
                     * Chunk the array of bytes and mask (these should be == size) into hardware vector sized byte arrays.
                     * 
                     * Iterate over all chunks, comparing these to the corresponding values being scanned.
                     *   - Vector AND all of the results together for detecting equal/not equal. Early exit if any chunk fails.
                     *   - Vector OR all of the results together for detecting changed/unchanged
                    */
            ByteArrayType byteArrayType = this.DataType as ByteArrayType;
                    Byte[] arrayOfBytes = scanConstraint?.ConstraintValue as Byte[];
                    Byte[] mask = scanConstraint?.ConstraintArgs as Byte[];

                    if (arrayOfBytes == null || mask == null || arrayOfBytes.Length != mask.Length)
                    {
                        throw new ArgumentException("Array of bytes and mask must be provided with all array of byte scans. These should be equal in length.");
                    }

                    switch (scanConstraint.Constraint)
                    {
                        case ScanConstraint.ConstraintType.Unchanged:
                        case ScanConstraint.ConstraintType.Changed:
                            if (scanConstraint.Constraint == ScanConstraint.ConstraintType.Unchanged)
                            {
                                return this.Unchanged;
                            }
                            else
                            {
                                return this.Changed;
                            }
                        case ScanConstraint.ConstraintType.Equal:
                        case ScanConstraint.ConstraintType.NotEqual:
                            Int32 remainder = arrayOfBytes.Length % this.VectorSize;
                            Int32 chunkCount = arrayOfBytes.Length / this.VectorSize + (remainder > 0 ? 1 : 0);
                            Span<Byte> arrayOfByteSpan = new Span<Byte>(arrayOfBytes);
                            Span<Byte> maskSpan = new Span<Byte>(mask);
                            Vector<Byte>[] arrayOfByteChunks = new Vector<Byte>[chunkCount];
                            Vector<Byte>[] maskChunks = new Vector<Byte>[chunkCount];

                            for (Int32 chunkIndex = 0; chunkIndex < chunkCount; chunkIndex++)
                            {
                                Int32 currentChunkSize = remainder > 0 && chunkIndex == chunkCount - 1 ? remainder : this.VectorSize;
                                Span<Byte> arrayOfBytesChunk = arrayOfByteSpan.Slice(this.VectorSize * chunkIndex, currentChunkSize);
                                Span<Byte> maskChunk = maskSpan.Slice(this.VectorSize * chunkIndex, currentChunkSize);

                                if (currentChunkSize != this.VectorSize)
                                {
                                    Byte[] arrayOfBytesChunkPadded = Enumerable.Repeat<Byte>(0x00, this.VectorSize).ToArray();
                                    Byte[] maskChunkPadded = Enumerable.Repeat<Byte>(0xFF, this.VectorSize).ToArray();

                                    arrayOfBytesChunk.CopyTo(arrayOfBytesChunkPadded);
                                    maskChunk.CopyTo(maskChunkPadded);

                                    arrayOfByteChunks[chunkIndex] = new Vector<Byte>(arrayOfBytesChunkPadded);
                                    maskChunks[chunkIndex] = new Vector<Byte>(maskChunkPadded);
                                }
                                else
                                {
                                    arrayOfByteChunks[chunkIndex] = new Vector<Byte>(arrayOfBytesChunk);
                                    maskChunks[chunkIndex] = new Vector<Byte>(maskChunk);
                                }
                            }

                            if (scanConstraint.Constraint == ScanConstraint.ConstraintType.Equal)
                            {
                                return () =>
                                {
                                    Vector<Byte> result = Vector<Byte>.One;

                                    for (this.ArrayOfBytesChunkIndex = 0; this.ArrayOfBytesChunkIndex < chunkCount; this.ArrayOfBytesChunkIndex++)
                                    {
                                        result = Vector.BitwiseAnd(result, this.EqualToValue(arrayOfByteChunks[this.ArrayOfBytesChunkIndex], maskChunks[this.ArrayOfBytesChunkIndex]));

                                        if (Vector.EqualsAll(result, Vector<Byte>.Zero))
                                        {
                                            break;
                                        }
                                    }

                                    return result;
                                };
                            }
                            else
                            {
                                return () =>
                                {
                                    Vector<Byte> result = Vector<Byte>.One;

                                    for (this.ArrayOfBytesChunkIndex = 0; this.ArrayOfBytesChunkIndex < chunkCount; this.ArrayOfBytesChunkIndex++)
                                    {
                                        result = Vector.BitwiseAnd(result, this.NotEqualToValue(arrayOfByteChunks[this.ArrayOfBytesChunkIndex], maskChunks[this.ArrayOfBytesChunkIndex]));

                                        if (Vector.EqualsAll(result, Vector<Byte>.Zero))
                                        {
                                            break;
                                        }
                                    }

                                    return result;
                                };
                            }
                        default:
                            throw new Exception("Unsupported constraint type");
                    }
                default:
                    throw new ArgumentException("Invalid constraint");
            }
        }
    }
    //// End class
}
//// End namespace