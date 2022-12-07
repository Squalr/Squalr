namespace Squalr.Source.ProjectExplorer.Dialogs
{
    using GalaSoft.MvvmLight;
    using Squalr.Engine.Common.Logging;
    using Squalr.Source.Settings;
    using Squalr.View.Dialogs;
    using System;
    using System.IO;
    using System.Threading;
    using System.Windows;

    /// <summary>
    /// The view model for the project deletion dialog.
    /// </summary>
    public class DeleteProjectDialogViewModel : ViewModelBase
    {
        /// <summary>
        /// Singleton instance of the <see cref="DeleteProjectDialogViewModel" /> class.
        /// </summary>
        private static Lazy<DeleteProjectDialogViewModel> deleteProjectDialogViewModelInstance = new Lazy<DeleteProjectDialogViewModel>(
                () => { return new DeleteProjectDialogViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        private String confirmDeleteText;

        private String projectName;

        private DeleteProjectDialogViewModel() : base()
        {
        }

        public String ConfirmDeleteText
        {
            get
            {
                return this.confirmDeleteText;
            }

            set
            {
                this.confirmDeleteText = value;
                this.RaisePropertyChanged(nameof(this.ConfirmDeleteText));
                this.RaisePropertyChanged(nameof(this.IsConfirmationMatching));
            }
        }

        public String ProjectName
        {
            get
            {
                return this.projectName;
            }

            set
            {
                this.projectName = value;
                this.RaisePropertyChanged(nameof(this.ProjectName));
            }
        }

        public Boolean IsConfirmationMatching
        {
            get
            {
                return this.ConfirmDeleteText == "delete";
            }
        }

        /// <summary>
        /// Gets a singleton instance of the <see cref="DeleteProjectDialogViewModel" /> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static DeleteProjectDialogViewModel GetInstance()
        {
            return DeleteProjectDialogViewModel.deleteProjectDialogViewModelInstance.Value;
        }

        /// <summary>
        /// Shows the delete project dialog, deleting the project if the dialog result was true.
        /// </summary>
        /// <param name="owner">The window that owns this dialog.</param>
        /// <param name="projectName">The project name to potentially delete.</param>
        /// <returns>A value indicating whether the project deletion was successful.</returns>
        public Boolean ShowDialog(Window owner, String projectName)
        {
            this.ConfirmDeleteText = String.Empty;
            this.ProjectName = projectName;

            DeleteProjectDialog deleteProjectDialog = new DeleteProjectDialog() { Owner = owner };

            if (deleteProjectDialog.ShowDialog() == true && this.IsConfirmationMatching)
            {
                String projectPath = Path.Combine(SettingsViewModel.GetInstance().ProjectRoot, projectName);

                if (!Directory.Exists(projectPath))
                {
                    Logger.Log(LogLevel.Error, "Unable to delete project, directory does not exist: " + projectPath);
                    return false;
                }

                try
                {
                    Directory.Delete(projectPath, recursive: true);
                    return true;
                }
                catch (Exception ex)
                {
                    Logger.Log(LogLevel.Error, "Error deleting project and files", ex);
                }
            }

            return false;
        }
    }
    //// End class
}
//// End namespace