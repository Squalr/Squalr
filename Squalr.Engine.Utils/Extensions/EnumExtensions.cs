namespace Squalr.Engine.Common.Extensions
{
    using System;
    using System.ComponentModel;
    using System.Reflection;

    /// <summary>
    /// A class defining extension methods for enums.
    /// </summary>
    public static class EnumExtensions
    {
        /// <summary>
        /// Gets a description for the given enum value.
        /// </summary>
        /// <typeparam name="T">The enum type.</typeparam>
        /// <param name="enumerationValue">The enum value for which a description is returned.</param>
        /// <returns>The description of the given enum.</returns>
        /// <exception cref="ArgumentException">An exception that is thrown if the given type is not an enum.</exception>
        public static String GetDescription<T>(this T enumerationValue) where T : struct
        {
            Type type = enumerationValue.GetType();

            if (!type.IsEnum)
            {
                throw new ArgumentException("EnumerationValue must be of Enum type", "enumerationValue");
            }

            // Tries to find a DescriptionAttribute for a potential friendly name for the enum
            MemberInfo[] memberInfo = type.GetMember(enumerationValue.ToString());
            if (memberInfo != null && memberInfo.Length > 0)
            {
                Object[] attrs = memberInfo[0].GetCustomAttributes(typeof(DescriptionAttribute), false);

                if (attrs != null && attrs.Length > 0)
                {
                    // Pull out the description value
                    return ((DescriptionAttribute)attrs[0]).Description;
                }
            }

            // If we have no description attribute, just return the ToString of the enum
            return enumerationValue.ToString();
        }
    }
    //// End class
}
//// End namespace