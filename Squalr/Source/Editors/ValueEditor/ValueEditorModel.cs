﻿namespace Squalr.Source.Editors.ValueEditor
{
    using Squalr.Engine.Projects.Items;
    using System;
    using System.ComponentModel;
    using System.Drawing.Design;
    using System.Windows;

    /// <summary>
    /// Editor for project item values.
    /// </summary>
    public class ValueEditorModel : UITypeEditor
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="ValueEditorModel" /> class.
        /// </summary>
        public ValueEditorModel()
        {
        }

        /// <summary>
        /// Gets the editor style. This will be Modal, as it launches a custom editor.
        /// </summary>
        /// <param name="context">Type descriptor context.</param>
        /// <returns>Modal type editor.</returns>
        public override UITypeEditorEditStyle GetEditStyle(ITypeDescriptorContext context)
        {
            return UITypeEditorEditStyle.Modal;
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
            View.Editors.ValueEditor valueEditor = new View.Editors.ValueEditor(value as AddressItem) { Owner = Application.Current.MainWindow };

            if (valueEditor.ShowDialog() == true)
            {
                return valueEditor.ValueEditorViewModel.Value;
            }

            return (value as AddressItem)?.AddressValue;
        }
    }
    //// End class
}
//// End namespace