namespace Squalr.Engine.Common.DataStructures
{
    using System;
    using System.Collections;
    using System.Collections.Generic;

    /// <summary>
    /// A binary heap data structure.
    /// </summary>
    /// <typeparam name="T">The data type contained by this binary heap.</typeparam>
    public class BinaryHeap<T> : IEnumerable<T>
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="BinaryHeap{T}" /> class.
        /// </summary>
        public BinaryHeap() : this(Comparer<T>.Default)
        {
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="BinaryHeap{T}" /> class.
        /// </summary>
        /// <param name="comparer">The custom comparer to use for the binary heap.</param>
        public BinaryHeap(IComparer<T> comparer)
        {
            this.Comparer = comparer;
            this.Items = new List<T>();
        }

        /// <summary>
        /// Gets the number of elements in this binary heap.
        /// </summary>
        public Int32 Count
        {
            get
            {
                return this.Items.Count;
            }
        }

        /// <summary>
        /// Gets or sets the comparer used for binary heap insertion.
        /// </summary>
        protected IComparer<T> Comparer { get; set; }

        /// <summary>
        /// Gets or sets a list that holds all the items in the heap.
        /// </summary>
        protected List<T> Items { get; set; }

        /// <summary>
        /// Inserts a new item into the binary heap.
        /// </summary>
        /// <param name="newItem">The item to insert into the binary heap.</param>
        public virtual void Insert(T newItem)
        {
            Int32 index = this.Count;

            // Add the new item to the bottom of the heap.
            this.Items.Add(newItem);

            // Until the new item is greater than its parent item, swap the two
            while (index > 0 && this.Comparer.Compare(this.Items[(index - 1) / 2], newItem) > 0)
            {
                this.Items[index] = this.Items[(index - 1) / 2];

                index = (index - 1) / 2;
            }

            // The new index in the list is the appropriate location for the new item
            this.Items[index] = newItem;
        }

        /// <summary>
        /// Gets the last element contained in this binary heap.
        /// </summary>
        /// <returns></returns>
        public T Last()
        {
            return this.Items[this.Items.Count - 1];
        }

        /// <summary>
        /// Converts this binary heap tree to an array layout.
        /// </summary>
        /// <returns>The binary heap as an array.</returns>
        public T[] ToArray()
        {
            return this.Items.ToArray();
        }

        /// <summary>
        /// Clears all items from this binary heap.
        /// </summary>
        public void Clear()
        {
            this.Items.Clear();
        }

        /// <summary>
        /// Returns an enumerator that iterates through the collection.
        /// </summary>
        /// <returns>An enumerator that iterates through the collection.</returns>
        public virtual IEnumerator GetEnumerator()
        {
            return this.GetEnumerator();
        }

        /// <summary>
        /// Returns an enumerator that iterates through the collection.
        /// </summary>
        /// <returns>An enumerator that iterates through the collection.</returns>
        IEnumerator<T> IEnumerable<T>.GetEnumerator()
        {
            foreach (T element in this.Items)
            {
                yield return element;
            }
        }
    }
    //// End class
}
//// End namespace