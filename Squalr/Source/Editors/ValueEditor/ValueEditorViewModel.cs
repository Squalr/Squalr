namespace Squalr.Source.Editors.ValueEditor
{
    using GalaSoft.MvvmLight;
    using Squalr.Engine.Projects.Items;
    using System;
    using System.Threading;
    using System.Windows;

    /// <summary>
    /// View model for the Value Editor.
    /// </summary>
    public class ValueEditorViewModel : ViewModelBase
    {
        /// <summary>
        /// Singleton instance of the <see cref="ValueEditorViewModel" /> class.
        /// </summary>
        private static Lazy<ValueEditorViewModel> valueEditorViewModelInstance = new Lazy<ValueEditorViewModel>(
                () => { return new ValueEditorViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// Initializes a new instance of the <see cref="ValueEditorViewModel" /> class.
        /// </summary>
        public ValueEditorViewModel()
        {
        }

        /// <summary>
        /// Gets or sets the value being edited.
        /// </summary>
        public Object Value { get; set; }

        /// <summary>
        /// Gets a singleton instance of the <see cref="ValueEditorViewModel" /> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static ValueEditorViewModel GetInstance()
        {
            return ValueEditorViewModel.valueEditorViewModelInstance.Value;
        }

        public void ShowDialog(AddressItem addressItem)
        {
            View.Editors.ValueEditor valueEditor = new View.Editors.ValueEditor(addressItem) { Owner = Application.Current.MainWindow };

            if (valueEditor.ShowDialog() == true && addressItem != null)
            {
                addressItem.AddressValue = this.Value;
            }
        }
    }
    //// End class
}
//// End namespace