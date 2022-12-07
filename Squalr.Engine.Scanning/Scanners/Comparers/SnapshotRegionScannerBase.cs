namespace Squalr.Engine.Scanning.Scanners.Comparers
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Scanning.Scanners.Constraints;
    using Squalr.Engine.Scanning.Snapshots;
    using System;
    using System.Collections.Generic;

    /// <summary>
    /// A faster version of SnapshotElementComparer that takes advantage of vectorization/SSE instructions.
    /// </summary>
    internal unsafe abstract class SnapshotRegionScannerBase : ISnapshotRegionScanner
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="SnapshotRegionScannerBase" /> class.
        /// </summary>
        public SnapshotRegionScannerBase()
        {
            this.RunLengthEncoder = new SnapshotRegionRunLengthEncoder();
        }

        /// <summary>
        /// Gets or sets the parent snapshot region.
        /// </summary>
        protected SnapshotElementRange ElementRnage { get; set; }

        /// <summary>
        /// Gets a snapshot region run length encoder, which is used to create the snapshots from this scan.
        /// </summary>
        protected SnapshotRegionRunLengthEncoder RunLengthEncoder { get; private set; }

        /// <summary>
        /// Gets or sets the size of the data type being compared.
        /// </summary>
        protected Int32 DataTypeSize { get; set; }

        /// <summary>
        /// Gets or sets the enforced memory alignment for this scan.
        /// </summary>
        protected MemoryAlignment Alignment { get; set; }

        /// <summary>
        /// Gets or sets the data type being compared.
        /// </summary>
        protected ScannableType DataType { get; set; }

        /// <summary>
        /// Gets or sets the action to perform when disposing this scanner.
        /// </summary>
        private Action OnDispose { get; set; }

        /// <summary>
        /// Initializes this scanner for the given region and constaints.
        /// </summary>
        /// <param name="elementRange">The element range to scan.</param>
        /// <param name="constraints">The set of constraints to use for the element comparisons.</param>
        public virtual void Initialize(SnapshotElementRange elementRange, ScanConstraints constraints)
        {
            this.RunLengthEncoder.Initialize(elementRange);
            this.ElementRnage = elementRange;
            this.DataType = constraints.ElementType;
            this.DataTypeSize = constraints.ElementType.Size;
            this.Alignment = this.DataType is ByteArrayType ? MemoryAlignment.Alignment1
                : (constraints.Alignment == MemoryAlignment.Auto ? (MemoryAlignment)this.DataTypeSize : constraints.Alignment);
        }

        /// <summary>
        /// Sets the action to perform when the scanner is disposed. This callback is used to return this scanner instance to an object pool for recycling.
        /// </summary>
        /// <param name="onDispose">The dispose function callback.</param>
        public void SetDisposeCallback(Action onDispose)
        {
            this.OnDispose = onDispose;
        }

        /// <summary>
        /// Perform cleanup and release references, since this snapshot scanner instance may be recycled and exist in cached memory.
        /// </summary>
        public virtual void Dispose()
        {
            this.RunLengthEncoder.Dispose();
            this.ElementRnage = null;

            if (this.OnDispose != null)
            {
                this.OnDispose();
            }
        }

        /// <summary>
        /// Performs a scan over the given element range, returning the elements that match the scan.
        /// </summary>
        /// <param name="elementRange">The element range to scan.</param>
        /// <param name="constraints">The scan constraints.</param>
        /// <returns>The resulting elements, if any.</returns>
        public abstract IList<SnapshotElementRange> ScanRegion(SnapshotElementRange region, ScanConstraints constraints);
    }
    //// End class
}
//// End namespace
