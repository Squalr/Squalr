namespace Squalr.Engine.Memory
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Extensions;
    using Squalr.Engine.Common.Logging;
    using System;
    using System.Collections.Generic;
    using System.Linq;

    /// <summary>
    /// Defines an OS independent region in process memory space.
    /// </summary>
    public class NormalizedRegion
    {
        /// <summary>
        /// The size of the region.
        /// </summary>
        private Int32 regionSize;

        /// <summary>
        /// Initializes a new instance of the <see cref="NormalizedRegion" /> class.
        /// </summary>
        public NormalizedRegion()
        {
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="NormalizedRegion" /> class.
        /// </summary>
        /// <param name="baseAddress">The base address of the region.</param>
        /// <param name="regionSize">The size of the region.</param>
        public NormalizedRegion(UInt64 baseAddress, Int32 regionSize)
        {
            this.GenericConstructor(baseAddress, regionSize);
        }

        /// <summary>
        /// Gets or sets the base address of the region.
        /// </summary>
        public UInt64 BaseAddress { get; set; }

        /// <summary>
        /// Gets or sets the size of the region.
        /// </summary>
        public Int32 RegionSize
        {
            get
            {
                return this.regionSize;
            }

            set
            {
                this.regionSize = value;
            }
        }

        /// <summary>
        /// Gets or sets the end address of the region.
        /// </summary>
        public UInt64 EndAddress
        {
            get
            {
                return unchecked(this.BaseAddress + (UInt64)this.RegionSize);
            }

            set
            {
                this.RegionSize = value.Subtract(this.BaseAddress, wrapAround: false).ToInt32();
            }
        }

        /// <summary>
        /// Determines if a page has a higher base address.
        /// </summary>
        /// <param name="first">The first region being compared.</param>
        /// <param name="second">The second region being compared.</param>
        /// <returns>The result of the comparison.</returns>
        public static Boolean operator >(NormalizedRegion first, NormalizedRegion second)
        {
            return first.BaseAddress > second.BaseAddress;
        }

        /// <summary>
        /// Determines if a page has a lower base address.
        /// </summary>
        /// <param name="first">The first region being compared.</param>
        /// <param name="second">The second region being compared.</param>
        /// <returns>The result of the comparison.</returns>
        public static Boolean operator <(NormalizedRegion first, NormalizedRegion second)
        {
            return first.BaseAddress < second.BaseAddress;
        }

        /// <summary>
        /// Determines if a page has an equal or higher base address.
        /// </summary>
        /// <param name="first">The first region being compared.</param>
        /// <param name="second">The second region being compared.</param>
        /// <returns>The result of the comparison.</returns>
        public static Boolean operator >=(NormalizedRegion first, NormalizedRegion second)
        {
            return first.BaseAddress >= second.BaseAddress;
        }

        /// <summary>
        /// Determines if a page has an equal or lower base address.
        /// </summary>
        /// <param name="first">The first region being compared.</param>
        /// <param name="second">The second region being compared.</param>
        /// <returns>The result of the comparison.</returns>
        public static Boolean operator <=(NormalizedRegion first, NormalizedRegion second)
        {
            return first.BaseAddress <= second.BaseAddress;
        }

        /// <summary>
        /// Explicitly the range of this region.
        /// </summary>
        /// <param name="baseAddress">The base address of the region.</param>
        /// <param name="regionSize">The size of the region.</param>
        public virtual void GenericConstructor(UInt64 baseAddress, Int32 regionSize)
        {
            this.BaseAddress = baseAddress;
            this.RegionSize = regionSize;
        }

        /// <summary>
        /// Updates the base address of this region to match the provided alignment.
        /// </summary>
        /// <param name="alignment">The base address alignment.</param>
        public void Align(MemoryAlignment alignment)
        {
            // The enum values are the same as the integer values
            Int32 alignmentValue = unchecked((Int32)alignment);

            if (alignmentValue <= 0 || this.BaseAddress.Mod(alignmentValue) == 0)
            {
                return;
            }

            // Enforce alignment constraint on base address
            unchecked
            {
                UInt64 endAddress = this.EndAddress;
                this.BaseAddress = this.BaseAddress.Subtract(this.BaseAddress.Mod(alignmentValue), wrapAround: false);
                this.BaseAddress = this.BaseAddress.Add(alignmentValue);
                this.EndAddress = endAddress;
            }
        }

        /// <summary>
        /// Determines if an address is contained in this snapshot.
        /// </summary>
        /// <param name="address">The address for which to search.</param>
        /// <returns>True if the address is contained.</returns>
        public virtual Boolean ContainsAddress(UInt64 address)
        {
            if (address >= this.BaseAddress && address <= this.EndAddress)
            {
                return true;
            }

            return false;
        }

        /// <summary>
        /// Expands a region by the element type size in both directions unconditionally.
        /// </summary>
        /// <param name="expandSize">The size by which to expand this region.</param>
        public virtual void Expand(Int32 expandSize)
        {
            this.BaseAddress = this.BaseAddress.Subtract(expandSize, wrapAround: false);
            this.RegionSize += expandSize * 2;
        }

        /// <summary>
        /// Returns a collection of regions within this region, based on the specified chunking size.
        /// Ex) If this region is 257 bytes, chunking with a size of 64 will return 5 new regions.
        /// </summary>
        /// <param name="chunkSize">The size to break down the region into.</param>
        /// <returns>A collection of regions broken down from the original region based on the chunk size.</returns>
        public IEnumerable<NormalizedRegion> ChunkNormalizedRegion(Int32 chunkSize)
        {
            if (chunkSize <= 0)
            {
                Logger.Log(LogLevel.Fatal, "Invalid chunk size specified for region");
                yield break;
            }

            chunkSize = Math.Min(chunkSize, this.RegionSize);

            Int32 chunkCount = (this.RegionSize / chunkSize) + (this.RegionSize % chunkSize == 0 ? 0 : 1);

            for (Int32 index = 0; index < chunkCount; index++)
            {
                Int32 size = chunkSize;

                // Set size to the remainder if on the final chunk and they are not divisible evenly
                if (index == chunkCount - 1 && this.RegionSize > chunkSize && this.RegionSize % chunkSize != 0)
                {
                    size = this.RegionSize % chunkSize;
                }

                yield return new NormalizedRegion(this.BaseAddress.Add(chunkSize * index), size);
            }
        }
    }
    //// End interface
}
//// End namespace