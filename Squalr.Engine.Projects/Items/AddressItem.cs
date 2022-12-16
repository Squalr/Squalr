namespace Squalr.Engine.Projects.Items
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Memory;
    using Squalr.Engine.Processes;
    using System;
    using System.ComponentModel;
    using System.Runtime.Serialization;

    /// <summary>
    /// Defines an address that can be added to the project explorer.
    /// </summary>
    [KnownType(typeof(ByteArrayType))]
    [DataContract]
    public abstract class AddressItem : ProjectItem
    {
        /// <summary>
        /// The extension for this project item type.
        /// </summary>
        public const String Extension = ".adr";

        /// <summary>
        /// The data type at this address.
        /// </summary>
        [Browsable(false)]
        [DataMember]
        private ScannableType dataType;

        /// <summary>
        /// The value at this address.
        /// </summary>
        [Browsable(false)]
        private Object addressValue;

        /// <summary>
        /// A value indicating whether the value at this address should be displayed as hex.
        /// </summary>
        [Browsable(false)]
        [DataMember]
        private Boolean isValueHex;

        /// <summary>
        /// The effective address after tracing all pointer offsets.
        /// </summary>
        [Browsable(false)]
        private UInt64 calculatedAddress;

        /// <summary>
        /// Initializes a new instance of the <see cref="AddressItem" /> class.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        public AddressItem(ProcessSession processSession) : this(processSession, ScannableType.Int32, "New Address")
        {
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="AddressItem" /> class.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        /// <param name="dataType">The data type of the value at this address.</param>
        /// <param name="description">The description of this address.</param>
        /// <param name="isValueHex">A value indicating whether the value at this address should be displayed as hex.</param>
        /// <param name="value">The value at this address. If none provided, it will be figured out later. Used here to allow immediate view updates upon creation.</param>
        public AddressItem(
            ProcessSession processSession,
            ScannableType dataType,
            String description = "New Address",
            Boolean isValueHex = false,
            Object value = null)
            : base(processSession, description)
        {
            // Bypass setters to avoid running setter code
            this.dataType = dataType;
            this.isValueHex = isValueHex;

            if (!this.isValueHex && SyntaxChecker.CanParseValue(dataType, value?.ToString()))
            {
                this.addressValue = value;
            }
            else if (this.isValueHex && SyntaxChecker.CanParseHex(dataType, value?.ToString()))
            {
                this.addressValue = value;
            }
        }

        /// <summary>
        /// Gets or sets the data type of the value at this address.
        /// </summary>
        public virtual ScannableType DataType
        {
            get
            {
                return this.dataType;
            }

            set
            {
                if (this.dataType == value)
                {
                    return;
                }

                this.dataType = value;

                // Clear our current address value
                this.addressValue = null;

                this.RaisePropertyChanged(nameof(this.DataType));
                this.Save();
            }
        }

        /// <summary>
        /// Gets or sets the value at this address.
        /// </summary>
        public virtual Object AddressValue
        {
            get
            {
                return this.addressValue;
            }

            set
            {
                if (value is String)
                {
                    if (!SyntaxChecker.CanParseValue(this.dataType, value as String))
                    {
                        Logger.Log(LogLevel.Error, "Error setting new value: " + (value as String));
                        return;
                    }

                    value = Conversions.ParsePrimitiveStringAsPrimitive(this.DataType, value as String);
                }

                this.addressValue = value;
                this.WriteValue(value);
                this.RaisePropertyChanged(nameof(this.AddressValue));
                this.RaisePropertyChanged(nameof(this.DisplayValue));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether the value at this address should be displayed as hex.
        /// </summary>
        public virtual Boolean IsValueHex
        {
            get
            {
                return this.isValueHex;
            }

            set
            {
                if (this.isValueHex == value)
                {
                    return;
                }

                this.isValueHex = value;
                this.RaisePropertyChanged(nameof(this.IsValueHex));
                this.Save();
            }
        }

        /// <summary>
        /// Gets or sets the effective address after tracing all pointer offsets.
        /// </summary>
        public virtual UInt64 CalculatedAddress
        {
            get
            {
                return this.calculatedAddress;
            }

            protected set
            {
                if (this.calculatedAddress == value)
                {
                    return;
                }

                this.calculatedAddress = value;
                this.RaisePropertyChanged(nameof(this.CalculatedAddress));
            }
        }

        /// <summary>
        /// Gets the display value for this project item, which is the address value.
        /// </summary>
        [Browsable(false)]
        public override String DisplayValue
        {
            get
            {
                return this.AddressValue?.ToString() ?? String.Empty;
            }

            set
            {
                this.AddressValue = value;
            }
        }

        /// <summary>
        /// Clones this project item.
        /// </summary>
        /// <param name="rename">A value indicating whether to rename this project item to a default after cloning.</param>
        /// <returns>The cloned project item.</returns>
        public override ProjectItem Clone(Boolean rename)
        {
            ProjectItem clone = base.Clone(rename);

            (clone as AddressItem).processSession = this.processSession;

            return clone;
        }

        /// <summary>
        /// Gets the extension for this project item.
        /// </summary>
        /// <returns>The extension for this project item.</returns>
        public override String GetExtension()
        {
            return AddressItem.Extension;
        }

        /// <summary>
        /// Updates this project item. Resolves addresses and values.
        /// </summary>
        public override void Update()
        {
            this.CalculatedAddress = this.ResolveAddress();

            // Freeze current value if this entry is activated
            if (this.IsActivated)
            {
                this.WriteValue(this.AddressValue);
            }
            else
            {
                Object previousValue = this.AddressValue;

                // Otherwise we read as normal (bypass assigning setter and set value directly to avoid a write-back to memory)
                this.addressValue = MemoryReader.Instance.Read(this.processSession?.OpenedProcess, this.DataType, this.CalculatedAddress, out _);

                if (!(this.AddressValue?.Equals(previousValue) ?? false))
                {
                    this.RaisePropertyChanged(nameof(this.AddressValue));
                    this.RaisePropertyChanged(nameof(this.DisplayValue));
                }
            }
        }

        /// <summary>
        /// Resolves the address of this object.
        /// </summary>
        /// <returns>The base address of this object.</returns>
        protected abstract UInt64 ResolveAddress();

        /// <summary>
        /// Writes a value to the computed address of this item.
        /// </summary>
        /// <param name="newValue">The value to write.</param>
        protected virtual void WriteValue(Object newValue)
        {
            if (newValue == null)
            {
                return;
            }

            try
            {
                MemoryWriter.Instance.Write(this.processSession?.OpenedProcess, this.DataType, this.CalculatedAddress, newValue);
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error writing value to memory.", ex);
            }
        }
    }
    //// End class
}
//// End namespace