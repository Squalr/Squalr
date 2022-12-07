namespace Squalr.Engine.Projects.Items
{
    using Squalr.Engine.Processes;
    using System;
    using System.Runtime.Serialization;

    /// <summary>
    /// A project item that specifies a resolvable address in a Java application.
    /// </summary>
    [DataContract]
    public class JavaItem : AddressItem
    {
        /// <summary>
        /// The extension for this project item type.
        /// </summary>
        public new const String Extension = ".jvm";

        /// <summary>
        /// Initializes a new instance of the <see cref="JavaItem" /> class.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        public JavaItem(ProcessSession processSession) : base(processSession)
        {
        }

        /// <summary>
        /// Gets the extension for this project item.
        /// </summary>
        /// <returns>The extension for this project item.</returns>
        public override String GetExtension()
        {
            return JavaItem.Extension;
        }

        /// <summary>
        /// Resolves the raw address of this Java address item.
        /// </summary>
        /// <returns>The resolved raw address of this Java address item.</returns>
        protected override UInt64 ResolveAddress()
        {
            return 0;
        }
    }
    //// End class
}
//// End namespace