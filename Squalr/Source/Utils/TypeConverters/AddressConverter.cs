namespace Squalr.Source.Utils.TypeConverters
{
    using Squalr.Engine.Common;
    using System;
    using System.ComponentModel;
    using System.Globalization;
    using System.Text.RegularExpressions;

    /// <summary>
    /// Address type converter for use in the property viewer.
    /// </summary>
    public class AddressConverter : TypeConverter
    {
        /// <summary>
        /// Converts a value to an address.
        /// </summary>
        /// <param name="context">Type descriptor context.</param>
        /// <param name="culture">Globalization info.</param>
        /// <param name="value">The value being converted.</param>
        /// <param name="destinationType">The target type to convert to.</param>
        /// <returns>The converted value.</returns>
        public override Object ConvertTo(ITypeDescriptorContext context, CultureInfo culture, Object value, Type destinationType)
        {
            if (SyntaxChecker.CanParseValue(ScannableType.UInt64, value?.ToString()))
            {
                return Conversions.ToHex(Conversions.ParsePrimitiveStringAsPrimitive(ScannableType.UInt64, value?.ToString()), formatAsAddress: true, includePrefix: false);
            }

            return base.ConvertTo(context, culture, value, destinationType);
        }

        /// <summary>
        /// Converts an address string to the corresponding value.
        /// </summary>
        /// <param name="context">Type descriptor context.</param>
        /// <param name="culture">Globalization info.</param>
        /// <param name="value">The value being converted.</param>
        /// <returns>The converted value.</returns>
        public override Object ConvertFrom(ITypeDescriptorContext context, CultureInfo culture, Object value)
        {
            String valueAsStr = value?.ToString() ?? String.Empty;
            valueAsStr = Regex.Replace(valueAsStr, @"\s+", String.Empty);

            if (SyntaxChecker.CanParseAddress(valueAsStr))
            {
                return Conversions.AddressToValue(valueAsStr);
            }

            return null;
        }

        /// <summary>
        /// Determines if this converter can convert to the given source type.
        /// </summary>
        /// <param name="context">Type descriptor context.</param>
        /// <param name="sourceType">The source type.</param>
        /// <returns>True if this converter can convert to the given type.</returns>
        public override Boolean CanConvertTo(ITypeDescriptorContext context, Type sourceType)
        {
            return sourceType == ScannableType.IntPtr || sourceType == ScannableType.UInt64;
        }

        /// <summary>
        /// Determines if this converter can convert from the given source type.
        /// </summary>
        /// <param name="context">Type descriptor context.</param>
        /// <param name="sourceType">The source type.</param>
        /// <returns>True if this converter can convert from the given type.</returns>
        public override Boolean CanConvertFrom(ITypeDescriptorContext context, Type sourceType)
        {
            return sourceType == typeof(String);
        }
    }
    //// End class
}
//// End namespace