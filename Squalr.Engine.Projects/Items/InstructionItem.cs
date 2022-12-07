namespace Squalr.Engine.Projects.Items
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Extensions;
    using Squalr.Engine.Processes;
    using System;
    using System.ComponentModel;
    using System.Runtime.Serialization;

    /// <summary>
    /// A project item referencing a native machine instruction in memory.
    /// </summary>
    [DataContract]
    public class InstructionItem : AddressItem // TODO: Pointer item? Could be rare cases where an instruction is in the heap.
    {
        /// <summary>
        /// The extension for this project item type.
        /// </summary>
        public new const String Extension = ".ins";

        /// <summary>
        /// The disassembled instruction.
        /// </summary>
        [Browsable(false)]
        [DataMember]
        private String instruction;

        /// <summary>
        /// The identifier for the base address of this object.
        /// </summary>
        [Browsable(false)]
        [DataMember]
        private String moduleName;

        /// <summary>
        /// The base address of this object. This will be added as an offset from the resolved base identifier.
        /// </summary>
        [Browsable(false)]
        [DataMember]
        private UInt64 moduleOffset;

        /// <summary>
        /// The bytes that preceede this instruction. Used to help in finding this instruction later via an array of bytes scans if needed.
        /// </summary>
        [Browsable(false)]
        [DataMember]
        private Byte[] precedingBytes;

        /// <summary>
        /// The raw instruction bytes for this instruction item.
        /// </summary>
        [Browsable(false)]
        [DataMember]
        private Byte[] instructionBytes;

        /// <summary>
        /// The bytes that follow this instruction. Used to help in finding this instruction later via an array of bytes scans if needed.
        /// </summary>
        [Browsable(false)]
        [DataMember]
        private Byte[] followingBytes;

        /// <summary>
        /// Initializes a new instance of the <see cref="InstructionItem" /> class.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        public InstructionItem(ProcessSession processSession) : this(processSession, 0, null, null, null)
        {
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="InstructionItem" /> class.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        /// <param name="moduleOffset">The module offset of this instruction item. If the address is not module based, this will be the raw address.</param>
        /// <param name="moduleName">The module name from which this instruction is based.</param>
        /// <param name="instruction">The disassembled instruction string.</param>
        /// <param name="instructionBytes">The bytes of this instruction.</param>
        public InstructionItem(ProcessSession processSession, UInt64 moduleOffset, String moduleName, String instruction, Byte[] instructionBytes)
            : base(processSession, ScannableType.NullByteArray, "New Instruction")
        {
            this.ModuleOffset = moduleOffset;
            this.ModuleName = moduleName;
            this.Instruction = instruction;
            this.InstructionBytes = instructionBytes;
        }

        /// <summary>
        /// Gets or sets the value at this address.
        /// </summary>
        public override Object AddressValue
        {
            get
            {
                return base.AddressValue;
            }

            set
            {
                // Assemble and write bytes
                this.RaisePropertyChanged(nameof(this.AddressValue));
            }
        }

        /// <summary>
        /// Gets or sets the bytes that preceede this instruction. Used to help in finding this instruction later via an array of bytes scans if needed.
        /// </summary>
        [Browsable(false)]
        public Byte[] PrecedingBytes
        {
            get
            {
                return this.precedingBytes;
            }

            set
            {
                if (this.precedingBytes == value)
                {
                    return;
                }

                this.precedingBytes = value;
                this.RaisePropertyChanged(nameof(this.PrecedingBytes));
                this.Save();
            }
        }

        /// <summary>
        /// Gets or sets the raw instruction bytes for this instruction item.
        /// </summary>
        public virtual Byte[] InstructionBytes
        {
            get
            {
                return this.instructionBytes;
            }

            set
            {
                if (this.instructionBytes == value)
                {
                    return;
                }

                this.instructionBytes = value;

                this.RaisePropertyChanged(nameof(this.InstructionBytes));
                this.Save();
            }
        }

        /// <summary>
        /// Gets or sets the bytes that follow this instruction. Used to help in finding this instruction later via an array of bytes scans if needed.
        /// </summary>
        public Byte[] FollowingBytes
        {
            get
            {
                return this.followingBytes;
            }

            set
            {
                if (this.followingBytes == value)
                {
                    return;
                }

                this.followingBytes = value;
                this.RaisePropertyChanged(nameof(this.FollowingBytes));
                this.Save();
            }
        }

        /// <summary>
        /// Gets or sets the disassembled instruction.
        /// </summary>
        public virtual String Instruction
        {
            get
            {
                return this.instruction;
            }

            set
            {
                this.instruction = value;
                this.RaisePropertyChanged(nameof(this.Instruction));
                this.Save();
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether the value at this address should be displayed as hex.
        /// </summary>
        public override Boolean IsValueHex
        {
            get
            {
                return true;
            }

            set
            {
                throw new NotImplementedException();
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether the value at this address should be displayed as hex.
        /// </summary>
        public override ScannableType DataType
        {
            get
            {
                return null;
            }

            set
            {
                throw new NotImplementedException();
            }
        }

        /// <summary>
        /// Gets or sets the identifier for the base address of this object.
        /// </summary>
        public virtual String ModuleName
        {
            get
            {
                return this.moduleName;
            }

            set
            {
                if (this.moduleName == value)
                {
                    return;
                }

                this.moduleName = value ?? String.Empty;

                this.RaisePropertyChanged(nameof(this.ModuleName));
                this.Save();
            }
        }

        /// <summary>
        /// Gets or sets the base address of this object. This will be added as an offset from the resolved base identifier.
        /// </summary>
        public virtual UInt64 ModuleOffset
        {
            get
            {
                return this.moduleOffset;
            }

            set
            {
                if (this.moduleOffset == value)
                {
                    return;
                }

                this.CalculatedAddress = value;
                this.moduleOffset = value;
                this.RaisePropertyChanged(nameof(this.ModuleOffset));
                this.RaisePropertyChanged(nameof(this.AddressSpecifier));
                this.Save();
            }
        }

        /// <summary>
        /// Gets the address specifier for this address. If a static address, this is 'ModuleName + offset', otherwise this is an address string.
        /// </summary>
        [Browsable(false)]
        public String AddressSpecifier
        {
            get
            {
                if (!this.ModuleName.IsNullOrEmpty())
                {
                    return this.ModuleName + "+" + Conversions.ToHex(this.ModuleOffset, formatAsAddress: true);
                }
                else
                {
                    return Conversions.ToHex(this.ModuleOffset, formatAsAddress: true);
                }
            }
        }

        /// <summary>
        /// Gets the extension for this project item.
        /// </summary>
        /// <returns>The extension for this project item.</returns>
        public override String GetExtension()
        {
            return InstructionItem.Extension;
        }

        /// <summary>
        /// Resolves the address of this instruction.
        /// </summary>
        /// <returns>The base address of this instruction.</returns>
        protected override UInt64 ResolveAddress()
        {
            return 0; // return AddressResolver.GetInstance().ResolveModule(this.ModuleName).Add(this.ModuleOffset);
        }
    }
    //// End class
}
//// End namespace