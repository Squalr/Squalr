namespace Squalr.Engine.Scanning.Scanners.Constraints
{
    using Squalr.Engine.Common;
    using System;

    /// <summary>
    /// Class for storing a collection of constraints to be used in a scan that applies more than one constraint per update.
    /// </summary>
    public class OperationConstraint : IScanConstraint
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="ScanConstraintTree" /> class.
        /// </summary>
        public OperationConstraint(OperationType operation, IScanConstraint left = null, IScanConstraint right = null)
        {
            this.BinaryOperation = operation;

            // TODO: Balance these trees to be "Right heavy". Scans currently early-exit after evaluating the left tree.
            this.Left = left;
            this.Right = right;
        }

        public OperationType BinaryOperation { get; private set; }

        public IScanConstraint Left { get; set; }

        public IScanConstraint Right { get; set; }

        /// <summary>
        /// Sets the element type to which all constraints apply.
        /// </summary>
        /// <param name="elementType">The new element type.</param>
        public void SetElementType(ScannableType elementType)
        {
            this.Left?.SetElementType(elementType);
            this.Right?.SetElementType(elementType);
        }

        public Boolean IsValid()
        {
            return (this.Left?.IsValid() ?? false) && (this.Right?.IsValid() ?? false);
        }

        public IScanConstraint Clone()
        {
            return new OperationConstraint(this.BinaryOperation, this.Left?.Clone(), this.Right?.Clone());
        }

        public enum OperationType
        {
            OR,
            AND,
            XOR,
        }
    }
    //// End class
}
//// End namespace