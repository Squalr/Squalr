namespace Squalr.Engine.Scanning.Scanners.Pointers.SearchKernels
{
    using Squalr.Engine.Scanning.Scanners.Comparers.Vectorized;
    using System;
    using System.Numerics;

    /// <summary>
    /// Defines an interface for an object that can search for pointers that point within a specified offset of a given set of snapshot regions.
    /// </summary>
    internal interface IVectorPointerSearchKernel
    {
        Func<Vector<Byte>> GetSearchKernel(SnapshotRegionVectorScannerBase snapshotRegionScanner);
    }
    //// End interface
}
//// End namespace