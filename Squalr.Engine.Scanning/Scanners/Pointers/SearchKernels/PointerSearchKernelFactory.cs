﻿namespace Squalr.Engine.Scanning.Scanners.Pointers.SearchKernels
{
    using Squalr.Engine.Scanning.Scanners.Pointers.Structures;
    using Squalr.Engine.Scanning.Snapshots;
    using System;

    internal class PointerSearchKernelFactory
    {
        public static IVectorPointerSearchKernel GetSearchKernel(Snapshot boundsSnapshot, UInt32 maxOffset, PointerSize pointerSize)
        {
            throw new NotImplementedException();

            if (false) // boundsSnapshot.SnapshotRegions.Length < 64)
            {
                // Linear is fast for small region sizes
                return new LinearPointerSearchKernel(boundsSnapshot, maxOffset, pointerSize);
            }
            else
            {
                return new SpanPointerSearchKernel(boundsSnapshot, maxOffset, pointerSize);
            }
        }
    }
    //// End class
}
//// End namespace