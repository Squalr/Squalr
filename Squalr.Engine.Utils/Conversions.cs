namespace Squalr.Engine.Common
{
    using Squalr.Engine.Common.Extensions;
    using System;
    using System.Buffers.Binary;
    using System.Collections.Generic;
    using System.Globalization;
    using System.Linq;
    using System.Runtime.InteropServices;
    using System.Text.RegularExpressions;

    /// <summary>
    /// Collection of methods to convert values from one format to another format.
    /// </summary>
    public class Conversions
    {
        /// <summary>
        /// Parse a string containing a non-hex value and return the value.
        /// </summary>
        /// <param name="dataType">The type the string represents.</param>
        /// <param name="value">The string to convert.</param>
        /// <returns>The value converted from the given string.</returns>
        public static Object ParsePrimitiveStringAsPrimitive(ScannableType dataType, String value)
        {
            switch (dataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return Byte.Parse(value);
                case ScannableType type when type == ScannableType.Char:
                    return Byte.Parse(value);
                case ScannableType type when type == ScannableType.SByte:
                    return SByte.Parse(value);
                case ScannableType type when type == ScannableType.Int16:
                case ScannableType typeBE when typeBE == ScannableType.Int16BE:
                    return Int16.Parse(value);
                case ScannableType type when type == ScannableType.Int32:
                case ScannableType typeBE when typeBE == ScannableType.Int32BE:
                    return Int32.Parse(value);
                case ScannableType type when type == ScannableType.Int64:
                case ScannableType typeBE when typeBE == ScannableType.Int64BE:
                    return Int64.Parse(value);
                case ScannableType type when type == ScannableType.UInt16:
                case ScannableType typeBE when typeBE == ScannableType.UInt16BE:
                    return UInt16.Parse(value);
                case ScannableType type when type == ScannableType.UInt32:
                case ScannableType typeBE when typeBE == ScannableType.UInt32BE:
                    return UInt32.Parse(value);
                case ScannableType type when type == ScannableType.UInt64:
                case ScannableType typeBE when typeBE == ScannableType.UInt64BE:
                    return UInt64.Parse(value);
                case ScannableType type when type == ScannableType.Single:
                case ScannableType typeBE when typeBE == ScannableType.SingleBE:
                    return Single.Parse(value.EndsWith("f") ? value.Remove(value.LastIndexOf("f")) : value);
                case ScannableType type when type == ScannableType.Double:
                case ScannableType typeBE when typeBE == ScannableType.DoubleBE:
                    return Double.Parse(value);
                case ByteArrayType type:
                    return Conversions.ParseByteArrayString(value, false);
                case ScannableType type when type == ScannableType.String:
                    return value;
                case ScannableType type when type == ScannableType.IntPtr:
                    return !Environment.Is64BitProcess ? new IntPtr(Int32.Parse(value)) : new IntPtr(Int64.Parse(value));
                case ScannableType type when type == ScannableType.UIntPtr:
                    return !Environment.Is64BitProcess ? new UIntPtr(UInt32.Parse(value)) : new UIntPtr(UInt64.Parse(value));
                default:
                    return null;
            }
        }

        /// <summary>
        /// Converts a string containing hex characters to the given data type.
        /// </summary>
        /// <param name="dataType">The type to convert the parsed hex to.</param>
        /// <param name="value">The hex string to parse.</param>
        /// <returns>The converted value from the hex.</returns>
        public static Object ParseHexStringAsPrimitive(ScannableType dataType, String value)
        {
            return ParsePrimitiveStringAsPrimitive(dataType, ParseHexStringAsPrimitiveString(dataType, value));
        }

        /// <summary>
        /// Parses a raw value as a hex string.
        /// </summary>
        /// <param name="dataType">The data type of the value.</param>
        /// <param name="value">The raw value.</param>
        /// <param name="signHex">Whether to sign the hex value for signed interger types.</param>
        /// <returns>The converted hex string.</returns>
        public static String ParsePrimitiveAsHexString(ScannableType dataType, Object value, Boolean signHex = false)
        {
            return ParsePrimitiveStringAsHexString(dataType, value?.ToString(), signHex);
        }

        /// <summary>
        /// Converts a string containing dec characters to the hex equivalent for the given data type.
        /// </summary>
        /// <param name="dataType">The data type.</param>
        /// <param name="value">The hex string to parse.</param>
        /// <param name="signHex">Whether to sign the hex value for signed interger types.</param>
        /// <returns>The converted value from the hex.</returns>
        public static String ParsePrimitiveStringAsHexString(ScannableType dataType, String value, Boolean signHex = false)
        {
            Object realValue = ParsePrimitiveStringAsPrimitive(dataType, value);

            switch (dataType)
            {
                case ScannableType type when type == ScannableType.Byte || type == ScannableType.Char:
                    return (signHex && (Byte)realValue < 0) ? ("-" + (-(Byte)realValue).ToString("X")) : ((Byte)realValue).ToString("X");
                case ScannableType type when type == ScannableType.SByte:
                    return ((SByte)realValue).ToString("X");
                case ScannableType type when type == ScannableType.Int16:
                case ScannableType typeBE when typeBE == ScannableType.Int16BE:
                    return (signHex && (Int16)realValue < 0) ? ("-" + (-(Int16)realValue).ToString("X")) : ((Int16)realValue).ToString("X");
                case ScannableType type when type == ScannableType.Int32:
                case ScannableType typeBE when typeBE == ScannableType.Int32BE:
                    return (signHex && (Int32)realValue < 0) ? ("-" + (-(Int32)realValue).ToString("X")) : ((Int32)realValue).ToString("X");
                case ScannableType type when type == ScannableType.Int64:
                case ScannableType typeBE when typeBE == ScannableType.Int64BE:
                    return (signHex && (Int64)realValue < 0) ? ("-" + (-(Int64)realValue).ToString("X")) : ((Int64)realValue).ToString("X");
                case ScannableType type when type == ScannableType.UInt16:
                case ScannableType typeBE when typeBE == ScannableType.UInt16BE:
                    return ((UInt16)realValue).ToString("X");
                case ScannableType type when type == ScannableType.UInt32:
                case ScannableType typeBE when typeBE == ScannableType.UInt32BE:
                    return ((UInt32)realValue).ToString("X");
                case ScannableType type when type == ScannableType.UInt64:
                case ScannableType typeBE when typeBE == ScannableType.UInt64BE:
                    return ((UInt64)realValue).ToString("X");
                case ScannableType type when type == ScannableType.Single:
                case ScannableType typeBE when typeBE == ScannableType.SingleBE:
                    return BitConverter.ToUInt32(BitConverter.GetBytes((Single)realValue), 0).ToString("X");
                case ScannableType type when type == ScannableType.Double:
                case ScannableType typeBE when typeBE == ScannableType.DoubleBE:
                    return BitConverter.ToUInt64(BitConverter.GetBytes((Double)realValue), 0).ToString("X");
                case ScannableType type when type == ScannableType.String:
                    return String.Join(' ', Conversions.ParseByteArrayString(value, true, true).Select(str => Conversions.ParsePrimitiveStringAsHexString(ScannableType.Byte, str.ToString(), signHex)));
                case ScannableType type when type == ScannableType.IntPtr:
                    return ((IntPtr)realValue).ToString("X");
                case ScannableType type when type == ScannableType.UIntPtr:
                    return ((UIntPtr)realValue).ToIntPtr().ToString("X");
                default:
                    return null;
            }
        }

        /// <summary>
        /// Converts a string containing hex characters to the dec equivalent for the given data type.
        /// </summary>
        /// <param name="dataType">The data type.</param>
        /// <param name="value">The dec string to parse.</param>
        /// <returns>The converted value from the dec.</returns>
        public static String ParseHexStringAsPrimitiveString(ScannableType dataType, String value)
        {
            Boolean signedHex = false;

            switch (dataType)
            {
                case ScannableType type when type == ScannableType.Byte || type == ScannableType.Int16 || type == ScannableType.Int32 || type == ScannableType.Int64:
                    if (value.StartsWith("-"))
                    {
                        value = value.Substring(1);
                        signedHex = true;
                    }

                    break;
                default:
                    break;
            }

            UInt64 realValue = Conversions.AddressToValue(value);

            // Negate the parsed value if the hex string is signed
            if (signedHex)
            {
                realValue = (-realValue.ToInt64()).ToUInt64();
            }

            switch (dataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return realValue.ToString();
                case ScannableType type when type == ScannableType.Char:
                    return realValue.ToString();
                case ScannableType type when type == ScannableType.SByte:
                    return unchecked((SByte)realValue).ToString();
                case ScannableType type when type == ScannableType.Int16:
                case ScannableType typeBE when typeBE == ScannableType.Int16BE:
                    return unchecked((Int16)realValue).ToString();
                case ScannableType type when type == ScannableType.Int32:
                case ScannableType typeBE when typeBE == ScannableType.Int32BE:
                    return unchecked((Int32)realValue).ToString();
                case ScannableType type when type == ScannableType.Int64:
                case ScannableType typeBE when typeBE == ScannableType.Int64BE:
                    return unchecked((Int64)realValue).ToString();
                case ScannableType type when type == ScannableType.UInt16:
                case ScannableType typeBE when typeBE == ScannableType.UInt16BE:
                    return realValue.ToString();
                case ScannableType type when type == ScannableType.UInt32:
                case ScannableType typeBE when typeBE == ScannableType.UInt32BE:
                    return realValue.ToString();
                case ScannableType type when type == ScannableType.UInt64:
                case ScannableType typeBE when typeBE == ScannableType.UInt64BE:
                    return realValue.ToString();
                case ScannableType type when type == ScannableType.Single:
                case ScannableType typeBE when typeBE == ScannableType.SingleBE:
                    return BitConverter.ToSingle(BitConverter.GetBytes(unchecked((UInt32)realValue)), 0).ToString();
                case ScannableType type when type == ScannableType.Double:
                case ScannableType typeBE when typeBE == ScannableType.DoubleBE:
                    return BitConverter.ToDouble(BitConverter.GetBytes(realValue), 0).ToString();
                case ByteArrayType type:
                    return String.Join(' ', Conversions.ParseByteArrayString(value, true, true).Select(x => x.ToString()));
                case ScannableType type when type == ScannableType.IntPtr:
                    return ((IntPtr)realValue).ToString();
                case ScannableType type when type == ScannableType.UIntPtr:
                    return ((UIntPtr)realValue).ToIntPtr().ToString();
                default:
                    return null;
            }
        }

        /// <summary>
        /// Converts a given value to hex.
        /// </summary>
        /// <typeparam name="T">The data type of the value being converted.</typeparam>
        /// <param name="value">The value to convert.</param>
        /// <param name="formatAsAddress">Whether to use a zero padded address format.</param>
        /// <param name="includePrefix">Whether to include the '0x' hex prefix.</param>
        /// <returns>The value converted to hex.</returns>
        public static String ToHex<T>(T value, Boolean formatAsAddress = true, Boolean includePrefix = false)
        {
            Type dataType = value.GetType();

            // If a pointer type, parse as a long integer
            if (dataType == ScannableType.IntPtr)
            {
                dataType = ScannableType.Int64;
            }
            else if (dataType == ScannableType.UIntPtr)
            {
                dataType = ScannableType.UInt64;
            }

            String result = Conversions.ParsePrimitiveStringAsHexString(dataType, value.ToString());

            if (formatAsAddress)
            {
                if (result.Length <= 8)
                {
                    result = result.PadLeft(8, '0');
                }
                else
                {
                    result = result.PadLeft(16, '0');
                }
            }

            if (includePrefix)
            {
                result = "0x" + result;
            }

            return result;
        }

        /// <summary>
        /// Converts an address string to a raw value.
        /// </summary>
        /// <param name="address">The address hex string.</param>
        /// <returns>The raw value as a <see cref="UInt64"/></returns>
        public static UInt64 AddressToValue(String address)
        {
            if (String.IsNullOrEmpty(address))
            {
                return 0;
            }

            if (address.StartsWith("0x", StringComparison.OrdinalIgnoreCase))
            {
                address = address.Substring("0x".Length);
            }

            address = address.TrimStart('0');

            if (String.IsNullOrEmpty(address))
            {
                return 0;
            }

            UInt64 result;
            UInt64.TryParse(address, NumberStyles.HexNumber, CultureInfo.InvariantCulture, out result);

            return result;
        }

        /// <summary>
        /// Gets the size of the given data type.
        /// </summary>
        /// <param name="dataType">The data type.</param>
        /// <returns>The size of the given type.</returns>
        public static Int32 SizeOf(ScannableType dataType)
        {
            switch (dataType)
            {
                case ScannableType type when type == ScannableType.Byte:
                    return sizeof(Byte);
                case ScannableType type when type == ScannableType.Char:
                    return sizeof(Char);
                case ScannableType type when type == ScannableType.SByte:
                    return sizeof(SByte);
                case ScannableType type when type == ScannableType.Int16:
                case ScannableType typeBE when typeBE == ScannableType.Int16BE:
                    return sizeof(Int16);
                case ScannableType type when type == ScannableType.Int32:
                case ScannableType typeBE when typeBE == ScannableType.Int32BE:
                    return sizeof(Int32);
                case ScannableType type when type == ScannableType.Int64:
                case ScannableType typeBE when typeBE == ScannableType.Int64BE:
                    return sizeof(Int64);
                case ScannableType type when type == ScannableType.UInt16:
                case ScannableType typeBE when typeBE == ScannableType.UInt16BE:
                    return sizeof(UInt16);
                case ScannableType type when type == ScannableType.UInt32:
                case ScannableType typeBE when typeBE == ScannableType.UInt32BE:
                    return sizeof(UInt32);
                case ScannableType type when type == ScannableType.UInt64:
                case ScannableType typeBE when typeBE == ScannableType.UInt64BE:
                    return sizeof(UInt64);
                case ScannableType type when type == ScannableType.Single:
                case ScannableType typeBE when typeBE == ScannableType.SingleBE:
                    return sizeof(Single);
                case ScannableType type when type == ScannableType.Double:
                case ScannableType typeBE when typeBE == ScannableType.DoubleBE:
                    return sizeof(Double);
                case ByteArrayType type:
                    return type.Length;
                default:
                    return Marshal.SizeOf(dataType);
            }
        }

        /// <summary>
        /// Converts an array of bytes to an object.
        /// </summary>
        /// <typeparam name="T">The data type of the object.</typeparam>
        /// <param name="byteArray">The array of bytes.</param>
        /// <returns>The converted object.</returns>
        /// <exception cref="ArgumentException">If unable to handle the conversion.</exception>
        public static T BytesToObject<T>(Byte[] byteArray)
        {
            ScannableType dataType = typeof(T);

            switch (dataType)
            {
                case ScannableType type when type == ScannableType.Boolean:
                    return (T)(Object)BitConverter.ToBoolean(byteArray, 0);
                case ScannableType type when type == ScannableType.Byte:
                    return (T)(Object)byteArray[0];
                case ScannableType type when type == ScannableType.Char:
                    return (T)(Object)BitConverter.ToChar(byteArray, 0);
                case ScannableType type when type == ScannableType.Int16:
                    return (T)(Object)BitConverter.ToInt16(byteArray, 0);
                case ScannableType type when type == ScannableType.Int32:
                    return (T)(Object)BitConverter.ToInt32(byteArray, 0);
                case ScannableType type when type == ScannableType.Int64:
                    return (T)(Object)BitConverter.ToInt64(byteArray, 0);
                case ScannableType type when type == ScannableType.SByte:
                    return (T)(Object)unchecked((SByte)byteArray[0]);
                case ScannableType type when type == ScannableType.UInt16:
                    return (T)(Object)BitConverter.ToUInt16(byteArray, 0);
                case ScannableType type when type == ScannableType.UInt32:
                    return (T)(Object)BitConverter.ToUInt32(byteArray, 0);
                case ScannableType type when type == ScannableType.UInt64:
                    return (T)(Object)BitConverter.ToUInt64(byteArray, 0);
                case ScannableType type when type == ScannableType.Single:
                    return (T)(Object)BitConverter.ToSingle(byteArray, 0);
                case ScannableType type when type == ScannableType.Double:
                    return (T)(Object)BitConverter.ToDouble(byteArray, 0);
                case ScannableType type when type == ScannableType.Int16BE:
                    return (T)(Object)BinaryPrimitives.ReverseEndianness(BitConverter.ToInt16(byteArray, 0));
                case ScannableType type when type == ScannableType.Int32BE:
                    return (T)(Object)BinaryPrimitives.ReverseEndianness(BitConverter.ToInt32(byteArray, 0));
                case ScannableType type when type == ScannableType.Int64:
                    return (T)(Object)BinaryPrimitives.ReverseEndianness(BitConverter.ToInt64(byteArray, 0));
                case ScannableType type when type == ScannableType.UInt16BE:
                    return (T)(Object)BinaryPrimitives.ReverseEndianness(BitConverter.ToUInt16(byteArray, 0));
                case ScannableType type when type == ScannableType.UInt32BE:
                    return (T)(Object)BinaryPrimitives.ReverseEndianness(BitConverter.ToUInt32(byteArray, 0));
                case ScannableType type when type == ScannableType.UInt64BE:
                    return (T)(Object)BinaryPrimitives.ReverseEndianness(BitConverter.ToUInt64(byteArray, 0));
                case ScannableType type when type == ScannableType.SingleBE:
                    return (T)(Object)BitConverter.Int32BitsToSingle(BinaryPrimitives.ReverseEndianness(BitConverter.ToInt32(byteArray, 0)));
                case ScannableType type when type == ScannableType.DoubleBE:
                    return (T)(Object)BitConverter.Int64BitsToDouble(BinaryPrimitives.ReverseEndianness(BitConverter.ToInt64(byteArray, 0)));
                default:
                    throw new ArgumentException("Invalid type provided");
            }
        }

        /// <summary>
        /// Gets the name of the specified type
        /// </summary>
        /// <param name="dataType">The type from which to get the name.</param>
        /// <returns>The name of the type.</returns>
        public static String DataTypeToName(ScannableType dataType)
        {
            switch (dataType)
            {
                case ScannableType type when type == ScannableType.Boolean:
                    return "Boolean";
                case ScannableType type when type == ScannableType.Byte:
                    return "Byte";
                case ScannableType type when type == ScannableType.Char:
                    return "Char";
                case ScannableType type when type == ScannableType.SByte:
                    return "SByte";
                case ScannableType type when type == ScannableType.Int16:
                    return "Int16";
                case ScannableType type when type == ScannableType.Int32:
                    return "Int32";
                case ScannableType type when type == ScannableType.Int64:
                    return "Int64";
                case ScannableType type when type == ScannableType.UInt16:
                    return "UInt16";
                case ScannableType type when type == ScannableType.UInt32:
                    return "UInt32";
                case ScannableType type when type == ScannableType.UInt64:
                    return "UInt64";
                case ScannableType type when type == ScannableType.Single:
                    return "Single";
                case ScannableType type when type == ScannableType.Double:
                    return "Double";
                case ScannableType type when type == ScannableType.Int16BE:
                    return "Int16 (BE)";
                case ScannableType type when type == ScannableType.Int32BE:
                    return "Int32 (BE)";
                case ScannableType type when type == ScannableType.Int64BE:
                    return "Int64 (BE)";
                case ScannableType type when type == ScannableType.UInt16BE:
                    return "UInt16 (BE)";
                case ScannableType type when type == ScannableType.UInt32BE:
                    return "UInt32 (BE)";
                case ScannableType type when type == ScannableType.UInt64BE:
                    return "UInt64 (BE)";
                case ScannableType type when type == ScannableType.SingleBE:
                    return "Single (BE)";
                case ScannableType type when type == ScannableType.DoubleBE:
                    return "Double (BE)";
                case ByteArrayType _:
                    return "Byte[ ]";
                case ScannableType type when type == ScannableType.String:
                    return "String";
                default:
                    return "Unknown Type";
            }
        }

        /// <summary>
        /// Parses a string representation of an array of bytes into an actual byte array.
        /// </summary>
        /// <param name="value">The array of bytes string representation.</param>
        /// <param name="isHex">A value indicating whether the array of bytes string has hex values.</param>
        /// <param name="filterMasks">A value indicating whether mask values (?, *, x) should be replaced with a value of 0.</param>
        /// <returns></returns>
        public static Byte[] ParseByteArrayString(String value, Boolean isHex = true, Boolean filterMasks = false)
        {
            if (isHex && filterMasks)
            {
                Regex wildcardRegex = new Regex("[?*x]");
                value = wildcardRegex.Replace(value, "0");
            }

            IEnumerable<String> byteStrings = SplitByteArrayString(value, isHex);
            Byte[] result = byteStrings.Select(str => Byte.Parse(str, isHex ? NumberStyles.HexNumber : NumberStyles.Integer)).ToArray();

            return result;
        }

        /// <summary>
        /// Creates a byte array mask from the given array of bytes hex string. All ?, *, and x characters will be mapped to the byte 0xF, and all others are mapped to 0x0.
        /// </summary>
        /// <param name="value">An array of bytes hex string.</param>
        /// <returns>The array of bytes wildcard mask.</returns>
        public static Byte[] ParseByteArrayWildcardMask(String value)
        {
            Regex hexRegex = new Regex("[a-fA-F0-9]");
            Regex wildcardRegex = new Regex("[?*x]");
            value = hexRegex.Replace(value, "0");
            value = wildcardRegex.Replace(value, "F");

            IEnumerable<String> byteStrings = SplitByteArrayString(value, true);
            Byte[] result = byteStrings.Select(str => Byte.Parse(str, NumberStyles.HexNumber)).ToArray();

            return result;
        }

        /// <summary>
        /// Partitions an array of bytes string into an enumerable collection of strings, each of which contains a string representation of a single byte.
        /// </summary>
        /// <param name="value">The array of byte string to parse.</param>
        /// <param name="isHex">A value indicating whether the elements of the provided array are represented in hex.</param>
        /// <returns>An enumerable collection of strings, each of which contains a string representation of a single byte.</returns>
        public static IEnumerable<String> SplitByteArrayString(String value, Boolean isHex = true)
        {
            // First split on whitespace, which has priority for separating bytes
            String[] values = value?.Split(' ', StringSplitOptions.RemoveEmptyEntries) ?? new String[0];

            // Next group bytes into chunks of two from left to right
            if (isHex)
            {
                Int32 index = 0;
                return values.Select(str => str
                    .ToLookup(character => index++ / 2)
                    .Select(lookup => new String(lookup.ToArray())))
                        .SelectMany(str => str);
            }
            else
            {
                return values;
            }
        }

        /// <summary>
        /// Converts a given value into a metric information storage size (ie KB, MB, GB, TB, etc.)
        /// </summary>
        /// <param name="value">A value representing the number of bytes.</param>
        /// <returns>The value as its corresponding metric size string.</returns>
        public static String ValueToMetricSize(UInt64 value)
        {
            // Note: UInt64s run out around EB
            String[] suffix = { "B", "KB", "MB", "GB", "TB", "PB", "EB" };

            if (value == 0)
            {
                return "0" + suffix[0];
            }

            Int32 place = Convert.ToInt32(Math.Floor(Math.Log(value, 1024)));
            Double number = Math.Round(value / Math.Pow(1024, place), 1);

            return number.ToString() + suffix[place];
        }
    }
    //// End class
}
//// End namespace