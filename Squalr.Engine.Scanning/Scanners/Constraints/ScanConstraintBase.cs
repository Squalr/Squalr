namespace Squalr.Engine.Scanning.Scanners.Constraints
{
    using Squalr.Engine.Common;
    using System;

    public interface IScanConstraint
    {
        /// <summary>
        /// Sets the element type to which all constraints apply.
        /// </summary>
        /// <param name="elementType">The new element type.</param>
        public abstract void SetElementType(ScannableType elementType);

        public abstract Boolean IsValid();

        /// <summary>
        /// Clones this scan constraint.
        /// </summary>
        /// <returns>The cloned scan constraint.</returns>
        public abstract IScanConstraint Clone();
    }
    //// End interface
}
//// End namespace