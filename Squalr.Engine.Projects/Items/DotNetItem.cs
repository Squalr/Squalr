namespace Squalr.Engine.Projects.Items
{
    using Squalr.Engine.Memory.Clr;
    using Squalr.Engine.Processes;
    using System;
    using System.Runtime.Serialization;

    /// <summary>
    /// A project item that specifies a resolvable address in a .NET application.
    /// </summary>
    [DataContract]
    public class DotNetItem : AddressItem
    {
        /// <summary>
        /// The extension for this project item type.
        /// </summary>
        public new const String Extension = ".clr";

        /// <summary>
        /// The identifier for this .NET object.
        /// </summary>
        [DataMember]
        private String identifier;

        /// <summary>
        /// Initializes a new instance of the <see cref="DotNetItem" /> class.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        public DotNetItem(ProcessSession processSession) : base(processSession)
        {
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="DotNetItem" /> class.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        /// <param name="name">The name of this .NET item.</param>
        /// <param name="type">The type of this .NET item.</param>
        /// <param name="identifier">The unique identifier for this .NET item.</param>
        public DotNetItem(ProcessSession processSession, String name, Type type, String identifier) : base(processSession, type, name)
        {
            this.Identifier = identifier;
        }

        /// <summary>
        /// Gets or sets the identifier for this .NET object.
        /// </summary>
        public virtual String Identifier
        {
            get
            {
                return this.identifier;
            }

            set
            {
                this.identifier = value;
                this.RaisePropertyChanged(nameof(this.Identifier));
            }
        }

        /// <summary>
        /// Gets the extension for this project item.
        /// </summary>
        /// <returns>The extension for this project item.</returns>
        public override String GetExtension()
        {
            return DotNetItem.Extension;
        }

        /// <summary>
        /// Resolves the raw address of this .NET address item.
        /// </summary>
        /// <returns>The resolved raw address of this .NET address item.</returns>
        protected override UInt64 ResolveAddress()
        {
            return AddressResolver.GetInstance().ResolveDotNetObject(this.Identifier);
        }
    }
    //// End class
}
//// End namespace