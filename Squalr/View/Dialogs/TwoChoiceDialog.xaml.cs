namespace Squalr.View.Dialogs
{
    using System;
    using System.Windows;

    /// <summary>
    /// Interaction logic for TwoChoiceDialog.xaml.
    /// </summary>
    public partial class TwoChoiceDialog : Window
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="TwoChoiceDialog" /> class.
        /// </summary>
        public TwoChoiceDialog()
        {
            this.InitializeComponent();
        }

        /// <summary>
        /// Invoked when the added offsets are canceled. Closes the view.
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="e">Event args.</param>
        private void CancelButtonClick(Object sender, RoutedEventArgs e)
        {
            this.DialogResult = false;
            this.Close();
        }

        /// <summary>
        /// Invoked when the added offsets are accepted. Closes the view.
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="e">Event args.</param>
        private void AcceptButtonClick(Object sender, RoutedEventArgs e)
        {
            this.DialogResult = true;
            this.Close();
        }
    }
    //// End class
}
//// End namespace