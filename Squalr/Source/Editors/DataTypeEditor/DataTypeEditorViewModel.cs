namespace Squalr.Source.Editors.DataTypeEditor
{
    using GalaSoft.MvvmLight;
    using GalaSoft.MvvmLight.Command;
    using Squalr.Engine.Common;
    using System;
    using System.Threading;
    using System.Threading.Tasks;
    using System.Windows.Input;

    /// <summary>
    /// View model for the Offset Editor.
    /// </summary>
    public class DataTypeEditorViewModel : ViewModelBase
    {
        /// <summary>
        /// Singleton instance of the <see cref="DataTypeEditorViewModel" /> class.
        /// </summary>
        private static readonly Lazy<DataTypeEditorViewModel> DataTypeEditorViewModelInstance = new Lazy<DataTypeEditorViewModel>(
                () => { return new DataTypeEditorViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// Initializes a new instance of the <see cref="DataTypeEditorViewModel" /> class.
        /// </summary>
        public DataTypeEditorViewModel()
        {
            this.UpdateDataTypeCommand = new RelayCommand<ScannableType>((dataType) => Task.Run(() => this.UpdateDataType(dataType)), (offset) => true);
        }

        /// <summary>
        /// Gets a command to update the active data type for the item being edited.
        /// </summary>
        public ICommand UpdateDataTypeCommand { get; private set; }

        /// <summary>
        /// Gets or sets the selected data type.
        /// </summary>
        public ScannableType DataType { get; set; }

        /// <summary>
        /// Gets a singleton instance of the <see cref="DataTypeEditorViewModel" /> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static DataTypeEditorViewModel GetInstance()
        {
            return DataTypeEditorViewModel.DataTypeEditorViewModelInstance.Value;
        }

        /// <summary>
        /// Updates the active data type.
        /// </summary>
        /// <param name="dataType">The new data type.</param>
        private void UpdateDataType(ScannableType dataType)
        {
            this.DataType = dataType;
            this.RaisePropertyChanged(nameof(this.DataType));
        }
    }
    //// End class
}
//// End namespace