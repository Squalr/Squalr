namespace Squalr.Source.ProjectExplorer.Dialogs
{
    using GalaSoft.MvvmLight;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Projects;
    using Squalr.Source.Settings;
    using Squalr.View.Dialogs;
    using System;
    using System.IO;
    using System.Threading;
    using System.Windows;

    /// <summary>
    /// The view model for the project renaming dialog.
    /// </summary>
    public class RenameProjectDialogViewModel : ViewModelBase
    {
        /// <summary>
        /// Singleton instance of the <see cref="RenameProjectDialogViewModel" /> class.
        /// </summary>
        private static Lazy<RenameProjectDialogViewModel> renameProjectDialogViewModelInstance = new Lazy<RenameProjectDialogViewModel>(
                () => { return new RenameProjectDialogViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        private String newProjectName;

        private String projectName;

        private RenameProjectDialogViewModel() : base()
        {
        }

        public String NewProjectName
        {
            get
            {
                return this.newProjectName;
            }

            set
            {
                this.newProjectName = value;
                this.RaisePropertyChanged(nameof(this.NewProjectName));
                this.RaisePropertyChanged(nameof(this.StatusText));
                this.RaisePropertyChanged(nameof(this.IsProjectNameValid));
            }
        }

        public String StatusText
        {
            get
            {
                if (this.NewProjectName != null)
                {
                    if (this.NewProjectName.IndexOfAny(Path.GetInvalidFileNameChars()) >= 0)
                    {
                        return "Invalid project name";
                    }
                    else if (Directory.Exists(Path.Combine(SettingsViewModel.GetInstance().ProjectRoot, this.NewProjectName)) && !String.IsNullOrWhiteSpace(this.NewProjectName))
                    {
                        return "Project already exists";
                    }
                }

                return String.Empty;
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

        public Boolean IsProjectNameValid
        {
            get
            {
                if (String.IsNullOrWhiteSpace(this.NewProjectName) || this.NewProjectName.IndexOfAny(Path.GetInvalidFileNameChars()) >= 0 ||
                    Directory.Exists(Path.Combine(SettingsViewModel.GetInstance().ProjectRoot, this.NewProjectName)))
                {
                    return false;
                }

                return true;
            }
        }

        /// <summary>
        /// Gets a singleton instance of the <see cref="RenameProjectDialogViewModel" /> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static RenameProjectDialogViewModel GetInstance()
        {
            return RenameProjectDialogViewModel.renameProjectDialogViewModelInstance.Value;
        }

        /// <summary>
        /// Shows the rename project dialog, deleting the project if the dialog result was true.
        /// </summary>
        /// <param name="owner">The window that owns this dialog.</param>
        /// <param name="projectName">The project name to potentially rename.</param>
        /// <returns>A value indicating whether the rename was successful.</returns>
        public Boolean ShowDialog(Window owner, String projectName)
        {
            this.NewProjectName = String.Empty;
            this.ProjectName = projectName;

            RenameProjectDialog renameProjectDialog = new RenameProjectDialog() { Owner = owner };

            if (renameProjectDialog.ShowDialog() == true && this.IsProjectNameValid)
            {
                try
                {
                    String projectPath = Path.Combine(SettingsViewModel.GetInstance().ProjectRoot, projectName);
                    String newProjectPath = Path.Combine(SettingsViewModel.GetInstance().ProjectRoot, this.NewProjectName);

                    Project project = new Project(SessionManager.Session, projectPath);

                    return project.Rename(newProjectPath);
                }
                catch (Exception ex)
                {
                    Logger.Log(LogLevel.Error, "Error renaming project folder", ex);
                }
            }

            return false;
        }
    }
    //// End class
}
//// End namespace