namespace Squalr.Engine.Scanning.Scanners.Constraints
{
    using Squalr.Engine.Common;
    using System;
    using System.ComponentModel;

    /// <summary>
    /// Class to define a constraint for certain types of scans.
    /// </summary>
    public class ScanConstraints : IScanConstraint, INotifyPropertyChanged
    {
        private MemoryAlignment alignment;

        /// <summary>
        /// Initializes a new instance of the <see cref="ScanConstraints" /> class.
        /// </summary>
        public ScanConstraints(Type elementType, IScanConstraint rootConstraint, MemoryAlignment alignment)
        {
            this.Alignment = alignment;
            this.RootConstraint = rootConstraint;
            this.SetElementType(elementType);
        }

        /// <summary>
        /// An event that is raised when a property of this object changes.
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;

        /// <summary>
        /// Gets the element type of this constraint manager.
        /// </summary>
        public ScannableType ElementType { get; private set; }

        /// <summary>
        /// Gets or sets the enforced memory alignment.
        /// </summary>
        public MemoryAlignment Alignment
        {
            get
            {
                return ScanSettings.ResolveAutoAlignment(this.alignment, this.ElementType.Size);
            }

            private set
            {
                this.alignment = value;
            }
        }

        /// <summary>
        /// Gets the root constraint for this scan constraint set. Usually, this is just a single scan constraint like "> 5".
        /// </summary>
        public IScanConstraint RootConstraint { get; private set; }

        /// <summary>
        /// Sets the element type to which all constraints apply.
        /// </summary>
        /// <param name="elementType">The new element type.</param>
        public void SetElementType(ScannableType elementType)
        {
            this.ElementType = elementType;
            this.RootConstraint?.SetElementType(elementType);
        }

        public Boolean IsValid()
        {
            return this.RootConstraint?.IsValid() ?? false;
        }

        public IScanConstraint Clone()
        {
            return new ScanConstraints(this.ElementType, this.RootConstraint?.Clone(), this.Alignment);
        }
    }
    //// End class
}
//// End namespace