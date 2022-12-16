namespace Squalr.View.Dialogs
{
    using Squalr.Engine.Projects.Items;
    using Squalr.Source.Editors.RenameEditor;
    using System;
    using System.Windows;

    /// <summary>
    /// Interaction logic for RenameProjectItemDialog.xaml.
    /// </summary>
    public partial class RenameProjectItemDialog : Window
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="RenameProjectItemDialog" /> class.
        /// </summary>
        public RenameProjectItemDialog()
        {
            this.InitializeComponent();
        }

        /// <summary>
        /// Gets the view model associated with this view.
        /// </summary>
        public RenameProjectItemDialogViewModel RenameEditorViewModel
        {
            get
            {
                return this.DataContext as RenameProjectItemDialogViewModel;
            }
        }

        /// <summary>
        /// Invoked when the added offsets are canceled. Closes the view.
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="args">Event args.</param>
        private void CancelButtonClick(Object sender, RoutedEventArgs args)
        {
            this.DialogResult = false;
            this.Close();
        }

        /// <summary>
        /// Invoked when the added offsets are accepted. Closes the view.
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="args">Event args.</param>
        private void AcceptButtonClick(Object sender, RoutedEventArgs args)
        {
            this.DialogResult = true;
            this.Close();
        }

        /// <summary>
        /// Invoked when the exit file menu event executes. Closes the view.
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="args">Event args.</param>
        private void ExitFileMenuItemClick(Object sender, RoutedEventArgs args)
        {
            this.Close();
        }

        /// <summary>
        /// Event when this window has been loaded.
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="args">Event args.</param>
        private void OnWindowLoaded(Object sender, RoutedEventArgs args)
        {
            this.NameTextBox.Focus();
            this.NameTextBox.SelectAll();
        }
    }
    //// End class
}
//// End namespace