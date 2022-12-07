namespace Squalr.View
{
    using Squalr.Source.Controls;
    using Squalr.Source.ScanResults;
    using Squalr.Source.Scanning;
    using System;
    using System.ComponentModel;
    using System.Windows;

    /// <summary>
    /// Interaction logic for MainWindow.xaml.
    /// </summary>
    public partial class MainWindow : Window
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="MainWindow"/> class.
        /// </summary>
        public MainWindow()
        {
            this.InitializeComponent();

            this.ValueHexDecBoxViewModel = this.ValueHexDecBox.DataContext as HexDecBoxViewModel;
            this.ValueHexDecBoxViewModel.SupportsMask = true;
            this.ValueHexDecBoxViewModel.PropertyChanged += HexDecBoxViewModelPropertyChanged;

            ScanResultsViewModel.GetInstance().PropertyChanged += ScanResultsPropertyChanged;
        }

        private HexDecBoxViewModel ValueHexDecBoxViewModel { get; set; }

        private void ScanResultsPropertyChanged(Object sender, PropertyChangedEventArgs e)
        {
            if (e.PropertyName == nameof(ScanResultsViewModel.ActiveType))
            {
                ValueHexDecBoxViewModel.DataType = ScanResultsViewModel.GetInstance().ActiveType;
            }
        }

        private void HexDecBoxViewModelPropertyChanged(Object sender, PropertyChangedEventArgs e)
        {
            if (e.PropertyName == nameof(ValueHexDecBoxViewModel.Text))
            {
                ManualScannerViewModel.GetInstance().UpdateActiveValueCommand.Execute(this.ValueHexDecBoxViewModel.GetValue());
                ManualScannerViewModel.GetInstance().UpdateActiveArgsCommand.Execute(this.ValueHexDecBoxViewModel.GetMask());
            }
        }
    }
    //// End class
}
//// End namespace