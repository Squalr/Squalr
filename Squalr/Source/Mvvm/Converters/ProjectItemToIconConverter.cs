namespace Squalr.Source.Mvvm.Converters
{
    using Squalr.Content;
    using Squalr.Source.ProjectExplorer.ProjectItems;
    using System;
    using System.Globalization;
    using System.Linq;
    using System.Windows.Data;

    /// <summary>
    /// Converts ProjectItems to an icon format readily usable by the view.
    /// </summary>
    public class ProjectItemToIconConverter : IValueConverter
    {
        /// <summary>
        /// An instance of the <see cref="DataTypeToIconConverter"/> class.
        /// </summary>
        private static readonly DataTypeToIconConverter DataTypeToIconConverter = new DataTypeToIconConverter();

        /// <summary>
        /// Converts an Icon to a BitmapSource.
        /// </summary>
        /// <param name="value">Value to be converted.</param>
        /// <param name="targetType">Type to convert to.</param>
        /// <param name="parameter">Optional conversion parameter.</param>
        /// <param name="culture">Globalization info.</param>
        /// <returns>Object with type of BitmapSource. If conversion cannot take place, returns null.</returns>
        public Object Convert(Object value, Type targetType, Object parameter, CultureInfo culture)
        {
            if (parameter != null)
            {
                value = parameter;
            }

            switch (value)
            {
                case DirectoryItemView type when type is DirectoryItemView:
                    return Images.Open;
                case PointerItemView type when type is PointerItemView:
                    if (type.IsStatic)
                    {
                        return Images.LetterS;
                    }
                    else if ((type.PointerOffsets?.Count() ?? 0) > 0)
                    {
                        return Images.LetterP;
                    }

                    return DataTypeToIconConverter.Convert(type.DataType, targetType, parameter, culture);
                case ProjectItemView type when type is ScriptItemView:
                    return Images.Script;
                case ProjectItemView type when type is InstructionItemView:
                    return Images.Cpu;
                default:
                    return Images.CollectValues;
            }
        }

        /// <summary>
        /// Not used or implemented.
        /// </summary>
        /// <param name="value">Value to be converted.</param>
        /// <param name="targetType">Type to convert to.</param>
        /// <param name="parameter">Optional conversion parameter.</param>
        /// <param name="culture">Globalization info.</param>
        /// <returns>Throws see <see cref="NotImplementedException" />.</returns>
        public Object ConvertBack(Object value, Type targetType, Object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
    //// End class
}
//// End namespace