﻿namespace Squalr.Engine.Common.Extensions
{
    using System;
    using System.Linq.Expressions;
    using System.Runtime.CompilerServices;

    /// <summary>
    /// Contains extension methods for integers.
    /// </summary>
    public static class IntegerExtensions
    {
        /// <summary>
        /// Converts the given integer to a <see cref="Int32"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="Int32"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Int32 ToInt32(this Int64 integer)
        {
            return unchecked((Int32)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="Int32"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="Int32"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Int32 ToInt32(this UInt64 integer)
        {
            return unchecked((Int32)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="Int32"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="Int32"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Int32 ToInt32(this UInt32 integer)
        {
            return unchecked((Int32)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="UInt32"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="UInt32"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static UInt32 ToUInt32(this Int64 integer)
        {
            return unchecked((UInt32)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="UInt32"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="UInt32"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static UInt32 ToUInt32(this UInt64 integer)
        {
            return unchecked((UInt32)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="UInt32"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="UInt32"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static UInt32 ToUInt32(this Int32 integer)
        {
            return unchecked((UInt32)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="UInt64"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="UInt64"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static UInt64 ToUInt64(this Int64 integer)
        {
            return unchecked((UInt64)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="UInt64"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="UInt64"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static UInt64 ToUInt64(this Int32 integer)
        {
            return unchecked((UInt64)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="UInt64"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="UInt64"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static UInt64 ToUInt64(this UInt32 integer)
        {
            return unchecked((UInt64)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="Int64"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="Int64"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Int64 ToInt64(this UInt64 integer)
        {
            return unchecked((Int64)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="Int64"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="Int64"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Int64 ToInt64(this Int32 integer)
        {
            return unchecked((Int64)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="Int64"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="Int64"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Int64 ToInt64(this UInt32 integer)
        {
            return unchecked((Int64)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="IntPtr"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="IntPtr"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static IntPtr ToIntPtr(this Int64 integer)
        {
            return unchecked((IntPtr)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="IntPtr"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="IntPtr"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static IntPtr ToIntPtr(this UInt64 integer)
        {
            return unchecked((IntPtr)(Int64)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="IntPtr"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="IntPtr"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static IntPtr ToIntPtr(this Int32 integer)
        {
            return unchecked((IntPtr)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="IntPtr"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="IntPtr"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static IntPtr ToIntPtr(this UInt32 integer)
        {
            return unchecked((IntPtr)(Int64)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="IntPtr"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="IntPtr"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static UIntPtr ToUIntPtr(this Int64 integer)
        {
            return unchecked((UIntPtr)(UInt64)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="IntPtr"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="IntPtr"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static UIntPtr ToUIntPtr(this UInt64 integer)
        {
            return unchecked((UIntPtr)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="IntPtr"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="IntPtr"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static UIntPtr ToUIntPtr(this Int32 integer)
        {
            return unchecked((UIntPtr)(UInt64)integer);
        }

        /// <summary>
        /// Converts the given integer to a <see cref="IntPtr"/>.
        /// </summary>
        /// <param name="integer">The integer to convert.</param>
        /// <returns>The integer converted to a <see cref="IntPtr"/>.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static UIntPtr ToUIntPtr(this UInt32 integer)
        {
            return unchecked((UIntPtr)integer);
        }

        /// <summary>
        /// Performs the addition operation with the given values.
        /// </summary>
        /// <param name="left">The left side value.</param>
        /// <param name="right">The right side value.</param>
        /// <param name="wrapAround">Whether or not values will wrap if the operation overflows. Otherwise, cap out at Int64.MaxValue or Int64.MinValue.</param>
        /// <returns>The result of the operation.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Int64 Add(this Int64 left, dynamic right, Boolean wrapAround = true)
        {
            return IntegerExtensions.DoOperation(left.ToUInt64(), right, ExpressionType.Add, wrapAround).ToInt64();
        }

        /// <summary>
        /// Performs the addition operation with the given values.
        /// </summary>
        /// <param name="left">The left side value.</param>
        /// <param name="right">The right side value.</param>
        /// <param name="wrapAround">Whether or not values will wrap if the operation overflows. Otherwise, cap out at Int64.MaxValue or Int64.MinValue.</param>
        /// <returns>The result of the operation.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static UInt64 Add(this UInt64 left, dynamic right, Boolean wrapAround = true)
        {
            return IntegerExtensions.DoOperation(left, right, ExpressionType.Add, wrapAround);
        }

        /// <summary>
        /// Performs the subtraction operation with the given values.
        /// </summary>
        /// <param name="left">The left side value.</param>
        /// <param name="right">The right side value.</param>
        /// <param name="wrapAround">Whether or not values will wrap if the operation overflows. Otherwise, cap out at Int64.MaxValue or Int64.MinValue.</param>
        /// <returns>The result of the operation.</returns>
        public static Int64 Subtract(this Int64 left, dynamic right, Boolean wrapAround = true)
        {
            return IntegerExtensions.DoOperation(left.ToUInt64(), right, ExpressionType.Subtract, wrapAround).ToInt64();
        }

        /// <summary>
        /// Performs the subtraction operation with the given values.
        /// </summary>
        /// <param name="left">The left side value.</param>
        /// <param name="right">The right side value.</param>
        /// <param name="wrapAround">Whether or not values will wrap if the operation overflows. Otherwise, cap out at Int64.MaxValue or Int64.MinValue.</param>
        /// <returns>The result of the operation.</returns>
        public static UInt64 Subtract(this UInt64 left, dynamic right, Boolean wrapAround = true)
        {
            return IntegerExtensions.DoOperation(left, right, ExpressionType.Subtract, wrapAround);
        }

        /// <summary>
        /// Performs the multiplication operation with the given values.
        /// </summary>
        /// <param name="left">The left side value.</param>
        /// <param name="right">The right side value.</param>
        /// <param name="wrapAround">Whether or not values will wrap if the operation overflows. Otherwise, cap out at Int64.MaxValue or Int64.MinValue.</param>
        /// <returns>The result of the operation.</returns>
        public static Int64 Multiply(this Int64 left, dynamic right, Boolean wrapAround = true)
        {
            return IntegerExtensions.DoOperation(left.ToUInt64(), right, ExpressionType.Multiply, wrapAround).ToInt64();
        }

        /// <summary>
        /// Performs the multiplication operation with the given values.
        /// </summary>
        /// <param name="left">The left side value.</param>
        /// <param name="right">The right side value.</param>
        /// <param name="wrapAround">Whether or not values will wrap if the operation overflows. Otherwise, cap out at Int64.MaxValue or Int64.MinValue.</param>
        /// <returns>The result of the operation.</returns>
        public static UInt64 Multiply(this UInt64 left, dynamic right, Boolean wrapAround = true)
        {
            return IntegerExtensions.DoOperation(left, right, ExpressionType.Multiply, wrapAround);
        }

        /// <summary>
        /// Performs the division operation with the given values.
        /// </summary>
        /// <param name="left">The left side value.</param>
        /// <param name="right">The right side value.</param>
        /// <returns>The result of the operation.</returns>
        public static Int64 Divide(this Int64 left, dynamic right)
        {
            return IntegerExtensions.DoOperation(left.ToUInt64(), right, ExpressionType.Multiply).ToInt64();
        }

        /// <summary>
        /// Performs the division operation with the given values.
        /// </summary>
        /// <param name="left">The left side value.</param>
        /// <param name="right">The right side value.</param>
        /// <returns>The result of the operation.</returns>
        public static UInt64 Divide(this UInt64 left, dynamic right)
        {
            return IntegerExtensions.DoOperation(left, right, ExpressionType.Multiply);
        }

        /// <summary>
        /// Performs the modulo operation with the given values.
        /// </summary>
        /// <param name="left">The left side value.</param>
        /// <param name="right">The right side value.</param>
        /// <returns>The result of the operation.</returns>
        public static Int64 Mod(this Int64 left, dynamic right)
        {
            return IntegerExtensions.DoOperation(left.ToUInt64(), right, ExpressionType.Modulo).ToInt64();
        }

        /// <summary>
        /// Performs the modulo operation with the given values.
        /// </summary>
        /// <param name="left">The left side value.</param>
        /// <param name="right">The right side value.</param>
        /// <returns>The result of the operation.</returns>
        public static UInt64 Mod(this UInt64 left, dynamic right)
        {
            return IntegerExtensions.DoOperation(left, right, ExpressionType.Modulo);
        }

        /// <summary>
        /// Extracts a single bit from an integer.
        /// </summary>
        /// <param name="value">The value from which a bit is extracted.</param>
        /// <param name="bit">The index of the bit to extract.</param>
        /// <returns>The extracted bit value as a boolean.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Boolean GetBit(this Int32 value, Int32 bit)
        {
            return (value & (1 << bit)) != 0;
        }

        /// <summary>
        /// Extracts a single bit from an integer.
        /// </summary>
        /// <param name="value">The value from which a bit is extracted.</param>
        /// <param name="bit">The index of the bit to extract.</param>
        /// <returns>The extracted bit value as a boolean.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Boolean GetBit(this Int64 value, Int32 bit)
        {
            return (value & (1 << bit)) != 0;
        }

        /// <summary>
        /// Extracts a single bit from an integer.
        /// </summary>
        /// <param name="value">The value from which a bit is extracted.</param>
        /// <param name="bit">The index of the bit to extract.</param>
        /// <returns>The extracted bit value as a boolean.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Boolean GetBit(this IntPtr value, Int32 bit)
        {
            return GetBit(value.ToInt64(), bit);
        }

        /// <summary>
        /// Extracts a range of bits from an integer.
        /// </summary>
        /// <param name="value">The value from which a bit is extracted.</param>
        /// <param name="bit">The bit index flags to extract.</param>
        /// <param name="length">The number of bits to extract</param>
        /// <returns>The extracted bit values as a new integer.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Int32 GetBits(this Int32 value, Int32 bit, Int32 length)
        {
            Int32 mask = (1 << length) - 1;

            return (value >> bit) & mask;
        }

        /// <summary>
        /// Extracts a range of bits from an integer.
        /// </summary>
        /// <param name="value">The value from which a bit is extracted.</param>
        /// <param name="bit">The bit index flags to extract.</param>
        /// <param name="length">The number of bits to extract</param>
        /// <returns>The extracted bit values as a new integer.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Int64 GetBits(this Int64 value, Int32 bit, Int32 length)
        {
            Int64 mask = (1L << length) - 1L;

            return (value >> bit) & mask;
        }

        /// <summary>
        /// Extracts a range of bits from an integer.
        /// </summary>
        /// <param name="value">The value from which a bit is extracted.</param>
        /// <param name="bit">The bit index flags to extract.</param>
        /// <param name="length">The number of bits to extract</param>
        /// <returns>The extracted bit values as a new integer.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Int64 GetBits(this IntPtr value, Int32 bit, Int32 length)
        {
            return GetBits(value.ToInt64(), bit, length);
        }

        /// <summary>
        /// Performs the given mathematical operation on the given left and right values.
        /// </summary>
        /// <param name="left">The left side value.</param>
        /// <param name="rightDynamic">The right side value.</param>
        /// <param name="expression">The mathematical operation to perform.</param>
        /// <param name="wrapAround">Whether or not values will wrap if the operation overflows. Otherwise, cap out at Int64.MaxValue or Int64.MinValue.</param>
        /// <returns>The result of the operation.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private static UInt32 DoOperation(UInt32 left, dynamic rightDynamic, ExpressionType expression, Boolean wrapAround = true)
        {
            UInt32 right;

            if (wrapAround)
            {
                right = (UInt32)rightDynamic;
            }
            else
            {
                right = unchecked((UInt32)rightDynamic);
            }

            try
            {
                switch (expression)
                {
                    case ExpressionType.Add:
                        return wrapAround ? unchecked(left + right) : checked(left + right);
                    case ExpressionType.Subtract:
                        return wrapAround ? unchecked(left - right) : checked(left - right);
                    case ExpressionType.Multiply:
                        return wrapAround ? unchecked(left * right) : checked(left * right);
                    case ExpressionType.Divide:
                        return unchecked(left / right);
                    case ExpressionType.Modulo:
                        return unchecked(left % right);
                    default:
                        throw new Exception("Unknown operation");
                }
            }
            catch (OverflowException ex)
            {
                switch (expression)
                {
                    case ExpressionType.Add:
                        return right >= 0 ? Int32.MaxValue.ToUInt32() : Int32.MinValue.ToUInt32();
                    case ExpressionType.Multiply:
                        return ((right >= 0 && left >= 0) || (right <= 0 && left <= 0)) ? Int32.MaxValue.ToUInt32() : Int32.MinValue.ToUInt32();
                    case ExpressionType.Subtract:
                        return right >= 0 ? Int32.MinValue.ToUInt32() : Int32.MaxValue.ToUInt32();
                    default:
                        throw ex;
                }
            }
        }

        /// <summary>
        /// Performs the given mathematical operation on the given left and right values.
        /// </summary>
        /// <param name="left">The left side value.</param>
        /// <param name="rightDynamic">The right side value.</param>
        /// <param name="expression">The mathematical operation to perform.</param>
        /// <param name="wrapAround">Whether or not values will wrap if the operation overflows. Otherwise, cap out at Int64.MaxValue or Int64.MinValue.</param>
        /// <returns>The result of the operation.</returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private static UInt64 DoOperation(UInt64 left, dynamic rightDynamic, ExpressionType expression, Boolean wrapAround = true)
        {
            UInt64 right;

            right = wrapAround ? (UInt64)rightDynamic : unchecked((UInt64)rightDynamic);

            try
            {
                switch (expression)
                {
                    case ExpressionType.Add:
                        return wrapAround ? unchecked(left + right) : checked(left + right);
                    case ExpressionType.Subtract:
                        return wrapAround ? unchecked(left - right) : checked(left - right);
                    case ExpressionType.Multiply:
                        return wrapAround ? unchecked(left * right) : checked(left * right);
                    case ExpressionType.Divide:
                        return unchecked(left / right);
                    case ExpressionType.Modulo:
                        return unchecked(left % right);
                    default:
                        throw new Exception("Unknown operation");
                }
            }
            catch (OverflowException ex)
            {
                switch (expression)
                {
                    case ExpressionType.Add:
                        return right >= 0 ? Int64.MaxValue.ToUInt64() : Int64.MinValue.ToUInt64();
                    case ExpressionType.Multiply:
                        return ((right >= 0 && left >= 0) || (right <= 0 && left <= 0)) ? Int64.MaxValue.ToUInt64() : Int64.MinValue.ToUInt64();
                    case ExpressionType.Subtract:
                        return right >= 0 ? Int64.MinValue.ToUInt64() : Int64.MaxValue.ToUInt64();
                    default:
                        throw ex;
                }
            }
        }
    }
    //// End class
}
//// End namespace