﻿namespace Squalr.Engine.Common.DataStructures
{
    using System;
    using System.Collections.Generic;

    /// <summary>
    /// A node of the range tree. Given a list of items, it builds its subtree. Contains methods to query the subtree and all interval tree logic.
    /// </summary>
    internal class IntervalTreeNode<TKey, TValue> : IComparer<RangeValuePair<TKey, TValue>>
    {
        /// <summary>
        ///     Initializes an empty node.
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

        public TKey Max { get; }

        public TKey Min { get; }

        private TKey Center { get; set; }

        private IComparer<TKey> Comparer { get; set; }

        private RangeValuePair<TKey, TValue>[] Items { get; set; }

        private IntervalTreeNode<TKey, TValue> LeftNode { get; set; }

        private IntervalTreeNode<TKey, TValue> RightNode { get; set; }

        /// <summary>
        ///     Initializes a node with a list of items, builds the sub tree.
        /// </summary>
        /// <param name="items">The items that should be added to this node</param>
        /// <param name="comparer">The comparer used to compare two items.</param>
        public IntervalTreeNode(IList<RangeValuePair<TKey, TValue>> items, IComparer<TKey> comparer)
        {
            this.Comparer = comparer ?? Comparer<TKey>.Default;

            // first, find the median
            List<TKey> endPoints = new List<TKey>(items.Count * 2);

            foreach (RangeValuePair<TKey, TValue> item in items)
            {
                endPoints.Add(item.From);
                endPoints.Add(item.To);
            }

            endPoints.Sort(this.Comparer);

            // the median is used as center value
            if (endPoints.Count > 0)
            {
                this.Min = endPoints[0];
                this.Center = endPoints[endPoints.Count / 2];
                this.Max = endPoints[endPoints.Count - 1];
            }

            List<RangeValuePair<TKey, TValue>> inner = new List<RangeValuePair<TKey, TValue>>();
            List<RangeValuePair<TKey, TValue>> left = new List<RangeValuePair<TKey, TValue>>();
            List<RangeValuePair<TKey, TValue>> right = new List<RangeValuePair<TKey, TValue>>();

            // iterate over all items
            // if the range of an item is completely left of the center, add it to the left items
            // if it is on the right of the center, add it to the right items
            // otherwise (range overlaps the center), add the item to this node's items
            foreach (RangeValuePair<TKey, TValue> item in items)
            {
                if (this.Comparer.Compare(item.To, Center) < 0)
                {
                    left.Add(item);
                }
                else if (this.Comparer.Compare(item.From, Center) > 0)
                {
                    right.Add(item);
                }
                else
                {
                    inner.Add(item);
                }
            }

            // sort the items, this way the query is faster later on
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

            // create left and right nodes, if there are any items
            if (left.Count > 0)
            {
                LeftNode = new IntervalTreeNode<TKey, TValue>(left, this.Comparer);
            }

            if (right.Count > 0)
            {
                RightNode = new IntervalTreeNode<TKey, TValue>(right, this.Comparer);
            }
        }

        /// <summary>
        /// Performs a point query with a single value. The first match is returned.
        /// </summary>
        public TValue QueryOne(TKey value)
        {
            // If the node has items, check for leaves containing the value.
            if (Items != null)
            {
                foreach (RangeValuePair<TKey, TValue> item in Items)
                {
                    if (Comparer.Compare(item.From, value) > 0)
                    {
                        break;
                    }
                    else if (Comparer.Compare(value, item.From) >= 0 && Comparer.Compare(value, item.To) <= 0)
                    {
                        return item.Value;
                    }
                }
            }

            // Go to the left or go to the right of the tree, depending where the query value lies compared to the center
            Int32 centerComp = Comparer.Compare(value, Center);

            if (this.LeftNode != null && centerComp < 0)
            {
                return this.LeftNode.QueryOne(value);
            }
            else if (RightNode != null && centerComp > 0)
            {
                return this.RightNode.QueryOne(value);
            }

            return default;
        }

        /// <summary>
        /// Performs a point query with a single value. All items with overlapping ranges are returned.
        /// </summary>
        public IEnumerable<TValue> Query(TKey value)
        {
            List<TValue> results = new List<TValue>();

            // If the node has items, check for leaves containing the value.
            if (Items != null)
            {
                foreach (RangeValuePair<TKey, TValue> item in Items)
                {
                    if (Comparer.Compare(item.From, value) > 0)
                    {
                        break;
                    }
                    else if (Comparer.Compare(value, item.From) >= 0 && Comparer.Compare(value, item.To) <= 0)
                    {
                        results.Add(item.Value);
                    }
                }
            }

            // Go to the left or go to the right of the tree, depending where the query value lies compared to the center
            Int32 centerComp = Comparer.Compare(value, Center);

            if (LeftNode != null && centerComp < 0)
            {
                results.AddRange(LeftNode.Query(value));
            }
            else if (RightNode != null && centerComp > 0)
            {
                results.AddRange(RightNode.Query(value));
            }

            return results;
        }

        /// <summary>
        /// Performs a range query. All items with overlapping ranges are returned.
        /// </summary>
        public IEnumerable<TValue> Query(TKey from, TKey to)
        {
            List<TValue> results = new List<TValue>();

            // If the node has items, check for leaves intersecting the range.
            if (Items != null)
            {
                foreach (RangeValuePair<TKey, TValue> o in Items)
                {
                    if (Comparer.Compare(o.From, to) > 0)
                    {
                        break;
                    }
                    else if (Comparer.Compare(to, o.From) >= 0 && Comparer.Compare(from, o.To) <= 0)
                    {
                        results.Add(o.Value);
                    }
                }
            }

            // go to the left or go to the right of the tree, depending
            // where the query value lies compared to the center
            if (LeftNode != null && Comparer.Compare(from, Center) < 0)
            {
                results.AddRange(LeftNode.Query(from, to));
            }

            if (RightNode != null && Comparer.Compare(to, Center) > 0)
            {
                results.AddRange(RightNode.Query(from, to));
            }

            return results;
        }

        /// <summary>
        /// Returns less than 0 if this range's From is less than the other, greater than 0 if greater.
        /// If both are equal, the comparison of the To values is returned.
        /// 0 if both ranges are equal.
        /// </summary>
        /// <param name="a">The first item.</param>
        /// <param name="b">The other item.</param>
        /// <returns></returns>
        Int32 IComparer<RangeValuePair<TKey, TValue>>.Compare(RangeValuePair<TKey, TValue> a, RangeValuePair<TKey, TValue> b)
        {
            Int32 fromComp = Comparer.Compare(a.From, b.From);

            if (fromComp == 0)
            {
                return Comparer.Compare(a.To, b.To);
            }

            return fromComp;
        }
    }
    //// End class
}
//// End namespace