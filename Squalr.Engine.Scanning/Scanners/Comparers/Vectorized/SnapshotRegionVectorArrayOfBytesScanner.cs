namespace Squalr.Engine.Scanning.Scanners.Comparers.Vectorized
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Hardware;
    using Squalr.Engine.Scanning.Scanners.Constraints;
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using System.Numerics;

    /// <summary>
    /// A faster version of SnapshotElementComparer that takes advantage of vectorization/SSE instructions.
    /// </summary>
    internal unsafe class SnapshotRegionVectorArrayOfBytesScanner : SnapshotRegionVectorScannerBase
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="SnapshotRegionVectorArrayOfBytesScanner" /> class.
        /// </summary>
        /// <param name="region">The parent region that contains this element.</param>
        /// <param name="constraints">The set of constraints to use for the element comparisons.</param>
        public SnapshotRegionVectorArrayOfBytesScanner() : base()
        {
        }

        /// <summary>
        /// Gets the current values at the current vector read index.
        /// </summary>
        public Vector<Byte> CurrentValuesArrayOfBytes
        {
            get
            {
                return new Vector<Byte>(this.ElementRnage.ParentRegion.CurrentValues, unchecked((Int32)(this.VectorReadBase + this.VectorReadOffset + this.ArrayOfBytesChunkIndex * Vectors.VectorSize)));
            }
        }

        /// <summary>
        /// Gets the previous values at the current vector read index.
        /// </summary>
        public Vector<Byte> PreviousValuesArrayOfBytes
        {
            get
            {
                return new Vector<Byte>(this.ElementRnage.ParentRegion.PreviousValues, unchecked((Int32)(this.VectorReadBase + this.VectorReadOffset + this.ArrayOfBytesChunkIndex * Vectors.VectorSize)));
            }
        }

        /// <summary>
        /// Gets or sets the iterator index for array of bytes vectorized chunks.
        /// </summary>
        protected Int32 ArrayOfBytesChunkIndex { get; set; }

        /// <summary>
        /// Gets or sets a function which determines if this element has changed.
        /// </summary>
        private Func<Vector<Byte>> Changed { get; set; }

        /// <summary>
        /// Gets or sets a function which determines if this element has not changed.
        /// </summary>
        private Func<Vector<Byte>> Unchanged { get; set; }

        /// <summary>
        /// Gets or sets a function which determines if this array of bytes has a value equal to the given array of bytes.
        /// </summary>
        private Func<Object, Vector<Byte>, Vector<Byte>> EqualToValue { get; set; }

        /// <summary>
        /// Gets or sets a function which determines if this array of bytes has a value not equal to the given array of bytes.
        /// </summary>
        private Func<Object, Vector<Byte>, Vector<Byte>> NotEqualToValue { get; set; }

        /// <summary>
        /// Initializes this scanner for the given region and constaints.
        /// </summary>
        /// <param name="region">The parent region that contains this element.</param>
        /// <param name="constraints">The set of constraints to use for the element comparisons.</param>
        public override void Initialize(SnapshotElementRange region, ScanConstraints constraints)
        {
            base.Initialize(region, constraints);

            this.SetConstraintFunctions();
        }

        /// <summary>
        /// Performs a scan over the given element range, returning the elements that match the scan.
        /// </summary>
        /// <param name="elementRange">The element range to scan.</param>
        /// <param name="constraints">The scan constraints.</param>
        /// <returns>The resulting elements, if any.</returns>
        public override IList<SnapshotElementRange> ScanRegion(SnapshotElementRange elementRange, ScanConstraints constraints)
        {
            Int32 ByteArraySize = (this.DataType as ByteArrayType)?.Length ?? 0;
            Byte[] Mask = (this.DataType as ByteArrayType)?.Mask;

            if (ByteArraySize <= 0)
            {
                return new List<SnapshotElementRange>();
            }

            // Note that array of bytes must increment by 1 per iteration, unlike data type scans which can increment by vector size
            for (; this.VectorReadOffset <= this.ElementRnage.Range - ByteArraySize; this.VectorReadOffset++)
            {
                Vector<Byte> scanResults = this.VectorCompare();

                // Optimization: check all vector results true (vector of 0xFF's, which is how SSE/AVX instructions store true)
                if (Vector.GreaterThanAll(scanResults, Vector<Byte>.Zero))
                {
                    this.RunLengthEncoder.EncodeRange(Vectors.VectorSize);
                    continue;
                }
                // Optimization: check all vector results false
                else if (Vector.EqualsAll(scanResults, Vector<Byte>.Zero))
                {
                    this.RunLengthEncoder.FinalizeCurrentEncodeChecked(ByteArraySize);
                    continue;
                }
                // Otherwise the vector contains a mixture of true and false
                for (Int32 index = 0; index < Vectors.VectorSize; index += this.DataTypeSize)
                {
                    // Vector result was false
                    if (scanResults[unchecked(index)] == 0)
                    {
                        this.RunLengthEncoder.FinalizeCurrentEncodeChecked(this.DataTypeSize);
                    }
                    // Vector result was true
                    else
                    {
                        this.RunLengthEncoder.EncodeRange(this.DataTypeSize);
                    }
                }
            }

            this.RunLengthEncoder.FinalizeCurrentEncodeChecked();

            return this.RunLengthEncoder.GetCollectedRegions();
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
        private Func<Vector<Byte>> BuildCompareActions(IScanConstraint constraint)
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
                                if (Vector.EqualsAll(resultLeft, Vector<Byte>.Zero))
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
                                if (Vector.GreaterThanAll(resultLeft, Vector<Byte>.Zero))
                                {
                                    return Vector.OnesComplement(Vector<Byte>.Zero);
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
                            Int32 remainder = arrayOfBytes.Length % Vectors.VectorSize;
                            Int32 chunkCount = arrayOfBytes.Length / Vectors.VectorSize + (remainder > 0 ? 1 : 0);
                            Span<Byte> arrayOfByteSpan = new Span<Byte>(arrayOfBytes);
                            Span<Byte> maskSpan = new Span<Byte>(mask);
                            Vector<Byte>[] arrayOfByteChunks = new Vector<Byte>[chunkCount];
                            Vector<Byte>[] maskChunks = new Vector<Byte>[chunkCount];

                            for (Int32 chunkIndex = 0; chunkIndex < chunkCount; chunkIndex++)
                            {
                                Int32 currentChunkSize = remainder > 0 && chunkIndex == chunkCount - 1 ? remainder : Vectors.VectorSize;
                                Span<Byte> arrayOfBytesChunk = arrayOfByteSpan.Slice(Vectors.VectorSize * chunkIndex, currentChunkSize);
                                Span<Byte> maskChunk = maskSpan.Slice(Vectors.VectorSize * chunkIndex, currentChunkSize);

                                if (currentChunkSize != Vectors.VectorSize)
                                {
                                    Byte[] arrayOfBytesChunkPadded = Enumerable.Repeat<Byte>(0x00, Vectors.VectorSize).ToArray();
                                    Byte[] maskChunkPadded = Enumerable.Repeat<Byte>(0xFF, Vectors.VectorSize).ToArray();

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
                                    Vector<Byte> result = Vectors.AllBits;

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
                                    Vector<Byte> result = Vectors.AllBits;

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
