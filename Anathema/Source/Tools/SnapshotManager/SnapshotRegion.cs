﻿using Binarysharp.MemoryManagement.Memory;
using System;
using System.Collections;

namespace Anathema
{
    /// <summary>
    /// Defines a snapshot of memory in an external process.
    /// </summary>
    public class SnapshotRegion : RemoteRegion, IEnumerable
    {
        protected Byte[] CurrentValues;
        protected Byte[] PreviousValues;
        protected Type ElementType;

        public SnapshotRegion(IntPtr BaseAddress, Int32 RegionSize) : base(null, BaseAddress, RegionSize) { }
        public SnapshotRegion(RemoteRegion RemoteRegion) : base(null, RemoteRegion.BaseAddress, RemoteRegion.RegionSize) { }

        /// <summary>
        /// Indexer to access a unified snapshot element at the specified index
        /// </summary>
        /// <param name="Index"></param>
        /// <returns></returns>
        public SnapshotElement this[Int32 Index]
        {
            get
            {
                return new SnapshotElement(
                BaseAddress + Index, ElementType,
                CurrentValues == null ? (Byte[])null : CurrentValues.SubArray(Index, System.Runtime.InteropServices.Marshal.SizeOf(ElementType)),
                PreviousValues == null ? (Byte[])null : PreviousValues.SubArray(Index, System.Runtime.InteropServices.Marshal.SizeOf(ElementType))
                );
            }
        }

        public void SetElementType(Type ElementTYpe)
        {
            this.ElementType = ElementTYpe;
        }

        public void SetCurrentValues(Byte[] NewValues)
        {
            PreviousValues = CurrentValues;
            CurrentValues = NewValues;
        }

        public Byte[] GetCurrentValues()
        {
            return CurrentValues;
        }

        public Byte[] GetPreviousValues()
        {
            return PreviousValues;
        }

        public virtual IEnumerator GetEnumerator()
        {
            for (Int32 Index = 0; Index < RegionSize; Index++)
                yield return this[Index];
        }
    }

    /// <summary>
    /// Defines a snapshot of memory in an external process, as well as assigned labels to this memory.
    /// </summary>
    public class SnapshotRegion<T> : SnapshotRegion where T : struct
    {
        private T?[] MemoryLabels;

        public SnapshotRegion(IntPtr BaseAddress, Int32 RegionSize) : base(BaseAddress, RegionSize) { }
        public SnapshotRegion(RemoteRegion RemoteRegion) : base(RemoteRegion) { }
        public SnapshotRegion(SnapshotRegion SnapshotRegion) : base(SnapshotRegion)
        {
            CurrentValues = SnapshotRegion.GetCurrentValues() == null ? null : (Byte[])SnapshotRegion.GetCurrentValues().Clone();
            PreviousValues = SnapshotRegion.GetPreviousValues() == null ? null : (Byte[])SnapshotRegion.GetPreviousValues().Clone();
            MemoryLabels = new T?[SnapshotRegion.RegionSize];
        }
        public SnapshotRegion(SnapshotElement<T> SnapshotElement) : base(SnapshotElement.BaseAddress, 1)
        {
            MemoryLabels = new T?[] { SnapshotElement.MemoryLabel };
        }

        public T?[] GetMemoryLabels()
        {
            return MemoryLabels;
        }

        public void SetMemoryLabels(T?[] MemoryLabels)
        {
            this.MemoryLabels = MemoryLabels;
        }

        /// <summary>
        /// Indexer to access a labeled unified snapshot element at the specified index
        /// </summary>
        /// <param name="Index"></param>
        /// <returns></returns>
        public new SnapshotElement<T> this[Int32 Index]
        {
            get
            {
                return new SnapshotElement<T>(
                BaseAddress + Index, ElementType, this, Index,
                CurrentValues == null ? (Byte[])null : CurrentValues.SubArray(Index, System.Runtime.InteropServices.Marshal.SizeOf(ElementType)),
                PreviousValues == null ? (Byte[])null : PreviousValues.SubArray(Index, System.Runtime.InteropServices.Marshal.SizeOf(ElementType)),
                MemoryLabels == null ? (T?)null : MemoryLabels[Index]
                );
            }
            set { if (value.MemoryLabel != null) MemoryLabels[Index] = value.MemoryLabel.Value; else MemoryLabels[Index] = null; }
        }

        public override IEnumerator GetEnumerator()
        {
            for (Int32 Index = 0; Index < RegionSize; Index++)
                yield return this[Index];
        }
    }
}