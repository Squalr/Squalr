namespace Squalr.Engine.Scanning.Snapshots
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Extensions;
    using System;
    using System.Buffers.Binary;
    using System.Runtime.CompilerServices;
    using System.Runtime.InteropServices;

    /// <summary>
    /// Defines a reference to an element within a snapshot region.
    /// </summary>
    public unsafe class SnapshotElementIndexer
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="SnapshotElementIndexer" /> class.
        /// </summary>
        /// <param name="elementRange">The element range that contains this element.</param>
        /// <param name="alignment">The memory alignment of the snapshot region being indexed.</param>
        /// <param name="elementIndex">The index of the element to begin pointing to.</param>
        public unsafe SnapshotElementIndexer(SnapshotElementRange elementRange, MemoryAlignment alignment, Int32 elementIndex = 0)
        {
            this.ElementRange = elementRange;
            this.ElementIndex = elementIndex;
            this.Alignment = alignment;
        }

        /// <summary>
        /// Gets the index of this element.
        /// </summary>
        public Int32 ElementIndex { get; set; }

        /// <summary>
        /// Gets or sets the memory alignment of this indexer.
        /// </summary>
        public MemoryAlignment Alignment { get; set; }

        /// <summary>
        /// Gets or sets the parent snapshot element range.
        /// </summary>
        private SnapshotElementRange ElementRange { get; set; }

        /// <summary>
        /// Gets the base address of this element.
        /// </summary>
        public UInt64 GetBaseAddress()
        {
            return unchecked(this.ElementRange.ParentRegion.BaseAddress + (UInt64)(this.ElementRange.RegionOffset + (this.ElementIndex * (Int32)this.Alignment)));
        }

        public Object LoadCurrentValue(ScannableType dataType)
        {
            fixed (Byte* pointerBase = &this.ElementRange.ParentRegion.CurrentValues[this.ElementRange.RegionOffset + this.ElementIndex * unchecked((Int32)this.Alignment)])
            {
                return LoadValues(dataType, pointerBase);
            }
        }

        public Object LoadPreviousValue(ScannableType dataType)
        {
            fixed (Byte* pointerBase = &this.ElementRange.ParentRegion.PreviousValues[this.ElementRange.RegionOffset + this.ElementIndex * unchecked((Int32)this.Alignment)])
            {
                return LoadValues(dataType, pointerBase);
            }
        }

        private Object LoadValues(ScannableType dataType, Byte* pointerBase)
        {
            switch (dataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return *pointerBase;
                case ScannableType type when type == ScannableType.SByte:
                    return *(SByte*)pointerBase;
                case ScannableType type when type == ScannableType.Int16:
                    return *(Int16*)pointerBase;
                case ScannableType type when type == ScannableType.Int32:
                    return *(Int32*)pointerBase;
                case ScannableType type when type == ScannableType.Int64:
                    return *(Int64*)pointerBase;
                case ScannableType type when type == ScannableType.UInt16:
                    return *(UInt16*)pointerBase;
                case ScannableType type when type == ScannableType.UInt32:
                    return *(UInt32*)pointerBase;
                case ScannableType type when type == ScannableType.UInt64:
                    return *(UInt64*)pointerBase;
                case ScannableType type when type == ScannableType.Single:
                    return *(Single*)pointerBase;
                case ScannableType type when type == ScannableType.Double:
                    return *(Double*)pointerBase;
                case ByteArrayType type:
                    Byte[] byteArray = new Byte[type.Length];
                    Marshal.Copy((IntPtr)pointerBase, byteArray, 0, type.Length);
                    return byteArray;
                case ScannableType type when type == ScannableType.Int16BE:
                    return BinaryPrimitives.ReverseEndianness(*(Int16*)pointerBase);
                case ScannableType type when type == ScannableType.Int32BE:
                    return BinaryPrimitives.ReverseEndianness(*(Int32*)pointerBase);
                case ScannableType type when type == ScannableType.Int64BE:
                    return BinaryPrimitives.ReverseEndianness(*(Int64*)pointerBase);
                case ScannableType type when type == ScannableType.UInt16BE:
                    return BinaryPrimitives.ReverseEndianness(*(UInt16*)pointerBase);
                case ScannableType type when type == ScannableType.UInt32BE:
                    return BinaryPrimitives.ReverseEndianness(*(UInt32*)pointerBase);
                case ScannableType type when type == ScannableType.UInt64BE:
                    return BinaryPrimitives.ReverseEndianness(*(UInt64*)pointerBase);
                case ScannableType type when type == ScannableType.SingleBE:
                    return BitConverter.Int32BitsToSingle(BinaryPrimitives.ReverseEndianness(*(Int32*)pointerBase));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return BitConverter.Int64BitsToDouble(BinaryPrimitives.ReverseEndianness(*(Int64*)pointerBase));
                default:
                    throw new ArgumentException();
            }
        }

        /// <summary>
        /// Determines if this element has a current value associated with it.
        /// </summary>
        /// <returns>True if a current value is present.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public unsafe Boolean HasCurrentValue()
        {
            if (this.ElementRange.ParentRegion.CurrentValues.IsNullOrEmpty())
            {
                return false;
            }

            return true;
        }

        /// <summary>
        /// Determines if this element has a previous value associated with it.
        /// </summary>
        /// <returns>True if a previous value is present.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public unsafe Boolean HasPreviousValue()
        {
            if (this.ElementRange.ParentRegion.PreviousValues.IsNullOrEmpty())
            {
                return false;
            }

            return true;
        }
    }
    //// End class
}
//// End namespace