namespace Squalr.Engine.Common.DataStructures
{
    using System;
    using System.Collections.Generic;

    /// <summary>
    /// A node of the range tree. Given a list of items, it builds its subtree. Contains methods to query the subtree and all interval tree logic.
    /// </summary>
    /// <typeparam name="TKey">The key data type.</typeparam>
    /// <typeparam name="TValue">The value data type.</typeparam>
    internal class IntervalTreeNode<TKey, TValue> : IComparer<RangeValuePair<TKey, TValue>>
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="IntervalTreeNode{TKey, TValue}" /> class.
        /// </summary>
        /// <param name="comparer">The comparer used to compare two items.</param>
        public IntervalTreeNode(IComparer<TKey> comparer)
        {
            this.Comparer = comparer ?? Comparer<TKey>.Default;

            this.Center = default;
            this.LeftNode = null;
            this.RightNode = null;
            this.Items = null;
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="IntervalTreeNode{TKey, TValue}" /> class.
        /// </summary>
        /// <param name="items">The items that should be added to this node.</param>
        /// <param name="comparer">The comparer used to compare two items.</param>
        public IntervalTreeNode(IList<RangeValuePair<TKey, TValue>> items, IComparer<TKey> comparer)
        {
            this.Comparer = comparer ?? Comparer<TKey>.Default;

            // First, find the median
            List<TKey> endPoints = new List<TKey>(items.Count * 2);

            foreach (RangeValuePair<TKey, TValue> item in items)
            {
                endPoints.Add(item.From);
                endPoints.Add(item.To);
            }

            endPoints.Sort(this.Comparer);

            // The median is used as center value
            if (endPoints.Count > 0)
            {
                this.Min = endPoints[0];
                this.Center = endPoints[endPoints.Count / 2];
                this.Max = endPoints[endPoints.Count - 1];
            }

            List<RangeValuePair<TKey, TValue>> inner = new List<RangeValuePair<TKey, TValue>>();
            List<RangeValuePair<TKey, TValue>> left = new List<RangeValuePair<TKey, TValue>>();
            List<RangeValuePair<TKey, TValue>> right = new List<RangeValuePair<TKey, TValue>>();

            // Iterate over all items
            // If the range of an item is completely left of the center, add it to the left items
            // If it is on the right of the center, add it to the right items
            // Otherwise (range overlaps the center), add the item to this node's items
            foreach (RangeValuePair<TKey, TValue> item in items)
            {
                if (this.Comparer.Compare(item.To, this.Center) < 0)
                {
                    left.Add(item);
                }
                else if (this.Comparer.Compare(item.From, this.Center) > 0)
                {
                    right.Add(item);
                }
                else
                {
                    inner.Add(item);
                }
            }

            // Sort the items, this way the query is faster later on
            if (inner.Count > 0)
            {
                if (inner.Count > 1)
                {
                    inner.Sort(this);
                }

                this.Items = inner.ToArray();
            }
            else
            {
                this.Items = null;
            }

            // Create left and right nodes, if there are any items
            if (left.Count > 0)
            {
                this.LeftNode = new IntervalTreeNode<TKey, TValue>(left, this.Comparer);
            }

            if (right.Count > 0)
            {
                this.RightNode = new IntervalTreeNode<TKey, TValue>(right, this.Comparer);
            }
        }

        /// <summary>
        /// Gets the maximum key value contained in this interval tree node.
        /// </summary>
        public TKey Max { get; private set; }

        /// <summary>
        /// Gets the minimum key value contained in this interval tree node.
        /// </summary>
        public TKey Min { get; private set; }

        /// <summary>
        /// Gets the center key value contained in this interval tree node. Used to balance the interval tree.
        /// </summary>
        private TKey Center { get; set; }

        /// <summary>
        /// Gets or sets the custom comparer function for comparing interval tree items.
        /// </summary>
        private IComparer<TKey> Comparer { get; set; }

        /// <summary>
        /// Gets or sets the array of items contained in the interval tree.
        /// </summary>
        private RangeValuePair<TKey, TValue>[] Items { get; set; }

        /// <summary>
        /// Gets or sets the left child node of this interval tree node.
        /// </summary>
        private IntervalTreeNode<TKey, TValue> LeftNode { get; set; }

        /// <summary>
        /// Gets or sets the right child node of this interval tree node.
        /// </summary>
        private IntervalTreeNode<TKey, TValue> RightNode { get; set; }

        /// <summary>
        /// Performs a point query with a single value. The first match is returned.
        /// </summary>
        /// <param name="value">The single value for which the query is performed.</param>
        /// <returns>The first result matching the given single value query.</returns>
        public TValue QueryOne(TKey value)
        {
            // If the node has items, check for leaves containing the value.
            if (this.Items != null)
            {
                foreach (RangeValuePair<TKey, TValue> item in this.Items)
                {
                    if (this.Comparer.Compare(item.From, value) > 0)
                    {
                        break;
                    }
                    else if (this.Comparer.Compare(value, item.From) >= 0 && this.Comparer.Compare(value, item.To) <= 0)
                    {
                        return item.Value;
                    }
                }
            }

            // Go to the left or go to the right of the tree, depending where the query value lies compared to the center
            Int32 centerComp = this.Comparer.Compare(value, this.Center);

            if (this.LeftNode != null && centerComp < 0)
            {
                return this.LeftNode.QueryOne(value);
            }
            else if (this.RightNode != null && centerComp > 0)
            {
                return this.RightNode.QueryOne(value);
            }

            return default;
        }

        /// <summary>
        /// Performs a point query with a single value. All items with overlapping ranges are returned.
        /// </summary>
        /// <param name="value">The single value for which the query is performed.</param>
        /// <returns>All items matching the given single value query.</returns>
        public IEnumerable<TValue> Query(TKey value)
        {
            List<TValue> results = new List<TValue>();

            // If the node has items, check for leaves containing the value.
            if (this.Items != null)
            {
                foreach (RangeValuePair<TKey, TValue> item in this.Items)
                {
                    if (this.Comparer.Compare(item.From, value) > 0)
                    {
                        break;
                    }
                    else if (this.Comparer.Compare(value, item.From) >= 0 && this.Comparer.Compare(value, item.To) <= 0)
                    {
                        results.Add(item.Value);
                    }
                }
            }

            // Go to the left or go to the right of the tree, depending where the query value lies compared to the center
            Int32 centerComp = this.Comparer.Compare(value, this.Center);

            if (this.LeftNode != null && centerComp < 0)
            {
                results.AddRange(this.LeftNode.Query(value));
            }
            else if (this.RightNode != null && centerComp > 0)
            {
                results.AddRange(this.RightNode.Query(value));
            }

            return results;
        }

        /// <summary>
        /// Performs a range query. All items with overlapping ranges are returned.
        /// </summary>
        /// <param name="from">The start of the query range.</param>
        /// <param name="to">The end of the query range.</param>
        /// <returns>All items discovered by this query.</returns>
        public IEnumerable<TValue> Query(TKey from, TKey to)
        {
            List<TValue> results = new List<TValue>();

            // If the node has items, check for leaves intersecting the range.
            if (this.Items != null)
            {
                foreach (RangeValuePair<TKey, TValue> o in this.Items)
                {
                    if (this.Comparer.Compare(o.From, to) > 0)
                    {
                        break;
                    }
                    else if (this.Comparer.Compare(to, o.From) >= 0 && this.Comparer.Compare(from, o.To) <= 0)
                    {
                        results.Add(o.Value);
                    }
                }
            }

            // Go to the left or go to the right of the tree, depending where the query value lies compared to the center
            if (this.LeftNode != null && this.Comparer.Compare(from, this.Center) < 0)
            {
                results.AddRange(this.LeftNode.Query(from, to));
            }

            if (this.RightNode != null && this.Comparer.Compare(to, this.Center) > 0)
            {
                results.AddRange(this.RightNode.Query(from, to));
            }

            return results;
        }

        /// <summary>
        /// Returns less than 0 if this range's From is less than the other, greater than 0 if greater.
        /// If both are equal, the comparison of the To values is returned. 0 if both ranges are equal.
        /// </summary>
        /// <param name="a">The first item.</param>
        /// <param name="b">The other item.</param>
        /// <returns>A value indicating whether the given range is equal, greater, or less than this range.</returns>
        Int32 IComparer<RangeValuePair<TKey, TValue>>.Compare(RangeValuePair<TKey, TValue> a, RangeValuePair<TKey, TValue> b)
        {
            Int32 fromComp = this.Comparer.Compare(a.From, b.From);

            if (fromComp == 0)
            {
                return this.Comparer.Compare(a.To, b.To);
            }

            return fromComp;
        }
    }
    //// End class
}
//// End namespace