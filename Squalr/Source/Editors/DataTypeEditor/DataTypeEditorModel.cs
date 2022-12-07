namespace Squalr.Source.Editors.DataTypeEditor
{
    using System;
    using System.ComponentModel;
    using System.Drawing.Design;
    using System.Windows;

    /// <summary>
    /// Type editor for data types
    /// </summary>
    public class DataTypeEditorModel : UITypeEditor
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="DataTypeEditorModel" /> class.
        /// </summary>
        public DataTypeEditorModel()
        {
        }

        /// <summary>
        /// Gets the editor style. This will be Modal, as it launches a custom editor.
        /// </summary>
        /// <param name="context">Type descriptor context.</param>
        /// <returns>Modal type editor.</returns>
        public override UITypeEditorEditStyle GetEditStyle(ITypeDescriptorContext context)
        {
            return UITypeEditorEditStyle.DropDown;
        }

        /// <summary>
        /// Launches the editor for this type.
        /// </summary>
        /// <param name="context">Type descriptor context.</param>
        /// <param name="provider">Service provider.</param>
        /// <param name="value">The current value.</param>
        /// <returns>The updated values.</returns>
        public override Object EditValue(ITypeDescriptorContext context, IServiceProvider provider, Object value)
        {
            System.Drawing.Point cursorPosition = System.Windows.Forms.Cursor.Position;
            View.Editors.DataTypeEditor dataTypeEditor = new View.Editors.DataTypeEditor() { Owner = Application.Current.MainWindow, Left = cursorPosition.X, Top = cursorPosition.Y };

            // Reposition window to open towards the left
            dataTypeEditor.Left = Math.Max(0, dataTypeEditor.Left - dataTypeEditor.Width);

            if (dataTypeEditor.ShowDialog() == true)
            {
                return dataTypeEditor.DataTypeEditorViewModel.DataType;
            }

            return value;
        }
    }
    //// End class
}
//// End namespace