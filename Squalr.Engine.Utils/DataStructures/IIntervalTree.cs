namespace Squalr.Engine.Common.DataStructures
{
    using System;
    using System.Collections.Generic;

    /// <summary>
    /// The standard interval tree implementation. Keeps a root node and forwards all queries to it.
    /// Whenever new items are added or items are removed, the tree goes temporarily "out of sync", which means that the
    /// internal index is not updated immediately, but upon the next query operation.    
    /// </summary>
    /// <typeparam name="TKey">The key data type.</typeparam>
    /// <typeparam name="TValue">The value data type.</typeparam>
    public interface IIntervalTree<TKey, TValue> : IEnumerable<RangeValuePair<TKey, TValue>>
    {
        /// <summary>
        /// Gets all items contained in the tree.
        /// </summary>
        IEnumerable<TValue> Values { get; }

        /// <summary>
        /// Gets the number of elements contained in the tree.
        /// </summary>
        Int32 Count { get; }

        /// <summary>
        /// Performs a point query with a single value. The first match is returned.
        /// </summary>
        /// <param name="value">The single value for which the query is performed.</param>
        /// <returns>The first result matching the given single value query.</returns>
        TValue QueryOne(TKey value);

        /// <summary>
        /// Performs a point query with a single value. The first match is returned.
        /// </summary>
        /// <param name="value">The single value for which the query is performed.</param>
        /// <returns>The first result matching the given single value query.</returns>
        RangeValuePair<TKey, TValue> QueryOneKey(TKey value);

        /// <summary>
        /// Performs a point query with a single value. All items with overlapping ranges are returned.
        /// </summary>
        /// <param name="value">The single value for which the query is performed.</param>
        /// <returns>All items matching the given single value query.</returns>
        IEnumerable<TValue> Query(TKey value);

        /// <summary>
        /// Performs a range query. All items with overlapping ranges are returned.
        /// </summary>
        /// <param name="from">The start of the query range.</param>
        /// <param name="to">The end of the query range.</param>
        /// <returns>All items discovered by this query.</returns>
        IEnumerable<TValue> Query(TKey from, TKey to);

        /// <summary>
        /// Adds the specified item to this interval tree.
        /// </summary>
        /// <param name="from">The start of the item range.</param>
        /// <param name="to">The end of the item range.</param>
        /// <param name="value">The value to insert into the given interval range.</param>
        void Add(TKey from, TKey to, TValue value);

        /// <summary>
        /// Removes the specified item from this interval tree by query value.
        /// </summary>
        /// <param name="value">The value to remove.</param>
        void Remove(TValue value);

        /// <summary>
        /// Removes the specified items from this interval tree.
        /// </summary>
        /// <param name="items">The items to remove.</param>
        void Remove(IEnumerable<TValue> items);

        /// <summary>
        /// Removes all elements from the range tree.
        /// </summary>
        void Clear();
    }
    //// End interface
}
//// End namespace