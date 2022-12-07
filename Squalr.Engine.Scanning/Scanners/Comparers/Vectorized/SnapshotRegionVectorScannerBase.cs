namespace Squalr.Engine.Scanning.Scanners.Comparers.Vectorized
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Hardware;
    using Squalr.Engine.Scanning.Scanners.Constraints;
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Buffers.Binary;
    using System.Numerics;
    using System.Runtime.CompilerServices;

    /// <summary>
    /// A faster version of SnapshotElementComparer that takes advantage of vectorization/SSE instructions.
    /// </summary>
    internal unsafe abstract class SnapshotRegionVectorScannerBase : SnapshotRegionScannerBase
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="SnapshotRegionVectorScannerBase" /> class.
        /// </summary>
        /// <param name="region">The parent region that contains this element.</param>
        /// <param name="constraints">The set of constraints to use for the element comparisons.</param>
        public SnapshotRegionVectorScannerBase() : base()
        {
        }

        /// <summary>
        /// Gets the current values at the current vector read index.
        /// </summary>
        public Vector<Byte> CurrentValues
        {
            get
            {
                return new Vector<Byte>(this.ElementRnage.ParentRegion.CurrentValues, unchecked((Int32)(this.VectorReadBase + this.VectorReadOffset)));
            }
        }

        /// <summary>
        /// Gets the previous values at the current vector read index.
        /// </summary>
        public Vector<Byte> PreviousValues
        {
            get
            {
                return new Vector<Byte>(this.ElementRnage.ParentRegion.PreviousValues, unchecked((Int32)(this.VectorReadBase + this.VectorReadOffset)));
            }
        }

        /// <summary>
        /// Gets the current values at the current vector read index in big endian format.
        /// </summary>
        public Vector<Byte> CurrentValuesBigEndian16
        {
            get
            {
                Vector<Int16> result = Vector.AsVectorInt16(this.CurrentValues);
                Span<Int16> endianStorage = stackalloc Int16[Vectors.VectorSize / sizeof(Int16)];

                for (Int32 index = 0; index < Vectors.VectorSize / sizeof(Int16); index++)
                {
                    endianStorage[index] = BinaryPrimitives.ReverseEndianness(result[index]);
                }

                return Vector.AsVectorByte(new Vector<Int16>(endianStorage));
            }
        }

        /// <summary>
        /// Gets the previous values at the current vector read index in big endian format.
        /// </summary>
        public Vector<Byte> PreviousValuesBigEndian16
        {
            get
            {
                Vector<Int16> result = Vector.AsVectorInt16(this.PreviousValues);
                Span<Int16> endianStorage = stackalloc Int16[Vectors.VectorSize / sizeof(Int16)];

                for (Int32 index = 0; index < Vectors.VectorSize / sizeof(Int16); index++)
                {
                    endianStorage[index] = BinaryPrimitives.ReverseEndianness(result[index]);
                }

                return Vector.AsVectorByte(new Vector<Int16>(endianStorage));
            }
        }

        /// <summary>
        /// Gets the current values at the current vector read index in big endian format.
        /// </summary>
        public Vector<Byte> CurrentValuesBigEndian32
        {
            get
            {
                Vector<Int32> result = Vector.AsVectorInt32(this.CurrentValues);
                Span<Int32> endianStorage = stackalloc Int32[Vectors.VectorSize / sizeof(Int32)];

                for (Int32 index = 0; index < Vectors.VectorSize / sizeof(Int32); index++)
                {
                    endianStorage[index] = BinaryPrimitives.ReverseEndianness(result[index]);
                }

                return Vector.AsVectorByte(new Vector<Int32>(endianStorage));
            }
        }

        /// <summary>
        /// Gets the previous values at the current vector read index in big endian format.
        /// </summary>
        public Vector<Byte> PreviousValuesBigEndian32
        {
            get
            {
                Vector<Int32> result = Vector.AsVectorInt32(this.PreviousValues);
                Span<Int32> endianStorage = stackalloc Int32[Vectors.VectorSize / sizeof(Int32)];

                for (Int32 index = 0; index < Vectors.VectorSize / sizeof(Int32); index++)
                {
                    endianStorage[index] = BinaryPrimitives.ReverseEndianness(result[index]);
                }

                return Vector.AsVectorByte(new Vector<Int32>(endianStorage));
            }
        }

        /// <summary>
        /// Gets the current values at the current vector read index in big endian format.
        /// </summary>
        public Vector<Byte> CurrentValuesBigEndian64
        {
            get
            {
                Vector<Int64> result = Vector.AsVectorInt64(this.CurrentValues);
                Span<Int64> endianStorage = stackalloc Int64[Vectors.VectorSize / sizeof(Int64)];

                for (Int32 index = 0; index < Vectors.VectorSize / sizeof(Int64); index++)
                {
                    endianStorage[index] = BinaryPrimitives.ReverseEndianness(result[index]);
                }

                return Vector.AsVectorByte(new Vector<Int64>(endianStorage));
            }
        }

        /// <summary>
        /// Gets the previous values at the current vector read index in big endian format.
        /// </summary>
        public Vector<Byte> PreviousValuesBigEndian64
        {
            get
            {
                Vector<Int64> result = Vector.AsVectorInt64(this.PreviousValues);
                Span<Int64> endianStorage = stackalloc Int64[Vectors.VectorSize / sizeof(Int64)];

                for (Int32 index = 0; index < Vectors.VectorSize / sizeof(Int64); index++)
                {
                    endianStorage[index] = BinaryPrimitives.ReverseEndianness(result[index]);
                }

                return Vector.AsVectorByte(new Vector<Int64>(endianStorage));
            }
        }

        /// <summary>
        /// Gets or sets the index from which the next vector is read.
        /// </summary>
        public Int32 VectorReadOffset { get; protected set; }

        /// <summary>
        /// Gets or sets the base address from which vectors are read.
        /// </summary>
        protected Int32 VectorReadBase { get; set; }

        /// <summary>
        /// Gets the vector misalignment of the first vector in the region being scanned.
        /// </summary>
        protected Int32 VectorMisalignment { get; private set; }

        /// <summary>
        /// Gets the vector overread of the last vector in the region being scanned.
        /// </summary>
        protected Int32 VectorOverread { get; private set; }

        /// <summary>
        /// Gets or sets an action based on the element iterator scan constraint.
        /// </summary>
        protected Func<Vector<Byte>> VectorCompare { get; set; }

        /// <summary>
        /// Gets or sets an action based on the element iterator scan constraint.
        /// </summary>
        protected Func<Vector<Byte>> CustomVectorCompare { get; set; }

        /// <summary>
        /// Initializes this scanner for the given region and constaints.
        /// </summary>
        /// <param name="region">The parent region that contains this element.</param>
        /// <param name="constraints">The set of constraints to use for the element comparisons.</param>
        public override void Initialize(SnapshotElementRange elementRange, ScanConstraints constraints)
        {
            base.Initialize(elementRange: elementRange, constraints: constraints);

            this.VectorCompare = this.BuildCompareActions(constraints);
            this.VectorMisalignment = this.CalculateVectorMisalignment();
            this.VectorReadBase = elementRange.RegionOffset - this.VectorMisalignment;
            this.VectorOverread = this.CalculateVectorOverread();
            this.VectorReadOffset = 0;
            this.RunLengthEncoder.AdjustForMisalignment(this.VectorMisalignment);
        }

        /// <summary>
        /// Sets a custom comparison function to use in scanning.
        /// </summary>
        /// <param name="customCompare"></param>
        public void SetCustomCompareAction(Func<Vector<Byte>> customCompare)
        {
            this.CustomVectorCompare = customCompare;
        }

        /// <summary>
        /// Create a misalignment mask based on the current vector misalignment. The first N misaligned bytes will be set to 0, the rest 0xFF.
        /// </summary>
        /// <returns>A misalignment mask based on the current vector misalignment.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        protected Vector<Byte> BuildVectorMisalignmentMask()
        {
            Span<Byte> misalignmentMask = stackalloc Byte[Vectors.VectorSize];

            misalignmentMask.Slice(this.VectorMisalignment, Vectors.VectorSize - this.VectorMisalignment).Fill(0xFF);

            return new Vector<Byte>(misalignmentMask);
        }

        /// <summary>
        /// Create a misalignment mask based on the current vector misalignment. The first N misaligned bytes will be set to 0, the rest 0xFF.
        /// </summary>
        /// <returns>A misalignment mask based on the current vector misalignment.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        protected Vector<Byte> BuildVectorOverreadMask()
        {
            Span<Byte> overreadMask = stackalloc Byte[Vectors.VectorSize];

            overreadMask.Slice(0, Vectors.VectorSize - this.VectorOverread).Fill(0xFF);

            return new Vector<Byte>(overreadMask);
        }

        /// <summary>
        /// Run-length encodes the given scan results into snapshot regions.
        /// </summary>
        /// <param name="scanResults">The scan results to encode.</param>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        protected void EncodeScanResults(ref Vector<Byte> scanResults)
        {
            // Optimization: check all vector results true
            if (Vector.GreaterThanAll(scanResults, Vector<Byte>.Zero))
            {
                this.RunLengthEncoder.EncodeRange(Vectors.VectorSize);
            }
            // Optimization: check all vector results false
            else if (Vector.EqualsAll(scanResults, Vector<Byte>.Zero))
            {
                this.RunLengthEncoder.FinalizeCurrentEncodeUnchecked(Vectors.VectorSize);
            }
            else
            {
                // Otherwise the vector contains a mixture of true and false
                for (Int32 resultIndex = 0; resultIndex < Vectors.VectorSize; resultIndex += this.DataTypeSize)
                {
                    if (scanResults[resultIndex] != 0)
                    {
                        this.RunLengthEncoder.EncodeRange(this.DataTypeSize);
                    }
                    else
                    {
                        this.RunLengthEncoder.FinalizeCurrentEncodeUnchecked(this.DataTypeSize);
                    }
                }
            }
        }

        /// <summary>
        /// Calculates the misalignment of the base address of the current snapshot region being scanned. This can be used to correct
        /// the base address to ensure all values can be scanned and fit into vectors as intended.
        /// </summary>
        /// <returns>The misalignment of the base address of the current snapshot region being scanned.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private Int32 CalculateVectorMisalignment()
        {
            Int32 availableByteCount = this.ElementRnage.GetByteCount(this.DataTypeSize);
            Int32 vectorRemainder = availableByteCount % Vectors.VectorSize;
            Int32 vectorAlignedByteCount = vectorRemainder <= 0 ? availableByteCount : (availableByteCount - vectorRemainder + Vectors.VectorSize);
            UInt64 vectorEndAddress = unchecked(this.ElementRnage.BaseElementAddress + (UInt64)vectorAlignedByteCount);
            Int32 vectorMisalignment = vectorEndAddress <= this.ElementRnage.ParentRegion.EndAddress ? 0 : unchecked((Int32)(vectorEndAddress - this.ElementRnage.ParentRegion.EndAddress));

            return vectorMisalignment;
        }

        /// <summary>
        /// Calculates the number of extra bytes read by the final scan vector of the current snapshot being scanned.
        /// </summary>
        /// <returns>The number of extra bytes read by the final scan vector of the current snapshot being scanned.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private Int32 CalculateVectorOverread()
        {
            Int32 remainingBytes = this.ElementRnage.Range % Vectors.VectorSize;
            Int32 vectorOverread = remainingBytes == 0 ? 0 : (Vectors.VectorSize - remainingBytes);

            return vectorOverread;
        }

        /// <summary>
        /// Gets the appropriate comparison function for a changed value scan.
        /// </summary>
        private unsafe Func<Vector<Byte>> GetComparisonChanged()
        {
            switch (this.DataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return () => Vector.OnesComplement(Vector.Equals(this.CurrentValues, this.PreviousValues));
                case ScannableType type when type == ScannableType.SByte:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSByte(this.CurrentValues), Vector.AsVectorSByte(this.PreviousValues))));
                case ScannableType type when type == ScannableType.Int16:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt16(this.CurrentValues), Vector.AsVectorInt16(this.PreviousValues))));
                case ScannableType type when type == ScannableType.Int16BE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt16(this.CurrentValuesBigEndian16), Vector.AsVectorInt16(this.PreviousValuesBigEndian16))));
                case ScannableType type when type == ScannableType.Int32:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt32(this.CurrentValues), Vector.AsVectorInt32(this.PreviousValues))));
                case ScannableType type when type == ScannableType.Int32BE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt32(this.CurrentValuesBigEndian32), Vector.AsVectorInt32(this.PreviousValuesBigEndian32))));
                case ScannableType type when type == ScannableType.Int64:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt64(this.CurrentValues), Vector.AsVectorInt64(this.PreviousValues))));
                case ScannableType type when type == ScannableType.Int64BE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt64(this.CurrentValuesBigEndian64), Vector.AsVectorInt64(this.PreviousValuesBigEndian64))));
                case ScannableType type when type == ScannableType.UInt16:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt16(this.CurrentValues), Vector.AsVectorUInt16(this.PreviousValues))));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt16(this.CurrentValuesBigEndian16), Vector.AsVectorUInt16(this.PreviousValuesBigEndian16))));
                case ScannableType type when type == ScannableType.UInt32:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt32(this.CurrentValues), Vector.AsVectorUInt32(this.PreviousValues))));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt32(this.CurrentValuesBigEndian32), Vector.AsVectorUInt32(this.PreviousValuesBigEndian32))));
                case ScannableType type when type == ScannableType.UInt64:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt64(this.CurrentValues), Vector.AsVectorUInt64(this.PreviousValues))));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt64(this.CurrentValuesBigEndian64), Vector.AsVectorUInt64(this.PreviousValuesBigEndian64))));
                case ScannableType type when type == ScannableType.Single:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSingle(this.CurrentValues), Vector.AsVectorSingle(this.PreviousValues))));
                case ScannableType type when type == ScannableType.SingleBE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSingle(this.CurrentValuesBigEndian32), Vector.AsVectorSingle(this.PreviousValuesBigEndian32))));
                case ScannableType type when type == ScannableType.Double:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorDouble(this.CurrentValues), Vector.AsVectorDouble(this.PreviousValues))));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorDouble(this.CurrentValuesBigEndian64), Vector.AsVectorDouble(this.PreviousValuesBigEndian64))));
                default:
                    throw new ArgumentException("Unsupported data type provided.");
            }
        }

        /// <summary>
        /// Gets the appropriate comparison function for an unchanged value scan.
        /// </summary>
        private unsafe Func<Vector<Byte>> GetComparisonUnchanged()
        {
            switch (this.DataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return () => Vector.Equals(this.CurrentValues, this.PreviousValues);
                case ScannableType type when type == ScannableType.SByte:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSByte(this.CurrentValues), Vector.AsVectorSByte(this.PreviousValues)));
                case ScannableType type when type == ScannableType.Int16:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt16(this.CurrentValues), Vector.AsVectorInt16(this.PreviousValues)));
                case ScannableType type when type == ScannableType.Int16BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt16(this.CurrentValuesBigEndian16), Vector.AsVectorInt16(this.PreviousValuesBigEndian16)));
                case ScannableType type when type == ScannableType.Int32:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt32(this.CurrentValues), Vector.AsVectorInt32(this.PreviousValues)));
                case ScannableType type when type == ScannableType.Int32BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt32(this.CurrentValuesBigEndian32), Vector.AsVectorInt32(this.PreviousValuesBigEndian32)));
                case ScannableType type when type == ScannableType.Int64:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt64(this.CurrentValues), Vector.AsVectorInt64(this.PreviousValues)));
                case ScannableType type when type == ScannableType.Int64BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt64(this.CurrentValuesBigEndian64), Vector.AsVectorInt64(this.PreviousValuesBigEndian64)));
                case ScannableType type when type == ScannableType.UInt16:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt16(this.CurrentValues), Vector.AsVectorUInt16(this.PreviousValues)));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt16(this.CurrentValuesBigEndian16), Vector.AsVectorUInt16(this.PreviousValuesBigEndian16)));
                case ScannableType type when type == ScannableType.UInt32:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt32(this.CurrentValues), Vector.AsVectorUInt32(this.PreviousValues)));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt32(this.CurrentValuesBigEndian32), Vector.AsVectorUInt32(this.PreviousValuesBigEndian32)));
                case ScannableType type when type == ScannableType.UInt64:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt64(this.CurrentValues), Vector.AsVectorUInt64(this.PreviousValues)));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt64(this.CurrentValuesBigEndian64), Vector.AsVectorUInt64(this.PreviousValuesBigEndian64)));
                case ScannableType type when type == ScannableType.Single:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSingle(this.CurrentValues), Vector.AsVectorSingle(this.PreviousValues)));
                case ScannableType type when type == ScannableType.SingleBE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSingle(this.CurrentValuesBigEndian32), Vector.AsVectorSingle(this.PreviousValuesBigEndian32)));
                case ScannableType type when type == ScannableType.Double:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorDouble(this.CurrentValues), Vector.AsVectorDouble(this.PreviousValues)));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorDouble(this.CurrentValuesBigEndian64), Vector.AsVectorDouble(this.PreviousValuesBigEndian64)));
                default:
                    throw new ArgumentException("Unsupported data type provided.");
            }
        }

        /// <summary>
        /// Gets the appropriate comparison function for an increased value scan.
        /// </summary>
        private unsafe Func<Vector<Byte>> GetComparisonIncreased()
        {
            switch (this.DataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return () => Vector.GreaterThan(this.CurrentValues, this.PreviousValues);
                case ScannableType type when type == ScannableType.SByte:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorSByte(this.CurrentValues), Vector.AsVectorSByte(this.PreviousValues)));
                case ScannableType type when type == ScannableType.Int16:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorInt16(this.CurrentValues), Vector.AsVectorInt16(this.PreviousValues)));
                case ScannableType type when type == ScannableType.Int16BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorInt16(this.CurrentValuesBigEndian16), Vector.AsVectorInt16(this.PreviousValuesBigEndian16)));
                case ScannableType type when type == ScannableType.Int32:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorInt32(this.CurrentValues), Vector.AsVectorInt32(this.PreviousValues)));
                case ScannableType type when type == ScannableType.Int32BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorInt32(this.CurrentValuesBigEndian32), Vector.AsVectorInt32(this.PreviousValuesBigEndian32)));
                case ScannableType type when type == ScannableType.Int64:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorInt64(this.CurrentValues), Vector.AsVectorInt64(this.PreviousValues)));
                case ScannableType type when type == ScannableType.Int64BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorInt64(this.CurrentValuesBigEndian64), Vector.AsVectorInt64(this.PreviousValuesBigEndian64)));
                case ScannableType type when type == ScannableType.UInt16:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorUInt16(this.CurrentValues), Vector.AsVectorUInt16(this.PreviousValues)));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorUInt16(this.CurrentValuesBigEndian16), Vector.AsVectorUInt16(this.PreviousValuesBigEndian16)));
                case ScannableType type when type == ScannableType.UInt32:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorUInt32(this.CurrentValues), Vector.AsVectorUInt32(this.PreviousValues)));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorUInt32(this.CurrentValuesBigEndian32), Vector.AsVectorUInt32(this.PreviousValuesBigEndian32)));
                case ScannableType type when type == ScannableType.UInt64:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorUInt64(this.CurrentValues), Vector.AsVectorUInt64(this.PreviousValues)));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorUInt64(this.CurrentValuesBigEndian64), Vector.AsVectorUInt64(this.PreviousValuesBigEndian64)));
                case ScannableType type when type == ScannableType.Single:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorSingle(this.CurrentValues), Vector.AsVectorSingle(this.PreviousValues)));
                case ScannableType type when type == ScannableType.SingleBE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorSingle(this.CurrentValuesBigEndian32), Vector.AsVectorSingle(this.PreviousValuesBigEndian32)));
                case ScannableType type when type == ScannableType.Double:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorDouble(this.CurrentValues), Vector.AsVectorDouble(this.PreviousValues)));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorDouble(this.CurrentValuesBigEndian64), Vector.AsVectorDouble(this.PreviousValuesBigEndian64)));
                default:
                    throw new ArgumentException("Unsupported data type provided.");
            }
        }

        /// <summary>
        /// Gets the appropriate comparison function for a decreased value scan.
        /// </summary>
        private unsafe Func<Vector<Byte>> GetComparisonDecreased()
        {
            switch (this.DataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return () => Vector.LessThan(this.CurrentValues, this.PreviousValues);
                case ScannableType type when type == ScannableType.SByte:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorSByte(this.CurrentValues), Vector.AsVectorSByte(this.PreviousValues)));
                case ScannableType type when type == ScannableType.Int16:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorInt16(this.CurrentValues), Vector.AsVectorInt16(this.PreviousValues)));
                case ScannableType type when type == ScannableType.Int16BE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorInt16(this.CurrentValuesBigEndian16), Vector.AsVectorInt16(this.PreviousValuesBigEndian16)));
                case ScannableType type when type == ScannableType.Int32:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorInt32(this.CurrentValues), Vector.AsVectorInt32(this.PreviousValues)));
                case ScannableType type when type == ScannableType.Int32BE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorInt32(this.CurrentValuesBigEndian32), Vector.AsVectorInt32(this.PreviousValuesBigEndian32)));
                case ScannableType type when type == ScannableType.Int64:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorInt64(this.CurrentValues), Vector.AsVectorInt64(this.PreviousValues)));
                case ScannableType type when type == ScannableType.Int64BE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorInt64(this.CurrentValuesBigEndian64), Vector.AsVectorInt64(this.PreviousValuesBigEndian64)));
                case ScannableType type when type == ScannableType.UInt16:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorUInt16(this.CurrentValues), Vector.AsVectorUInt16(this.PreviousValues)));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorUInt16(this.CurrentValuesBigEndian16), Vector.AsVectorUInt16(this.PreviousValuesBigEndian16)));
                case ScannableType type when type == ScannableType.UInt32:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorUInt32(this.CurrentValues), Vector.AsVectorUInt32(this.PreviousValues)));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorUInt32(this.CurrentValuesBigEndian32), Vector.AsVectorUInt32(this.PreviousValuesBigEndian32)));
                case ScannableType type when type == ScannableType.UInt64:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorUInt64(this.CurrentValues), Vector.AsVectorUInt64(this.PreviousValues)));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorUInt64(this.CurrentValuesBigEndian64), Vector.AsVectorUInt64(this.PreviousValuesBigEndian64)));
                case ScannableType type when type == ScannableType.Single:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorSingle(this.CurrentValues), Vector.AsVectorSingle(this.PreviousValues)));
                case ScannableType type when type == ScannableType.SingleBE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorSingle(this.CurrentValuesBigEndian32), Vector.AsVectorSingle(this.PreviousValuesBigEndian32)));
                case ScannableType type when type == ScannableType.Double:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorDouble(this.CurrentValues), Vector.AsVectorDouble(this.PreviousValues)));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorDouble(this.CurrentValuesBigEndian64), Vector.AsVectorDouble(this.PreviousValuesBigEndian64)));
                default:
                    throw new ArgumentException("Unsupported data type provided.");
            }
        }

        /// <summary>
        /// Gets the appropriate comparison function for an increased by value scan.
        /// </summary>
        private unsafe Func<Vector<Byte>> GetComparisonIncreasedBy(Object value)
        {
            switch (this.DataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return () => Vector.Equals(this.CurrentValues, Vector.Add(this.PreviousValues, new Vector<Byte>(unchecked((Byte)value))));
                case ScannableType type when type == ScannableType.SByte:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSByte(this.CurrentValues), Vector.Add(Vector.AsVectorSByte(this.PreviousValues), new Vector<SByte>(unchecked((SByte)value)))));
                case ScannableType type when type == ScannableType.Int16:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt16(this.CurrentValues), Vector.Add(Vector.AsVectorInt16(this.PreviousValues), new Vector<Int16>(unchecked((Int16)value)))));
                case ScannableType type when type == ScannableType.Int16BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt16(this.CurrentValuesBigEndian16), Vector.Add(Vector.AsVectorInt16(this.PreviousValuesBigEndian16), new Vector<Int16>(unchecked((Int16)value)))));
                case ScannableType type when type == ScannableType.Int32:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt32(this.CurrentValues), Vector.Add(Vector.AsVectorInt32(this.PreviousValues), new Vector<Int32>(unchecked((Int32)value)))));
                case ScannableType type when type == ScannableType.Int32BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt32(this.CurrentValuesBigEndian32), Vector.Add(Vector.AsVectorInt32(this.PreviousValuesBigEndian32), new Vector<Int32>(unchecked((Int32)value)))));
                case ScannableType type when type == ScannableType.Int64:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt64(this.CurrentValues), Vector.Add(Vector.AsVectorInt64(this.PreviousValues), new Vector<Int64>(unchecked((Int64)value)))));
                case ScannableType type when type == ScannableType.Int64BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt64(this.CurrentValuesBigEndian64), Vector.Add(Vector.AsVectorInt64(this.PreviousValuesBigEndian64), new Vector<Int64>(unchecked((Int64)value)))));
                case ScannableType type when type == ScannableType.UInt16:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt16(this.CurrentValues), Vector.Add(Vector.AsVectorUInt16(this.PreviousValues), new Vector<UInt16>(unchecked((UInt16)value)))));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt16(this.CurrentValuesBigEndian16), Vector.Add(Vector.AsVectorUInt16(this.PreviousValuesBigEndian16), new Vector<UInt16>(unchecked((UInt16)value)))));
                case ScannableType type when type == ScannableType.UInt32:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt32(this.CurrentValues), Vector.Add(Vector.AsVectorUInt32(this.PreviousValues), new Vector<UInt32>(unchecked((UInt32)value)))));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt32(this.CurrentValuesBigEndian32), Vector.Add(Vector.AsVectorUInt32(this.PreviousValuesBigEndian32), new Vector<UInt32>(unchecked((UInt32)value)))));
                case ScannableType type when type == ScannableType.UInt64:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt64(this.CurrentValues), Vector.Add(Vector.AsVectorUInt64(this.PreviousValues), new Vector<UInt64>(unchecked((UInt64)value)))));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt64(this.CurrentValuesBigEndian64), Vector.Add(Vector.AsVectorUInt64(this.PreviousValuesBigEndian64), new Vector<UInt64>(unchecked((UInt64)value)))));
                case ScannableType type when type == ScannableType.Single:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSingle(this.CurrentValues), Vector.Add(Vector.AsVectorSingle(this.PreviousValues), new Vector<Single>(unchecked((Single)value)))));
                case ScannableType type when type == ScannableType.SingleBE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSingle(this.CurrentValuesBigEndian32), Vector.Add(Vector.AsVectorSingle(this.PreviousValuesBigEndian32), new Vector<Single>(unchecked((Single)value)))));
                case ScannableType type when type == ScannableType.Double:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorDouble(this.CurrentValues), Vector.Add(Vector.AsVectorDouble(this.PreviousValues), new Vector<Double>(unchecked((Double)value)))));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorDouble(this.CurrentValuesBigEndian64), Vector.Add(Vector.AsVectorDouble(this.PreviousValuesBigEndian64), new Vector<Double>(unchecked((Double)value)))));
                default:
                    throw new ArgumentException("Unsupported data type provided.");
            }
        }

        /// <summary>
        /// Gets the appropriate comparison function for a decreased by value scan.
        /// </summary>
        private unsafe Func<Vector<Byte>> GetComparisonDecreasedBy(Object value)
        {
            switch (this.DataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return () => Vector.Equals(this.CurrentValues, Vector.Subtract(this.PreviousValues, new Vector<Byte>(unchecked((Byte)value))));
                case ScannableType type when type == ScannableType.SByte:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSByte(this.CurrentValues), Vector.Subtract(Vector.AsVectorSByte(this.PreviousValues), new Vector<SByte>(unchecked((SByte)value)))));
                case ScannableType type when type == ScannableType.Int16:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt16(this.CurrentValues), Vector.Subtract(Vector.AsVectorInt16(this.PreviousValues), new Vector<Int16>(unchecked((Int16)value)))));
                case ScannableType type when type == ScannableType.Int16BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt16(this.CurrentValuesBigEndian16), Vector.Subtract(Vector.AsVectorInt16(this.PreviousValuesBigEndian16), new Vector<Int16>(unchecked((Int16)value)))));
                case ScannableType type when type == ScannableType.Int32:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt32(this.CurrentValues), Vector.Subtract(Vector.AsVectorInt32(this.PreviousValues), new Vector<Int32>(unchecked((Int32)value)))));
                case ScannableType type when type == ScannableType.Int32BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt32(this.CurrentValuesBigEndian32), Vector.Subtract(Vector.AsVectorInt32(this.PreviousValuesBigEndian32), new Vector<Int32>(unchecked((Int32)value)))));
                case ScannableType type when type == ScannableType.Int64:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt64(this.CurrentValues), Vector.Subtract(Vector.AsVectorInt64(this.PreviousValues), new Vector<Int64>(unchecked((Int64)value)))));
                case ScannableType type when type == ScannableType.Int64BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt64(this.CurrentValuesBigEndian64), Vector.Subtract(Vector.AsVectorInt64(this.PreviousValuesBigEndian64), new Vector<Int64>(unchecked((Int64)value)))));
                case ScannableType type when type == ScannableType.UInt16:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt16(this.CurrentValues), Vector.Subtract(Vector.AsVectorUInt16(this.PreviousValues), new Vector<UInt16>(unchecked((UInt16)value)))));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt16(this.CurrentValuesBigEndian16), Vector.Subtract(Vector.AsVectorUInt16(this.PreviousValuesBigEndian16), new Vector<UInt16>(unchecked((UInt16)value)))));
                case ScannableType type when type == ScannableType.UInt32:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt32(this.CurrentValues), Vector.Subtract(Vector.AsVectorUInt32(this.PreviousValues), new Vector<UInt32>(unchecked((UInt32)value)))));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt32(this.CurrentValuesBigEndian32), Vector.Subtract(Vector.AsVectorUInt32(this.PreviousValuesBigEndian32), new Vector<UInt32>(unchecked((UInt32)value)))));
                case ScannableType type when type == ScannableType.UInt64:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt64(this.CurrentValues), Vector.Subtract(Vector.AsVectorUInt64(this.PreviousValues), new Vector<UInt64>(unchecked((UInt64)value)))));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt64(this.CurrentValuesBigEndian64), Vector.Subtract(Vector.AsVectorUInt64(this.PreviousValuesBigEndian64), new Vector<UInt64>(unchecked((UInt64)value)))));
                case ScannableType type when type == ScannableType.Single:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSingle(this.CurrentValues), Vector.Subtract(Vector.AsVectorSingle(this.PreviousValues), new Vector<Single>(unchecked((Single)value)))));
                case ScannableType type when type == ScannableType.SingleBE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSingle(this.CurrentValuesBigEndian32), Vector.Subtract(Vector.AsVectorSingle(this.PreviousValuesBigEndian32), new Vector<Single>(unchecked((Single)value)))));
                case ScannableType type when type == ScannableType.Double:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorDouble(this.CurrentValues), Vector.Subtract(Vector.AsVectorDouble(this.PreviousValues), new Vector<Double>(unchecked((Double)value)))));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorDouble(this.CurrentValuesBigEndian64), Vector.Subtract(Vector.AsVectorDouble(this.PreviousValuesBigEndian64), new Vector<Double>(unchecked((Double)value)))));
                default:
                    throw new ArgumentException("Unsupported data type provided.");
            }
        }

        /// <summary>
        /// Gets the appropriate comparison function for an equal to value scan.
        /// </summary>
        private unsafe Func<Vector<Byte>> GetComparisonEqual(Object value)
        {
            switch (this.DataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return () => Vector.Equals(this.CurrentValues, new Vector<Byte>(unchecked((Byte)value)));
                case ScannableType type when type == ScannableType.SByte:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSByte(this.CurrentValues), new Vector<SByte>(unchecked((SByte)value))));
                case ScannableType type when type == ScannableType.Int16:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt16(this.CurrentValues), new Vector<Int16>(unchecked((Int16)value))));
                case ScannableType type when type == ScannableType.Int16BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt16(this.CurrentValuesBigEndian16), new Vector<Int16>(unchecked((Int16)value))));
                case ScannableType type when type == ScannableType.Int32:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt32(this.CurrentValues), new Vector<Int32>(unchecked((Int32)value))));
                case ScannableType type when type == ScannableType.Int32BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt32(this.CurrentValuesBigEndian32), new Vector<Int32>(unchecked((Int32)value))));
                case ScannableType type when type == ScannableType.Int64:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt64(this.CurrentValues), new Vector<Int64>(unchecked((Int64)value))));
                case ScannableType type when type == ScannableType.Int64BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt64(this.CurrentValuesBigEndian64), new Vector<Int64>(unchecked((Int64)value))));
                case ScannableType type when type == ScannableType.UInt16:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt16(this.CurrentValues), new Vector<UInt16>(unchecked((UInt16)value))));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt16(this.CurrentValuesBigEndian16), new Vector<UInt16>(unchecked((UInt16)value))));
                case ScannableType type when type == ScannableType.UInt32:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt32(this.CurrentValues), new Vector<UInt32>(unchecked((UInt32)value))));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt32(this.CurrentValuesBigEndian32), new Vector<UInt32>(unchecked((UInt32)value))));
                case ScannableType type when type == ScannableType.UInt64:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt64(this.CurrentValues), new Vector<UInt64>(unchecked((UInt64)value))));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt64(this.CurrentValuesBigEndian64), new Vector<UInt64>(unchecked((UInt64)value))));
                case ScannableType type when type == ScannableType.Single:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSingle(this.CurrentValues), new Vector<Single>(unchecked((Single)value))));
                case ScannableType type when type == ScannableType.SingleBE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSingle(this.CurrentValuesBigEndian32), new Vector<Single>(unchecked((Single)value))));
                case ScannableType type when type == ScannableType.Double:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorDouble(this.CurrentValues), new Vector<Double>(unchecked((Double)value))));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return () => Vector.AsVectorByte(Vector.Equals(Vector.AsVectorDouble(this.CurrentValuesBigEndian64), new Vector<Double>(unchecked((Double)value))));
                default:
                    throw new ArgumentException("Unsupported data type provided.");
            }
        }

        /// <summary>
        /// Gets the appropriate comparison function for a not equal to value scan.
        /// </summary>
        private unsafe Func<Vector<Byte>> GetComparisonNotEqual(Object value)
        {
            switch (this.DataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return () => Vector.OnesComplement(Vector.Equals(this.CurrentValues, new Vector<Byte>(unchecked((Byte)value))));
                case ScannableType type when type == ScannableType.SByte:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSByte(this.CurrentValues), new Vector<SByte>(unchecked((SByte)value)))));
                case ScannableType type when type == ScannableType.Int16:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt16(this.CurrentValues), new Vector<Int16>(unchecked((Int16)value)))));
                case ScannableType type when type == ScannableType.Int16BE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt16(this.CurrentValuesBigEndian16), new Vector<Int16>(unchecked((Int16)value)))));
                case ScannableType type when type == ScannableType.Int32:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt32(this.CurrentValues), new Vector<Int32>(unchecked((Int32)value)))));
                case ScannableType type when type == ScannableType.Int32BE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt32(this.CurrentValuesBigEndian32), new Vector<Int32>(unchecked((Int32)value)))));
                case ScannableType type when type == ScannableType.Int64:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt64(this.CurrentValues), new Vector<Int64>(unchecked((Int64)value)))));
                case ScannableType type when type == ScannableType.Int64BE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorInt64(this.CurrentValuesBigEndian64), new Vector<Int64>(unchecked((Int64)value)))));
                case ScannableType type when type == ScannableType.UInt16:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt16(this.CurrentValues), new Vector<UInt16>(unchecked((UInt16)value)))));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt16(this.CurrentValuesBigEndian16), new Vector<UInt16>(unchecked((UInt16)value)))));
                case ScannableType type when type == ScannableType.UInt32:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt32(this.CurrentValues), new Vector<UInt32>(unchecked((UInt32)value)))));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt32(this.CurrentValuesBigEndian32), new Vector<UInt32>(unchecked((UInt32)value)))));
                case ScannableType type when type == ScannableType.UInt64:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt64(this.CurrentValues), new Vector<UInt64>(unchecked((UInt64)value)))));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorUInt64(this.CurrentValuesBigEndian64), new Vector<UInt64>(unchecked((UInt64)value)))));
                case ScannableType type when type == ScannableType.Single:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSingle(this.CurrentValues), new Vector<Single>(unchecked((Single)value)))));
                case ScannableType type when type == ScannableType.SingleBE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorSingle(this.CurrentValuesBigEndian32), new Vector<Single>(unchecked((Single)value)))));
                case ScannableType type when type == ScannableType.Double:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorDouble(this.CurrentValues), new Vector<Double>(unchecked((Double)value)))));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return () => Vector.OnesComplement(Vector.AsVectorByte(Vector.Equals(Vector.AsVectorDouble(this.CurrentValuesBigEndian64), new Vector<Double>(unchecked((Double)value)))));
                default:
                    throw new ArgumentException("Unsupported data type provided.");
            }
        }

        /// <summary>
        /// Gets the appropriate comparison function for a greater than value scan.
        /// </summary>
        private unsafe Func<Vector<Byte>> GetComparisonGreaterThan(Object value)
        {
            switch (this.DataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return () => Vector.GreaterThan(this.CurrentValues, new Vector<Byte>(unchecked((Byte)value)));
                case ScannableType type when type == ScannableType.SByte:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorSByte(this.CurrentValues), new Vector<SByte>(unchecked((SByte)value))));
                case ScannableType type when type == ScannableType.Int16:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorInt16(this.CurrentValues), new Vector<Int16>(unchecked((Int16)value))));
                case ScannableType type when type == ScannableType.Int16BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorInt16(this.CurrentValuesBigEndian16), new Vector<Int16>(unchecked((Int16)value))));
                case ScannableType type when type == ScannableType.Int32:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorInt32(this.CurrentValues), new Vector<Int32>(unchecked((Int32)value))));
                case ScannableType type when type == ScannableType.Int32BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorInt32(this.CurrentValuesBigEndian32), new Vector<Int32>(unchecked((Int32)value))));
                case ScannableType type when type == ScannableType.Int64:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorInt64(this.CurrentValues), new Vector<Int64>(unchecked((Int64)value))));
                case ScannableType type when type == ScannableType.Int64BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorInt64(this.CurrentValues), new Vector<Int64>(unchecked((Int64)value))));
                case ScannableType type when type == ScannableType.UInt16:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorUInt16(this.CurrentValues), new Vector<UInt16>(unchecked((UInt16)value))));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorUInt16(this.CurrentValuesBigEndian16), new Vector<UInt16>(unchecked((UInt16)value))));
                case ScannableType type when type == ScannableType.UInt32:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorUInt32(this.CurrentValues), new Vector<UInt32>(unchecked((UInt32)value))));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorUInt32(this.CurrentValuesBigEndian32), new Vector<UInt32>(unchecked((UInt32)value))));
                case ScannableType type when type == ScannableType.UInt64:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorUInt64(this.CurrentValues), new Vector<UInt64>(unchecked((UInt64)value))));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorUInt64(this.CurrentValuesBigEndian64), new Vector<UInt64>(unchecked((UInt64)value))));
                case ScannableType type when type == ScannableType.Single:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorSingle(this.CurrentValues), new Vector<Single>(unchecked((Single)value))));
                case ScannableType type when type == ScannableType.SingleBE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorSingle(this.CurrentValuesBigEndian32), new Vector<Single>(unchecked((Single)value))));
                case ScannableType type when type == ScannableType.Double:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorDouble(this.CurrentValues), new Vector<Double>(unchecked((Double)value))));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return () => Vector.AsVectorByte(Vector.GreaterThan(Vector.AsVectorDouble(this.CurrentValuesBigEndian64), new Vector<Double>(unchecked((Double)value))));
                default:
                    throw new ArgumentException("Unsupported data type provided.");
            }
        }

        /// <summary>
        /// Gets the appropriate comparison function for a greater than or equal to value scan.
        /// </summary>
        private unsafe Func<Vector<Byte>> GetComparisonGreaterThanOrEqual(Object value)
        {
            switch (this.DataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return () => Vector.GreaterThanOrEqual(this.CurrentValues, new Vector<Byte>(unchecked((Byte)value)));
                case ScannableType type when type == ScannableType.SByte:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorSByte(this.CurrentValues), new Vector<SByte>(unchecked((SByte)value))));
                case ScannableType type when type == ScannableType.Int16:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorInt16(this.CurrentValues), new Vector<Int16>(unchecked((Int16)value))));
                case ScannableType type when type == ScannableType.Int16BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorInt16(this.CurrentValuesBigEndian16), new Vector<Int16>(unchecked((Int16)value))));
                case ScannableType type when type == ScannableType.Int32:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorInt32(this.CurrentValues), new Vector<Int32>(unchecked((Int32)value))));
                case ScannableType type when type == ScannableType.Int32BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorInt32(this.CurrentValuesBigEndian32), new Vector<Int32>(unchecked((Int32)value))));
                case ScannableType type when type == ScannableType.Int64:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorInt64(this.CurrentValues), new Vector<Int64>(unchecked((Int64)value))));
                case ScannableType type when type == ScannableType.Int64BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorInt64(this.CurrentValuesBigEndian64), new Vector<Int64>(unchecked((Int64)value))));
                case ScannableType type when type == ScannableType.UInt16:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorUInt16(this.CurrentValues), new Vector<UInt16>(unchecked((UInt16)value))));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorUInt16(this.CurrentValuesBigEndian16), new Vector<UInt16>(unchecked((UInt16)value))));
                case ScannableType type when type == ScannableType.UInt32:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorUInt32(this.CurrentValues), new Vector<UInt32>(unchecked((UInt32)value))));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorUInt32(this.CurrentValuesBigEndian32), new Vector<UInt32>(unchecked((UInt32)value))));
                case ScannableType type when type == ScannableType.UInt64:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorUInt64(this.CurrentValues), new Vector<UInt64>(unchecked((UInt64)value))));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorUInt64(this.CurrentValuesBigEndian64), new Vector<UInt64>(unchecked((UInt64)value))));
                case ScannableType type when type == ScannableType.Single:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorSingle(this.CurrentValues), new Vector<Single>(unchecked((Single)value))));
                case ScannableType type when type == ScannableType.SingleBE:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorSingle(this.CurrentValuesBigEndian32), new Vector<Single>(unchecked((Single)value))));
                case ScannableType type when type == ScannableType.Double:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorDouble(this.CurrentValues), new Vector<Double>(unchecked((Double)value))));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return () => Vector.AsVectorByte(Vector.GreaterThanOrEqual(Vector.AsVectorDouble(this.CurrentValuesBigEndian64), new Vector<Double>(unchecked((Double)value))));
                default:
                    throw new ArgumentException("Unsupported data type provided.");
            }
        }

        /// <summary>
        /// Gets the appropriate comparison function for a greater than value scan.
        /// </summary>
        private unsafe Func<Vector<Byte>> GetComparisonLessThan(Object value)
        {
            switch (this.DataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return () => Vector.LessThan(this.CurrentValues, new Vector<Byte>(unchecked((Byte)value)));
                case ScannableType type when type == ScannableType.SByte:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorSByte(this.CurrentValues), new Vector<SByte>(unchecked((SByte)value))));
                case ScannableType type when type == ScannableType.Int16:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorInt16(this.CurrentValues), new Vector<Int16>(unchecked((Int16)value))));
                case ScannableType type when type == ScannableType.Int16BE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorInt16(this.CurrentValuesBigEndian16), new Vector<Int16>(unchecked((Int16)value))));
                case ScannableType type when type == ScannableType.Int32:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorInt32(this.CurrentValues), new Vector<Int32>(unchecked((Int32)value))));
                case ScannableType type when type == ScannableType.Int32BE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorInt32(this.CurrentValuesBigEndian32), new Vector<Int32>(unchecked((Int32)value))));
                case ScannableType type when type == ScannableType.Int64:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorInt64(this.CurrentValues), new Vector<Int64>(unchecked((Int64)value))));
                case ScannableType type when type == ScannableType.Int64BE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorInt64(this.CurrentValuesBigEndian64), new Vector<Int64>(unchecked((Int64)value))));
                case ScannableType type when type == ScannableType.UInt16:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorUInt16(this.CurrentValues), new Vector<UInt16>(unchecked((UInt16)value))));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorUInt16(this.CurrentValuesBigEndian16), new Vector<UInt16>(unchecked((UInt16)value))));
                case ScannableType type when type == ScannableType.UInt32:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorUInt32(this.CurrentValues), new Vector<UInt32>(unchecked((UInt32)value))));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorUInt32(this.CurrentValuesBigEndian32), new Vector<UInt32>(unchecked((UInt32)value))));
                case ScannableType type when type == ScannableType.UInt64:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorUInt64(this.CurrentValues), new Vector<UInt64>(unchecked((UInt64)value))));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorUInt64(this.CurrentValuesBigEndian64), new Vector<UInt64>(unchecked((UInt64)value))));
                case ScannableType type when type == ScannableType.Single:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorSingle(this.CurrentValues), new Vector<Single>(unchecked((Single)value))));
                case ScannableType type when type == ScannableType.SingleBE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorSingle(this.CurrentValuesBigEndian32), new Vector<Single>(unchecked((Single)value))));
                case ScannableType type when type == ScannableType.Double:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorDouble(this.CurrentValues), new Vector<Double>(unchecked((Double)value))));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return () => Vector.AsVectorByte(Vector.LessThan(Vector.AsVectorDouble(this.CurrentValuesBigEndian64), new Vector<Double>(unchecked((Double)value))));
                default:
                    throw new ArgumentException("Unsupported data type provided.");
            }
        }

        /// <summary>
        /// Gets the appropriate comparison function for a less than or equal to value scan.
        /// </summary>
        private unsafe Func<Vector<Byte>> GetComparisonLessThanOrEqual(Object value)
        {
            switch (this.DataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return () => Vector.LessThanOrEqual(this.CurrentValues, new Vector<Byte>(unchecked((Byte)value)));
                case ScannableType type when type == ScannableType.SByte:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorSByte(this.CurrentValues), new Vector<SByte>(unchecked((SByte)value))));
                case ScannableType type when type == ScannableType.Int16:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorInt16(this.CurrentValues), new Vector<Int16>(unchecked((Int16)value))));
                case ScannableType type when type == ScannableType.Int16BE:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorInt16(this.CurrentValuesBigEndian16), new Vector<Int16>(unchecked((Int16)value))));
                case ScannableType type when type == ScannableType.Int32:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorInt32(this.CurrentValues), new Vector<Int32>(unchecked((Int32)value))));
                case ScannableType type when type == ScannableType.Int32BE:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorInt32(this.CurrentValuesBigEndian32), new Vector<Int32>(unchecked((Int32)value))));
                case ScannableType type when type == ScannableType.Int64:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorInt64(this.CurrentValues), new Vector<Int64>(unchecked((Int64)value))));
                case ScannableType type when type == ScannableType.Int64BE:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorInt64(this.CurrentValuesBigEndian64), new Vector<Int64>(unchecked((Int64)value))));
                case ScannableType type when type == ScannableType.UInt16:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorUInt16(this.CurrentValues), new Vector<UInt16>(unchecked((UInt16)value))));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorUInt16(this.CurrentValuesBigEndian16), new Vector<UInt16>(unchecked((UInt16)value))));
                case ScannableType type when type == ScannableType.UInt32:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorUInt32(this.CurrentValues), new Vector<UInt32>(unchecked((UInt32)value))));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorUInt32(this.CurrentValuesBigEndian32), new Vector<UInt32>(unchecked((UInt32)value))));
                case ScannableType type when type == ScannableType.UInt64:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorUInt64(this.CurrentValues), new Vector<UInt64>(unchecked((UInt64)value))));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorUInt64(this.CurrentValuesBigEndian64), new Vector<UInt64>(unchecked((UInt64)value))));
                case ScannableType type when type == ScannableType.Single:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorSingle(this.CurrentValues), new Vector<Single>(unchecked((Single)value))));
                case ScannableType type when type == ScannableType.SingleBE:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorSingle(this.CurrentValuesBigEndian32), new Vector<Single>(unchecked((Single)value))));
                case ScannableType type when type == ScannableType.Double:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorDouble(this.CurrentValues), new Vector<Double>(unchecked((Double)value))));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return () => Vector.AsVectorByte(Vector.LessThanOrEqual(Vector.AsVectorDouble(this.CurrentValuesBigEndian64), new Vector<Double>(unchecked((Double)value))));
                default:
                    throw new ArgumentException("Unsupported data type provided.");
            }
        }

        /// <summary>
        /// Sets the default compare action to use for this element.
        /// </summary>
        /// <param name="constraint">The constraint(s) to use for the scan.</param>
        /// <returns></returns>
        /// <exception cref="ArgumentException"></exception>
        /// <exception cref="Exception"></exception>
        private Func<Vector<Byte>> BuildCompareActions(IScanConstraint constraint)
        {
            if (this.CustomVectorCompare != null)
            {
                return this.CustomVectorCompare;
            }

            switch (constraint)
            {
                case ScanConstraints scanConstraints:
                    return this.BuildCompareActions(scanConstraints?.RootConstraint);
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
                    switch (scanConstraint.Constraint)
                    {
                        case ScanConstraint.ConstraintType.Unchanged:
                            return this.GetComparisonUnchanged();
                        case ScanConstraint.ConstraintType.Changed:
                            return this.GetComparisonChanged();
                        case ScanConstraint.ConstraintType.Increased:
                            return this.GetComparisonIncreased();
                        case ScanConstraint.ConstraintType.Decreased:
                            return this.GetComparisonDecreased();
                        case ScanConstraint.ConstraintType.IncreasedByX:
                            return this.GetComparisonIncreasedBy(scanConstraint.ConstraintValue);
                        case ScanConstraint.ConstraintType.DecreasedByX:
                            return this.GetComparisonDecreasedBy(scanConstraint.ConstraintValue);
                        case ScanConstraint.ConstraintType.Equal:
                            return this.GetComparisonEqual(scanConstraint.ConstraintValue);
                        case ScanConstraint.ConstraintType.NotEqual:
                            return this.GetComparisonNotEqual(scanConstraint.ConstraintValue);
                        case ScanConstraint.ConstraintType.GreaterThan:
                            return this.GetComparisonGreaterThan(scanConstraint.ConstraintValue);
                        case ScanConstraint.ConstraintType.GreaterThanOrEqual:
                            return this.GetComparisonGreaterThanOrEqual(scanConstraint.ConstraintValue);
                        case ScanConstraint.ConstraintType.LessThan:
                            return this.GetComparisonLessThan(scanConstraint.ConstraintValue);
                        case ScanConstraint.ConstraintType.LessThanOrEqual:
                            return this.GetComparisonLessThanOrEqual(scanConstraint.ConstraintValue);
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
