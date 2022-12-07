namespace Squalr.Source.Mvvm.Converters
{
    using System;
    using System.Globalization;
    using System.Windows;
    using System.Windows.Controls;
    using System.Windows.Data;
    using System.Windows.Media;
    using System.Windows.Shapes;

    /// <summary>
    /// BorderGapMaskConverter class
    /// </summary>
    public class BorderGapMaskConverter : IMultiValueConverter
    {
        /// <summary>
        /// Convert a value.
        /// </summary>
        /// <param name="values">values as produced by source binding</param>
        /// <param name="targetType">target type</param>
        /// <param name="parameter">converter parameter</param>
        /// <param name="culture">culture information</param>
        /// <returns>
        /// Converted value.
        /// Visual Brush that is used as the opacity mask for the Border
        /// in the style for GroupBox.
        /// </returns>
        public Object Convert(Object[] values, Type targetType, Object parameter, CultureInfo culture)
        {
            Type doubleType = typeof(Double);

            if (parameter == null ||
                values == null ||
                values.Length != 3 ||
                values[0] == null ||
                values[1] == null ||
                values[2] == null ||
                !doubleType.IsAssignableFrom(values[0].GetType()) ||
                !doubleType.IsAssignableFrom(values[1].GetType()) ||
                !doubleType.IsAssignableFrom(values[2].GetType()))
            {
                return DependencyProperty.UnsetValue;
            }

            Type paramType = parameter.GetType();
            if (!(doubleType.IsAssignableFrom(paramType) || typeof(string).IsAssignableFrom(paramType)))
            {
                return DependencyProperty.UnsetValue;
            }

            Double headerWidth = (Double)values[0];
            Double borderWidth = (Double)values[1];
            Double borderHeight = (Double)values[2];

            // Doesn't make sense to have a Grid
            // with 0 as width or height
            if (borderWidth == 0 || borderHeight == 0)
            {
                return null;
            }

            // Width of the line to the left of the header
            // to be used to set the width of the first column of the Grid
            Double lineWidth;

            if (parameter is string)
            {
                lineWidth = Double.Parse((string)parameter, NumberFormatInfo.InvariantInfo);
            }
            else
            {
                lineWidth = (Double)parameter;
            }

            Grid grid = new Grid();
            grid.Width = borderWidth;
            grid.Height = borderHeight;
            ColumnDefinition colDef1 = new ColumnDefinition();
            ColumnDefinition colDef2 = new ColumnDefinition();
            ColumnDefinition colDef3 = new ColumnDefinition();
            colDef1.Width = new GridLength(lineWidth);
            colDef2.Width = new GridLength(headerWidth);
            colDef3.Width = new GridLength(1, GridUnitType.Star);
            grid.ColumnDefinitions.Add(colDef1);
            grid.ColumnDefinitions.Add(colDef2);
            grid.ColumnDefinitions.Add(colDef3);
            RowDefinition rowDef1 = new RowDefinition();
            RowDefinition rowDef2 = new RowDefinition();
            rowDef1.Height = new GridLength(borderHeight / 2);
            rowDef2.Height = new GridLength(1, GridUnitType.Star);
            grid.RowDefinitions.Add(rowDef1);
            grid.RowDefinitions.Add(rowDef2);

            Rectangle rectColumn1 = new Rectangle();
            Rectangle rectColumn2 = new Rectangle();
            Rectangle rectColumn3 = new Rectangle();
            rectColumn1.Fill = Brushes.Black;
            rectColumn2.Fill = Brushes.Black;
            rectColumn3.Fill = Brushes.Black;

            Grid.SetRowSpan(rectColumn1, 2);
            Grid.SetRow(rectColumn1, 0);
            Grid.SetColumn(rectColumn1, 0);

            Grid.SetRow(rectColumn2, 1);
            Grid.SetColumn(rectColumn2, 1);

            Grid.SetRowSpan(rectColumn3, 2);
            Grid.SetRow(rectColumn3, 0);
            Grid.SetColumn(rectColumn3, 2);

            grid.Children.Add(rectColumn1);
            grid.Children.Add(rectColumn2);
            grid.Children.Add(rectColumn3);

            return (new VisualBrush(grid));
        }

        /// <summary>
        /// Not Supported.
        /// </summary>
        /// <param name="value">value, as produced by target</param>
        /// <param name="targetTypes">target types</param>
        /// <param name="parameter">converter parameter</param>
        /// <param name="culture">culture information</param>
        /// <returns>An array containing a single DoNoting binding.</returns>
        public Object[] ConvertBack(Object value, Type[] targetTypes, Object parameter, CultureInfo culture)
        {
            return new Object[] { Binding.DoNothing };
        }
    }
}
