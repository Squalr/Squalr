namespace Squalr.Engine.Memory.Windows
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Extensions;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Memory.Windows.Native;
    using System;
    using System.Buffers.Binary;
    using System.Diagnostics;
    using System.Text;
    using static Squalr.Engine.Memory.Windows.Native.Enumerations;

    /// <summary>
    /// Class for memory editing a remote process.
    /// </summary>
    internal class WindowsMemoryWriter : IMemoryWriter
    {
        private WindowsMemoryQuery windowsMemoryQuery = new WindowsMemoryQuery();

        /// <summary>
        /// Initializes a new instance of the <see cref="WindowsMemoryWriter"/> class.
        /// </summary>
        public WindowsMemoryWriter()
        {
        }

        /// <summary>
        /// Writes a value to memory in the opened process.
        /// </summary>
        /// <param name="elementType">The data type to write.</param>
        /// <param name="address">The address to write to.</param>
        /// <param name="value">The value to write.</param>
        public void Write(Process process, ScannableType elementType, UInt64 address, Object value)
        {
            Byte[] bytes;

            switch (elementType)
            {
                case ScannableType type when type == ScannableType.Byte || type == typeof(Boolean):
                    bytes = new Byte[] { (Byte)value };
                    break;
                case ScannableType type when type == ScannableType.SByte:
                    bytes = new Byte[] { unchecked((Byte)(SByte)value) };
                    break;
                case ScannableType type when type == ScannableType.Char:
                    bytes = Encoding.UTF8.GetBytes(new Char[] { (Char)value });
                    break;
                case ScannableType type when type == ScannableType.Int16:
                    bytes = BitConverter.GetBytes((Int16)value);
                    break;
                case ScannableType type when type == ScannableType.Int16BE:
                    bytes = BitConverter.GetBytes(BinaryPrimitives.ReverseEndianness((Int16)value));
                    break;
                case ScannableType type when type == ScannableType.Int32:
                    bytes = BitConverter.GetBytes((Int32)value);
                    break;
                case ScannableType type when type == ScannableType.Int32BE:
                    bytes = BitConverter.GetBytes(BinaryPrimitives.ReverseEndianness((Int32)value));
                    break;
                case ScannableType type when type == ScannableType.Int64:
                    bytes = BitConverter.GetBytes((Int64)value);
                    break;
                case ScannableType type when type == ScannableType.Int64BE:
                    bytes = BitConverter.GetBytes(BinaryPrimitives.ReverseEndianness((Int64)value));
                    break;
                case ScannableType type when type == ScannableType.UInt16:
                    bytes = BitConverter.GetBytes((UInt16)value);
                    break;
                case ScannableType type when type == ScannableType.UInt16BE:
                    bytes = BitConverter.GetBytes(BinaryPrimitives.ReverseEndianness((UInt16)value));
                    break;
                case ScannableType type when type == ScannableType.UInt32:
                    bytes = BitConverter.GetBytes((UInt32)value);
                    break;
                case ScannableType type when type == ScannableType.UInt32BE:
                    bytes = BitConverter.GetBytes(BinaryPrimitives.ReverseEndianness((UInt32)value));
                    break;
                case ScannableType type when type == ScannableType.UInt64:
                    bytes = BitConverter.GetBytes((UInt64)value);
                    break;
                case ScannableType type when type == ScannableType.UInt64BE:
                    bytes = BitConverter.GetBytes(BinaryPrimitives.ReverseEndianness((UInt64)value));
                    break;
                case ScannableType type when type == ScannableType.Single:
                    bytes = BitConverter.GetBytes((Single)value);
                    break;
                case ScannableType type when type == ScannableType.SingleBE:
                    bytes = BitConverter.GetBytes(BinaryPrimitives.ReverseEndianness(BitConverter.SingleToInt32Bits((Single)value)));
                    break;
                case ScannableType type when type == ScannableType.Double:
                    bytes = BitConverter.GetBytes((Double)value);
                    break;
                case ScannableType type when type == ScannableType.DoubleBE:
                    bytes = BitConverter.GetBytes(BinaryPrimitives.ReverseEndianness(BitConverter.DoubleToInt64Bits((Double)value)));
                    break;
                case ByteArrayType type:
                    bytes = (Byte[])value;
                    break;
                default:
                    throw new ArgumentException("Invalid type provided");
            }

            this.WriteBytes(process, address, bytes);
        }

        /// <summary>
        /// Writes the values of a specified type in the remote process.
        /// </summary>
        /// <typeparam name="T">The type of the value.</typeparam>
        /// <param name="address">The address where the value is written.</param>
        /// <param name="value">The value to write.</param>
        public void Write<T>(Process process, UInt64 address, T value)
        {
            this.Write(process, typeof(T), address, (Object)value);
        }

        /// <summary>
        /// Write an array of bytes in the remote process.
        /// </summary>
        /// <param name="address">The address where the array is written.</param>
        /// <param name="byteArray">The array of bytes to write.</param>
        public void WriteBytes(Process process, UInt64 address, Byte[] byteArray)
        {
            IntPtr processHandle = process == null ? IntPtr.Zero : process.Handle;

            MemoryProtectionFlags oldProtection = MemoryProtectionFlags.NoAccess;
            Int32 bytesWritten;

            try
            {
                Boolean isAddressWritable = this.windowsMemoryQuery.IsAddressWritable(process, address);

                // Make address writable if it is not so already
                if (!isAddressWritable)
                {
                    NativeMethods.VirtualProtectEx(processHandle, address.ToIntPtr(), byteArray.Length, MemoryProtectionFlags.ReadWrite, out oldProtection);
                }

                // Write the data to the target process
                if (NativeMethods.WriteProcessMemory(processHandle, address.ToIntPtr(), byteArray, byteArray.Length, out bytesWritten))
                {
                    if (bytesWritten != byteArray.Length)
                    {
                        Logger.Log(LogLevel.Error, "Error writing memory. Wrote " + bytesWritten + " bytes, but expected " + byteArray.Length);
                    }
                }

                // Restore old protection after doing the write, if it was previously unwritable
                if (!isAddressWritable)
                {
                    NativeMethods.VirtualProtectEx(processHandle, address.ToIntPtr(), byteArray.Length, oldProtection, out oldProtection);
                }
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error writing memory.", ex);
            }
        }

        /// <summary>
        /// Writes a string with a specified encoding in the remote process.
        /// </summary>
        /// <param name="address">The address where the string is written.</param>
        /// <param name="text">The text to write.</param>
        /// <param name="encoding">The encoding used.</param>
        public void WriteString(Process process, UInt64 address, String text, Encoding encoding)
        {
            // Write the text
            this.WriteBytes(process, address, encoding.GetBytes(text + '\0'));
        }
    }
    //// End class
}
//// End namespace