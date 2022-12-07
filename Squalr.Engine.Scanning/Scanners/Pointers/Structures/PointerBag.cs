namespace Squalr.Engine.Scanning.Scanners.Pointers.Structures
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Extensions;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Memory;
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Collections;
    using System.Collections.Generic;
    using System.Diagnostics;
    using System.Linq;
    using System.Net;

    /// <summary>
    /// A class to contain the discovered pointers from a pointer scan.
    /// </summary>
    public class PointerBag : IEnumerable<Level>
    {
        private static readonly Random RandInstance = new Random();

        /// <summary>
        /// Initializes a new instance of the <see cref="PointerBag" /> class.
        /// </summary>
        internal PointerBag(IList<Level> levels, UInt32 maxOffset, PointerSize pointerSize)
        {
            this.MaxOffset = maxOffset;
            this.PointerSize = pointerSize;

            // Add levels, removing invalid ones
            this.Levels = levels.TakeWhile(level => level.HeapPointers.ElementCount > 0).ToList();

            if (this.Levels.Count > 0 && this.Levels.Last().StaticPointers.ElementCount <= 0)
            {
                this.Levels.Remove(this.Levels.Last());
            }
        }

        /// <summary>
        /// Gets the list of levels in this pointer bag.
        /// </summary>
        public IList<Level> Levels { get; private set; }

        /// <summary>
        /// Gets the maximum pointer offset.
        /// </summary>
        public UInt32 MaxOffset { get; private set; }

        /// <summary>
        /// Gets the depth of the highest pointer level in this bag.
        /// </summary>
        public Int32 Depth
        {
            get
            {
                return this.Levels.Count;
            }
        }

        /// <summary>
        /// Gets the pointer size.
        /// </summary>
        internal PointerSize PointerSize { get; private set; }

        /// <summary>
        /// Gets a random pointer from the pointer collection.
        /// </summary>
        /// <returns>A random discovered pointer, or null if unable to find one.</returns>
        public Pointer GetRandomPointer(Process targetProcess, Int32 levelIndex)
        {
            if (levelIndex >= this.Levels.Count || this.Levels[levelIndex].StaticPointers == null)
            {
                return null;
            }

            Snapshot currentSnapshot = this.Levels[levelIndex].StaticPointers;
            ExtractedPointer pointer = this.ExtractRandomPointer(currentSnapshot);

            UInt64 pointerBase = pointer.BaseAddress;
            List<Int32> offsets = new List<Int32>();

            foreach (Level level in this.Levels.Take(levelIndex + 1).Reverse())
            {
                IEnumerable<Int32> shuffledOffsets = Enumerable.Range(-(Int32)this.MaxOffset, (Int32)(this.MaxOffset * 2) + 1).Shuffle();

                Boolean found = false;

                // Brute force all possible offsets in a random order to find the next path (this guarantees uniform path probabilities)
                foreach (Int32 nextRandomOffset in shuffledOffsets)
                {
                    UInt64 newDestination = nextRandomOffset < 0 ? pointer.Destination.Subtract(-nextRandomOffset, wrapAround: false) : pointer.Destination.Add(nextRandomOffset, wrapAround: false);
                    SnapshotRegion snapshotRegion = level.HeapPointers.SnapshotRegions.Where(y => newDestination >= y.BaseAddress && newDestination <= y.EndAddress).FirstOrDefault();

                    if (snapshotRegion != null)
                    {
                        // We may have sampled an offset that results in a mis-aligned index, so just randomly take an element from this snapshot rather than using the random offset
                        Int32 elementCount = snapshotRegion.TotalElementCount;
                        SnapshotElementIndexer randomElement = snapshotRegion[PointerBag.RandInstance.Next(0, elementCount).ToUInt64(), currentSnapshot.Alignment];

                        if (randomElement != null)
                        {
                            UInt64 baseAddress = randomElement.GetBaseAddress();
                            Int32 alignedOffset = pointer.Destination >= baseAddress ? -((Int32)(pointer.Destination - baseAddress)) : ((Int32)(baseAddress - pointer.Destination));

                            pointer = this.ExtractPointerFromElement(randomElement);
                            offsets.Add(alignedOffset);
                            found = true;
                            break;
                        }
                    }
                }

                if (!found)
                {
                    Logger.Log(LogLevel.Error, "Unable to collect a pointer, encountered dead end path");
                    return null;
                }
            }

            String moduleName;
            pointerBase = MemoryQueryer.Instance.AddressToModule(targetProcess, pointerBase, out moduleName);

            return new Pointer(moduleName, pointerBase, this.PointerSize, offsets.ToArray());
        }

        private ExtractedPointer ExtractRandomPointer(Snapshot snapshot)
        {
            UInt64 elementIndex = PointerBag.RandInstance.RandomUInt64(0, snapshot.ElementCount);
            SnapshotElementIndexer extractedElement = snapshot[elementIndex, MemoryAlignment.Alignment4];

            if (extractedElement == null)
            {
                Logger.Log(LogLevel.Error, "Unknown error encountered when extracting a pointer.");
                return new ExtractedPointer(0, 0);
            }

            return this.ExtractPointerFromElement(extractedElement);
        }

        private ExtractedPointer ExtractPointerFromElement(SnapshotElementIndexer element)
        {
            return new ExtractedPointer(element.GetBaseAddress(), element.HasCurrentValue()
                ? (this.PointerSize == PointerSize.Byte4 ? (UInt32)element.LoadCurrentValue(PointerSize.ToDataType())
                : (UInt64)element.LoadCurrentValue(PointerSize.ToDataType())) : 0);
        }

        IEnumerator IEnumerable.GetEnumerator() => this.Levels.GetEnumerator();

        public IEnumerator<Level> GetEnumerator()
        {
            return Levels.GetEnumerator();
        }

        private struct ExtractedPointer
        {
            public ExtractedPointer(UInt64 address, UInt64 destination)
            {
                this.BaseAddress = address;
                this.Destination = destination;
            }

            public UInt64 BaseAddress { get; private set; }

            public UInt64 Destination { get; private set; }
        }
    }
    //// End class
}
//// End namespace
