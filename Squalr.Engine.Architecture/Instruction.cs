namespace Squalr.Engine.Architecture
{
    using System;
    using System.ComponentModel;
    using System.Runtime.Serialization;

    /// <summary>
    /// Object that represents a platform agnostic instruction.
    /// </summary>
    [DataContract]
    public class Instruction : INotifyPropertyChanged
    {
        /// <summary>
        /// The instruction address.
        /// </summary>
        private UInt64 address;

        /// <summary>
        /// The string representation of the instruction.
        /// </summary>
        private String mnemonic;

        /// <summary>
        /// Bytes that preceede this instruction. Used to help in finding this instruction later via an array of bytes scans if needed.
        /// </summary>
        private Byte[] precedingBytes;

        /// <summary>
        /// The instruction bytes.
        /// </summary>
        private Byte[] bytes;

        /// <summary>
        /// Bytes that follow this instruction. Used to help in finding this instruction later via an array of bytes scans if needed.
        /// </summary>
        private Byte[] followingBytes;

        /// <summary>
        /// The size of this instruction.
        /// </summary>
        private Int32 size;

        /// <summary>
        /// Initializes a new instance of the <see cref="Instruction" /> class.
        /// </summary>
        /// <param name="address">The instruction address.</param>
        /// <param name="mnemonic">The instruction string.</param>
        /// <param name="bytes">The bytes of the instruction.</param>
        /// <param name="size">The instruction size.</param>
        public Instruction(UInt64 address, String mnemonic, Byte[] bytes, Int32 size)
        {
            this.Address = address;
            this.Mnemonic = mnemonic;
            this.Bytes = bytes;
            this.Size = size;
        }

        /// <summary>
        /// An event that is raised when a property of this object changes.
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;

        /// <summary>
        /// Gets or sets the instruction address.
        /// </summary>
        [DataMember]
        public UInt64 Address
        {
            get
            {
                return this.address;
            }

            set
            {
                this.address = value;
                this.RaisePropertyChanged(nameof(this.Address));
            }
        }

        /// <summary>
        /// Gets or sets the string representation of the instruction.
        /// </summary>
        [DataMember]
        public String Mnemonic
        {
            get
            {
                return this.mnemonic;
            }

            set
            {
                this.mnemonic = value;
                this.RaisePropertyChanged(nameof(this.Mnemonic));
            }
        }

        /// <summary>
        /// Gets or sets the data type of the value at this address.
        /// </summary>
        [DataMember]
        public Byte[] PrecedingBytes
        {
            get
            {
                return this.precedingBytes;
            }

            set
            {
                this.precedingBytes = value;
                this.RaisePropertyChanged(nameof(this.PrecedingBytes));
            }
        }

        /// <summary>
        /// Gets or sets the instruction bytes.
        /// </summary>
        [DataMember]
        public Byte[] Bytes
        {
            get
            {
                return this.bytes;
            }

            set
            {
                this.bytes = value;
                this.RaisePropertyChanged(nameof(this.Bytes));
            }
        }

        /// <summary>
        /// Gets or sets the data type of the value at this address.
        /// </summary>
        [DataMember]
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
            }
        }

        /// <summary>
        /// Gets or sets the size of this instruction.
        /// </summary>
        [DataMember]
        public Int32 Size
        {
            get
            {
                return this.size;
            }

            set
            {
                this.size = value;
                this.RaisePropertyChanged(nameof(this.Size));
            }
        }

        /// <summary>
        /// Indicates that a given property in this project item has changed.
        /// </summary>
        /// <param name="propertyName">The name of the changed property.</param>
        protected void RaisePropertyChanged(String propertyName)
        {
            this.PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
    //// End class
}
//// End namespace