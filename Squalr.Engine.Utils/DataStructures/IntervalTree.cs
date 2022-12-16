namespace Squalr.Engine.Common.DataStructures
{
    using System;
    using System.Collections;
    using System.Collections.Generic;
    using System.Linq;

    /// <summary>
    /// An interval tree data structure for mapping contiguous key ranges to a single value.
    /// </summary>
    /// <typeparam name="TKey">The key data type.</typeparam>
    /// <typeparam name="TValue">The value data type.</typeparam>
    public class IntervalTree<TKey, TValue> : IIntervalTree<TKey, TValue>
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="IntervalTree{TKey, TValue}" /> class.
        /// </summary>
        public IntervalTree() : this(Comparer<TKey>.Default)
        {
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="IntervalTree{TKey, TValue}" /> class.
        /// </summary>
        /// <param name="comparer">The custom comparer to use for comparing interval tree items.</param>
        public IntervalTree(IComparer<TKey> comparer)
        {
            this.Comparer = comparer ?? Comparer<TKey>.Default;
            this.IsInSync = true;
            this.Root = new IntervalTreeNode<TKey, TValue>(this.Comparer);
            this.Items = new List<RangeValuePair<TKey, TValue>>();
        }

        /// <summary>
        /// Gets the maximum key value contained in this interval tree.
        /// </summary>
        public TKey Max
        {
            get
            {
                if (!this.IsInSync)
                {
                    this.Rebuild();
                }

                return this.Root.Max;
            }
        }

        /// <summary>
        /// Gets the minimum key value contained in this interval tree.
        /// </summary>
        public TKey Min
        {
            get
            {
                if (!this.IsInSync)
                {
                    this.Rebuild();
                }

                return this.Root.Min;
            }
        }

        /// <summary>
        /// Gets all items contained in the tree.
        /// </summary>
        public IEnumerable<TValue> Values => this.Items.Select(i => i.Value);

        /// <summary>
        /// Gets the number of elements contained in the tree.
        /// </summary>
        public Int32 Count => this.Items.Count;

        /// <summary>
        /// Gets or sets the root node of the interval tree.
        /// </summary>
        private IntervalTreeNode<TKey, TValue> Root { get; set; }

        /// <summary>
        /// Gets or sets the list of items contained in the interval tree.
        /// </summary>
        private List<RangeValuePair<TKey, TValue>> Items { get; set; }

        /// <summary>
        /// Gets or sets the custom comparer function for comparing interval tree items.
        /// </summary>
        private IComparer<TKey> Comparer { get; set; }

        /// <summary>
        /// Gets or sets a value indicating whether the interval tree is in sync and able to be queried.
        /// </summary>
        private Boolean IsInSync { get; set; }

        /// <summary>
        /// Returns an enumerator that iterates through the collection.
        /// </summary>
        /// <returns>An enumerator that iterates through the collection.</returns>
        IEnumerator IEnumerable.GetEnumerator() => this.GetEnumerator();

        /// <summary>
        /// Performs a point query with a single value. The first match is returned.
        /// </summary>
        /// <param name="value">The single value for which the query is performed.</param>
        /// <returns>The first result matching the given single value query.</returns>
        public TValue QueryOne(TKey value)
        {
            if (!this.IsInSync)
            {
                this.Rebuild();
            }

            return this.Root.QueryOne(value);
        }

        /// <summary>
        /// Performs a point query with a single value. The first match is returned.
        /// </summary>
        /// <param name="value">The single value for which the query is performed.</param>
        /// <returns>The first result matching the given single value query.</returns>
        public RangeValuePair<TKey, TValue> QueryOneKey(TKey value)
        {
            if (!this.IsInSync)
            {
                this.Rebuild();
            }

            return this.Root.QueryOneKey(value);
        }

        /// <summary>
        /// Performs a point query with a single value. All items with overlapping ranges are returned.
        /// </summary>
        /// <param name="value">The single value for which the query is performed.</param>
        /// <returns>All items matching the given single value query.</returns>
        public IEnumerable<TValue> Query(TKey value)
        {
            if (!this.IsInSync)
            {
                this.Rebuild();
            }

            return this.Root.Query(value);
        }

        /// <summary>
        /// Performs a range query. All items with overlapping ranges are returned.
        /// </summary>
        /// <param name="from">The start of the query range.</param>
        /// <param name="to">The end of the query range.</param>
        /// <returns>All items discovered by this query.</returns>
        public IEnumerable<TValue> Query(TKey from, TKey to)
        {
            if (!this.IsInSync)
            {
                this.Rebuild();
            }

            return this.Root.Query(from, to);
        }

        /// <summary>
        /// Adds the specified item to this interval tree.
        /// </summary>
        /// <param name="from">The start of the item range.</param>
        /// <param name="to">The end of the item range.</param>
        /// <param name="value">The value to insert into the given interval range.</param>
        public void Add(TKey from, TKey to, TValue value)
        {
            if (this.Comparer.Compare(from, to) > 0)
            {
                throw new ArgumentOutOfRangeException($"{nameof(from)} cannot be larger than {nameof(to)}");
            }

            this.IsInSync = false;
            this.Items.Add(new RangeValuePair<TKey, TValue>(from, to, value));
        }

        /// <summary>
        /// Removes the specified item from this interval tree.
        /// </summary>
        /// <param name="item">The item to remove.</param>
        public void Remove(TValue item)
        {
            this.IsInSync = false;

            this.Items.RemoveAll((RangeValuePair<TKey, TValue> next) =>
            {
                return next.Value.Equals(item);
            });
        }

        /// <summary>
        /// Removes the specified items from this interval tree.
        /// </summary>
        /// <param name="items">The items to remove.</param>
        public void Remove(IEnumerable<TValue> items)
        {
            this.IsInSync = false;
            this.Items = this.Items.Where(item => !items.Contains(item.Value)).ToList();
        }

        /// <summary>
        /// Removes all elements from the range tree.
        /// </summary>
        public void Clear()
        {
            this.Root = new IntervalTreeNode<TKey, TValue>(this.Comparer);
            this.Items = new List<RangeValuePair<TKey, TValue>>();
            this.IsInSync = true;
        }

        /// <summary>
        /// Returns an enumerator that iterates through the collection.
        /// </summary>
        /// <returns>An enumerator that iterates through the collection.</returns>
        public IEnumerator<RangeValuePair<TKey, TValue>> GetEnumerator()
        {
            if (!this.IsInSync)
            {
                this.Rebuild();
            }

            return this.Items.GetEnumerator();
        }

        /// <summary>
        /// Rebuilds this interval tree if it is out of sync.
        /// </summary>
        private void Rebuild()
        {
            if (this.IsInSync)
            {
                return;
            }

            if (this.Items.Count > 0)
            {
                this.Root = new IntervalTreeNode<TKey, TValue>(this.Items, this.Comparer);
            }
            else
            {
                this.Root = new IntervalTreeNode<TKey, TValue>(this.Comparer);
            }

            this.IsInSync = true;
        }
    }
    //// End class
}
//// End namespace