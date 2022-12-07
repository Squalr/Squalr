namespace Squalr.Engine.Common
{
    using System;

    /// <summary>
    /// A class defining a scheduling conflict for a <see cref="TrackableTask"/>.
    /// </summary>
    public class TaskConflictException : Exception
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="TaskConflictException" /> class.
        /// </summary>
        public TaskConflictException() : base()
        {
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="TaskConflictException" /> class.
        /// </summary>
        /// <param name="message">The error message for this exception.</param>
        public TaskConflictException(String message) : base(message)
        {
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="TaskConflictException" /> class.
        /// </summary>
        /// <param name="message">The error message for this exception.</param>
        /// <param name="inner">The inner exception that this exception wraps.</param>
        public TaskConflictException(String message, Exception inner) : base(message, inner)
        {
        }
    }
    //// End class
}
//// End namespace