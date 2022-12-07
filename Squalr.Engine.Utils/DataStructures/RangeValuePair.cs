namespace Squalr.Engine.Common.DataStructures
{
    using System;
    using System.Collections.Generic;

    /// <summary>
    /// Represents a range of values. Both values must be of the same type and comparable.
    /// </summary>
    /// <typeparam name="TKey">Type of the keys.</typeparam>
    /// <typeparam name="TValue">Type of the values.</typeparam>
    public struct RangeValuePair<TKey, TValue> : IEquatable<RangeValuePair<TKey, TValue>>
    {
        /// <summary>
        /// Initializes a new <see cref="RangeValuePair{TKey, TValue}"/> instance.
        /// </summary>
        /// <param name="from">The lower bound of this range.</param>
        /// <param name="to">The upper bound of this range.</param>
        /// <param name="value">The value contained by this range.param>
        public RangeValuePair(TKey from, TKey to, TValue value) : this()
        {
            this.From = from;
            this.To = to;
            this.Value = value;
        }

        /// <summary>
        /// Gets the lower bound of this range.
        /// </summary>
        public TKey From { get; private set; }

        /// <summary>
        /// Gets the upper bound of this range.
        /// </summary>
        public TKey To { get; private set; }

        /// <summary>
        /// Gets the value contained by this range.
        /// </summary>
        public TValue Value { get; private set; }

        /// <summary>
        /// Returns a <see cref="String"/> that represents this instance.
        /// </summary>
        /// <returns>A <see cref="String"/> that represents this instance.</returns>
        public override String ToString()
        {
            return String.Format("[{0} - {1}] {2}", this.From, this.To, this.Value);
        }

        /// <summary>
        /// Gets a hash code identifying this range value pair. The hash is computed based on the <see cref="From"/>, <see cref="To"/>, and <see cref="Value"/> fields.
        /// </summary>
        /// <returns>A hash code identifying this range value pair.</returns>
        public override Int32 GetHashCode()
        {
            Int32 hash = 23;

            if (this.From != null)
            {
                hash = hash * 37 + From.GetHashCode();
            }

            if (this.To != null)
            {
                hash = hash * 37 + To.GetHashCode();
            }

            if (this.Value != null)
            {
                hash = hash * 37 + Value.GetHashCode();
            }

            return hash;
        }

        /// <summary>
        /// Compares this <see cref="RangeValuePair{TKey, TValue}"/> to another <see cref="RangeValuePair{TKey, TValue}"/> instance. Checks equality on both range and values.
        /// </summary>
        /// <param name="other">The other <see cref="RangeValuePair{TKey, TValue}"/> to which this one is compared.</param>
        /// <returns>A value indicating whether this <see cref="RangeValuePair{TKey, TValue}"/> has the same range and value as the given <see cref="RangeValuePair{TKey, TValue}"/>.</returns>
        public Boolean Equals(RangeValuePair<TKey, TValue> other)
        {
            return EqualityComparer<TKey>.Default.Equals(this.From, other.From)
                   && EqualityComparer<TKey>.Default.Equals(this.To, other.To)
                   && EqualityComparer<TValue>.Default.Equals(this.Value, other.Value);
        }

        /// <summary>
        /// Compares this <see cref="RangeValuePair{TKey, TValue}"/> to another <see cref="RangeValuePair{TKey, TValue}"/> instance. Checks equality on both range and values.
        /// </summary>
        /// <param name="other">The other <see cref="RangeValuePair{TKey, TValue}"/> to which this one is compared.</param>
        /// <returns>A value indicating whether this <see cref="RangeValuePair{TKey, TValue}"/> has the same range and value as the given <see cref="RangeValuePair{TKey, TValue}"/>.</returns>
        public override Boolean Equals(Object other)
        {
            if (!(other is RangeValuePair<TKey, TValue>))
            {
                return false;
            }

            return Equals((RangeValuePair<TKey, TValue>)other);
        }

        /// <summary>
        /// Compares two <see cref="RangeValuePair{TKey, TValue}"/> instances. Checks equality on both range and values.
        /// </summary>
        /// <param name="left">The first <see cref="RangeValuePair{TKey, TValue}"/> instance.</param>
        /// <param name="right">The second <see cref="RangeValuePair{TKey, TValue}"/> instance.</param>
        /// <returns>A value indicating whether the two <see cref="RangeValuePair{TKey, TValue}"/> objects have the same ranges and values.</returns>
        public static Boolean operator ==(RangeValuePair<TKey, TValue> left, RangeValuePair<TKey, TValue> right)
        {
            return left.Equals(right);
        }

        /// <summary>
        /// Compares two <see cref="RangeValuePair{TKey, TValue}"/> instances. Checks inequality on both range and values.
        /// </summary>
        /// <param name="left">The first <see cref="RangeValuePair{TKey, TValue}"/> instance.</param>
        /// <param name="right">The second <see cref="RangeValuePair{TKey, TValue}"/> instance.</param>
        /// <returns>A value indicating whether the two <see cref="RangeValuePair{TKey, TValue}"/> objects have different ranges or values.</returns>
        public static Boolean operator !=(RangeValuePair<TKey, TValue> left, RangeValuePair<TKey, TValue> right)
        {
            return !(left == right);
        }
    }
    //// End class
}
//// End namespace