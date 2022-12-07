namespace Squalr.Engine.Scanning
{
    using Squalr.Engine.Scanning.Snapshots;
    using System.Collections.Concurrent;
    using System.Collections.Generic;

    public class ConcurrentScanElementRangeBag : ConcurrentBag<IList<SnapshotElementRange>>, IEnumerable<SnapshotElementRange>
    {
        IEnumerator<SnapshotElementRange> IEnumerable<SnapshotElementRange>.GetEnumerator()
        {
            foreach (IList<SnapshotElementRange> list in this)
            {
                foreach (SnapshotElementRange item in list)
                {
                    yield return item;
                }
            }
        }
    }
    //// End class
}
//// End namespace
