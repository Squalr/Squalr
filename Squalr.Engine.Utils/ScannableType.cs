/// <summary>
/// Custom data type classes, namespaced under System for consistency in naming patterns.
/// This also avoids any potential project compatibility issues if Squalr namespaces change.
/// Just be sure not to name any class something that might end up under the System namespace naturally.
/// </summary>
namespace System
{
    public class Int16BigEndian
    {
    }

    public class Int32BigEndian
    {
    }

    public class Int64BigEndian
    {
    }

    public class UInt16BigEndian
    {
    }

    public class UInt32BigEndian
    {
    }

    public class UInt64BigEndian
    {
    }

    public class SingleBigEndian
    {
    }

    public class DoubleBigEndian
    {
    }
}

namespace Squalr.Engine.Common
{
    using System;
    using System.Collections.Generic;
    using System.Runtime.Serialization;

    [DataContract]
    public class ByteArrayType : ScannableType
    {
        public ByteArrayType(Int32 length = 1, Byte[] mask = null) : base(typeof(Byte[]))
        {
            this.Length = length;
            this.Mask = mask;
        }

        [DataMember]
        public Int32 Length { get; set; }

        /// <summary>
        /// Gets or sets the mask used during scanning. Not serialized.
        /// </summary>
        [DataMember]
        public Byte[] Mask { get; set; }
    }

    /// <summary>
    /// A class representing a serializable data type. This is a wrapper over the Type class.
    /// </summary>
    [DataContract]
    public class ScannableType
    {
        /// <summary>
        /// DataType for an array of bytes.
        /// </summary>
        public static readonly ByteArrayType NullByteArray = new ByteArrayType();

        /// <summary>
        /// DataType for a boolean.
        /// </summary>
        public static readonly ScannableType Boolean = new ScannableType(typeof(Boolean));

        /// <summary>
        /// DataType for a signed byte.
        /// </summary>
        public static readonly ScannableType SByte = new ScannableType(typeof(SByte));

        /// <summary>
        /// DataType for a 16-bit integer.
        /// </summary>
        public static readonly ScannableType Int16 = new ScannableType(typeof(Int16));

        /// <summary>
        /// DataType for a 16-bit integer (little endian).
        /// </summary>
        public static readonly ScannableType Int16BE = new ScannableType(typeof(Int16BigEndian));

        /// <summary>
        /// DataType for a 32-bit integer.
        /// </summary>
        public static readonly ScannableType Int32 = new ScannableType(typeof(Int32));

        /// <summary>
        /// DataType for a 32-bit integer (little endian).
        /// </summary>
        public static readonly ScannableType Int32BE = new ScannableType(typeof(Int32BigEndian));

        /// <summary>
        /// DataType for a 64-bit integer.
        /// </summary>
        public static readonly ScannableType Int64 = new ScannableType(typeof(Int64));

        /// <summary>
        /// DataType for a 64-bit integer (little endian).
        /// </summary>
        public static readonly ScannableType Int64BE = new ScannableType(typeof(Int64BigEndian));

        /// <summary>
        /// DataType for a byte.
        /// </summary>
        public static readonly ScannableType Byte = new ScannableType(typeof(Byte));

        /// <summary>
        /// DataType for an unsigned 16-bit integer.
        /// </summary>
        public static readonly ScannableType UInt16 = new ScannableType(typeof(UInt16));

        /// <summary>
        /// DataType for an unsigned 16-bit integer (little endian).
        /// </summary>
        public static readonly ScannableType UInt16BE = new ScannableType(typeof(UInt16BigEndian));

        /// <summary>
        /// DataType for an unsigned 32-bit integer.
        /// </summary>
        public static readonly ScannableType UInt32 = new ScannableType(typeof(UInt32));

        /// <summary>
        /// DataType for an unsigned 32-bit integer (little endian).
        /// </summary>
        public static readonly ScannableType UInt32BE = new ScannableType(typeof(UInt32BigEndian));

        /// <summary>
        /// DataType for an unsigned 64-bit integer.
        /// </summary>
        public static readonly ScannableType UInt64 = new ScannableType(typeof(UInt64));

        /// <summary>
        /// DataType for an unsigned 64-bit integer (little endian).
        /// </summary>
        public static readonly ScannableType UInt64BE = new ScannableType(typeof(UInt64BigEndian));

        /// <summary>
        /// DataType for a single precision floating point value.
        /// </summary>
        public static readonly ScannableType Single = new ScannableType(typeof(Single));

        /// <summary>
        /// DataType for a single precision floating point value (little endian).
        /// </summary>
        public static readonly ScannableType SingleBE = new ScannableType(typeof(SingleBigEndian));

        /// <summary>
        /// DataType for a double precision floating point value.
        /// </summary>
        public static readonly ScannableType Double = new ScannableType(typeof(Double));

        /// <summary>
        /// DataType for a double precision floating point value (little endian).
        /// </summary>
        public static readonly ScannableType DoubleBE = new ScannableType(typeof(DoubleBigEndian));

        /// <summary>
        /// DataType for a char.
        /// </summary>
        public static readonly ScannableType Char = new ScannableType(typeof(Char));

        /// <summary>
        /// DataType for a string.
        /// </summary>
        public static readonly ScannableType String = new ScannableType(typeof(String));

        /// <summary>
        /// DataType for an integer pointer.
        /// </summary>
        public static readonly ScannableType IntPtr = new ScannableType(typeof(IntPtr));

        /// <summary>
        /// DataType for an unsigned integer pointer.
        /// </summary>
        public static readonly ScannableType UIntPtr = new ScannableType(typeof(UIntPtr));

        /// <summary>
        /// The list of scannable data types.
        /// </summary>
        private static readonly ScannableType[] ScannableDataTypes = new ScannableType[]
        {
            ScannableType.Boolean,
            ScannableType.SByte,
            ScannableType.Int16,
            ScannableType.Int16BE,
            ScannableType.Int32,
            ScannableType.Int32BE,
            ScannableType.Int64,
            ScannableType.Int64BE,
            ScannableType.Byte,
            ScannableType.UInt16,
            ScannableType.UInt16BE,
            ScannableType.UInt32,
            ScannableType.UInt32BE,
            ScannableType.UInt64,
            ScannableType.UInt64BE,
            ScannableType.Single,
            ScannableType.SingleBE,
            ScannableType.Double,
            ScannableType.DoubleBE,
            ScannableType.NullByteArray,
            ScannableType.Char,
            ScannableType.String,
        };

        private Int32 size;

        /// <summary>
        /// Initializes a new instance of the <see cref="ScannableType" /> class.
        /// </summary>
        public ScannableType() : this(null)
        {
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="ScannableType" /> class.
        /// </summary>
        /// <param name="type">The default type.</param>
        public ScannableType(Type type)
        {
            this.Type = type;
        }

        /// <summary>
        /// Gets the type wrapped by this class.
        /// </summary>
        public Type Type { get; private set; }

        public Int32 Size
        {
            get
            {
                // Conversions.SizeOf() can be expensive if called repeatedly, so only perform this check once.
                if (this.size == 0)
                {
                    this.size = Conversions.SizeOf(this);
                }

                return this.size;
            }
        }

        /// <summary>
        /// Gets or sets the string of the full namespace path representing this type.
        /// </summary>
        [DataMember]
        private String TypeString
        {
            get
            {
                return this.Type?.FullName;
            }

            set
            {
                this.Type = value == null ? null : Type.GetType(value);
            }
        }

        /// <summary>
        /// Implicitly converts a DataType to a Type for comparisons.
        /// </summary>
        /// <param name="dataType">The DataType to convert.</param>
        public static implicit operator Type(ScannableType dataType)
        {
            return dataType?.Type;
        }

        /// <summary>
        /// Implicitly converts a Type to a DataType for comparisons.
        /// </summary>
        /// <param name="type">The Type to convert.</param>
        public static implicit operator ScannableType(Type type)
        {
            if (type == typeof(Byte[]))
            {
                return ScannableType.NullByteArray;
            }

            return new ScannableType(type);
        }

        /// <summary>
        /// Indicates whether this object is equal to another.
        /// </summary>
        /// <param name="self">The object being compared.</param>
        /// <param name="other">The other object.</param>
        /// <returns>True if equal, otherwise false.</returns>
        public static Boolean operator ==(ScannableType self, ScannableType other)
        {
            if (Object.ReferenceEquals(self, other))
            {
                return true;
            }

            return self?.Type == other?.Type;
        }

        /// <summary>
        /// Indicates whether this object is not equal to another.
        /// </summary>
        /// <param name="self">The object being compared.</param>
        /// <param name="other">The other object.</param>
        /// <returns>True if not equal, otherwise false.</returns>
        public static Boolean operator !=(ScannableType self, ScannableType other)
        {
            return !(self == other);
        }

        /// <summary>
        /// Indicates whether this object is equal to another.
        /// </summary>
        /// <param name="self">The object being compared.</param>
        /// <param name="other">The other object.</param>
        /// <returns>True if equal, otherwise false.</returns>
        public static Boolean operator ==(ScannableType self, Type other)
        {
            if (Object.ReferenceEquals(self, other))
            {
                return true;
            }

            return self?.Type == other;
        }

        /// <summary>
        /// Indicates whether this object is not equal to another.
        /// </summary>
        /// <param name="self">The object being compared.</param>
        /// <param name="other">The other object.</param>
        /// <returns>True if not equal, otherwise false.</returns>
        public static Boolean operator !=(ScannableType self, Type other)
        {
            return !(self == other);
        }

        /// <summary>
        /// Indicates whether this object is equal to another.
        /// </summary>
        /// <param name="self">The object being compared.</param>
        /// <param name="other">The other object.</param>
        /// <returns>True if equal, otherwise false.</returns>
        public static Boolean operator ==(Type self, ScannableType other)
        {
            if (Object.ReferenceEquals(self, other))
            {
                return true;
            }

            return self == other?.Type;
        }

        /// <summary>
        /// Indicates whether this object is not equal to another.
        /// </summary>
        /// <param name="self">The object being compared.</param>
        /// <param name="other">The other object.</param>
        /// <returns>True if not equal, otherwise false.</returns>
        public static Boolean operator !=(Type self, ScannableType other)
        {
            return !(self == other);
        }

        /// <summary>
        /// Gets primitive types that are available for scanning.
        /// </summary>
        /// <returns>An enumeration of scannable types.</returns>
        public static IEnumerable<ScannableType> GetScannableDataTypes()
        {
            return ScannableType.ScannableDataTypes;
        }

        /// <summary>
        /// Returns a hashcode for this instance.
        /// </summary>
        /// <returns>A hashcode for this instance.</returns>
        public override Int32 GetHashCode()
        {
            return this.Type.GetHashCode();
        }

        /// <summary>
        /// Indicates whether <see cref="ScannableType" /> objects are equal.
        /// </summary>
        /// <param name="dataType">The other <see cref="ScannableType" />.</param>
        /// <returns>True if the objects have an equal value.</returns>
        public override Boolean Equals(Object dataType)
        {
            return this.Type == (dataType as ScannableType)?.Type;
        }

        /// <summary>
        /// Indicates whether <see cref="ScannableType" /> objects are equal.
        /// </summary>
        /// <param name="dataType">The other <see cref="ScannableType" />.</param>
        /// <returns>True if the objects have an equal value.</returns>
        public Boolean Equals(ScannableType dataType)
        {
            return this.Type == dataType?.Type;
        }

        /// <summary>
        /// Returns a <see cref="String" /> representing the name of the current <see cref="ScannableType" />.
        /// </summary>
        /// <returns>The <see cref="String" /> representing the name of the current <see cref="ScannableType" /></returns>
        public override String ToString()
        {
            return this.Type?.ToString();
        }
    }
    //// End class
}
//// End namespace