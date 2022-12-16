namespace Squalr.View.Dialogs
{
    using System;
    using System.Windows;
    using System.Windows.Input;

    /// <summary>
    /// Interaction logic for SelectProjectDialog.xaml.
    /// </summary>
    public partial class SelectProjectDialog : Window
    {
        /// <summary>
        /// The selected project name.
        /// </summary>
        private String projectName;

        /// <summary>
        /// Initializes a new instance of the <see cref="SelectProjectDialog" /> class.
        /// </summary>
        public SelectProjectDialog()
        {
            this.InitializeComponent();
        }

        /// <summary>
        /// Gets or sets the selected project name.
        /// </summary>
        public String ProjectName
        {
            get
            {
                return this.projectName;
            }

            set
            {
                this.projectName = value;
            }
        }

        /// <summary>
        /// Selects the given project by name.
        /// </summary>
        /// <param name="projectName">The name of the project to select.</param>
        public void SelectProject(String projectName)
        {
            this.ProjectName = projectName;
            this.DialogResult = true;
            this.Close();
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

        /// <summary>
        /// Invoked when the projects list view is double clicked (ie a project is selected).
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="e">Event args.</param>
        private void ProjectsListViewMouseDoubleClick(Object sender, MouseButtonEventArgs e)
        {
            this.DialogResult = true;
            this.Close();
        }
    }
    //// End class
}
//// End namespace