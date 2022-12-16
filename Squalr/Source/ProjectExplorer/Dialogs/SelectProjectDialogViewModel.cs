﻿namespace Squalr.Source.ProjectExplorer.Dialogs
{
    using GalaSoft.MvvmLight;
    using GalaSoft.MvvmLight.Command;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Common.OS;
    using Squalr.Source.Settings;
    using Squalr.View.Dialogs;
    using System;
    using System.Collections;
    using System.Collections.Generic;
    using System.IO;
    using System.Linq;
    using System.Threading;
    using System.Windows;
    using System.Windows.Input;

    public class SelectProjectDialogViewModel : ViewModelBase
    {
        /// <summary>
        /// Singleton instance of the <see cref="SelectProjectDialogViewModel" /> class.
        /// </summary>
        private static readonly Lazy<SelectProjectDialogViewModel> SelectProjectDialogViewModelInstance = new Lazy<SelectProjectDialogViewModel>(
                () => { return new SelectProjectDialogViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        private String searchTerm;

        private String selectedProject;

        private String newProjectName;

        private SelectProjectDialogViewModel() : base()
        {
            this.UpdateSelectedProjectCommand = new RelayCommand<Object>((selectedItems) => this.SelectedProject = (selectedItems as IList)?.Cast<String>()?.FirstOrDefault());
            this.OpenProjectCommand = new RelayCommand<String>((project) => this.OpenProject(project));
            this.RenameProjectCommand = new RelayCommand<String>((project) => this.RenameProject(project));
            this.RenameSelectedProjectCommand = new RelayCommand(() => this.RenameProject(this.SelectedProject));
            this.NewProjectCommand = new RelayCommand(() => this.CreateNewProject());
            this.DeleteProjectCommand = new RelayCommand<String>((project) => this.DeleteProject(project));
            this.DeleteSelectedProjectCommand = new RelayCommand(() => this.DeleteProject(this.SelectedProject));
            this.ShowSelectedProjectInFileExplorerCommand = new RelayCommand(() => this.ShowProjectInFileExplorer(this.SelectedProject));
            this.FileSystemWatcher = new FileSystemWatcher(SettingsViewModel.GetInstance().ProjectRoot, "*.*")
            {
                NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.DirectoryName,
                EnableRaisingEvents = true
            };

            this.FileSystemWatcher.Changed += this.OnProjectDirectoriesChanged;
            this.FileSystemWatcher.Created += this.OnProjectDirectoriesChanged;
            this.FileSystemWatcher.Renamed += this.OnProjectDirectoriesChanged;
            this.FileSystemWatcher.Deleted += this.OnProjectDirectoriesChanged;
        }

        /// <summary>
        /// Gets the command to update the current project selection.
        /// </summary>
        public ICommand UpdateSelectedProjectCommand { get; private set; }

        /// <summary>
        /// Gets the command to open a project.
        /// </summary>
        public ICommand OpenProjectCommand { get; private set; }

        /// <summary>
        /// Gets the command to rename the given project.
        /// </summary>
        public ICommand RenameProjectCommand { get; private set; }

        /// <summary>
        /// Gets the command to rename the selected project.
        /// </summary>
        public ICommand RenameSelectedProjectCommand { get; private set; }

        /// <summary>
        /// Gets the command to create a new project.
        /// </summary>
        public ICommand NewProjectCommand { get; private set; }

        /// <summary>
        /// Gets the command to delete a project.
        /// </summary>
        public ICommand DeleteProjectCommand { get; private set; }

        /// <summary>
        /// Gets the command to delete the selected project.
        /// </summary>
        public ICommand DeleteSelectedProjectCommand { get; private set; }

        /// <summary>
        /// Gets the command to open the selected project in file explorer.
        /// </summary>
        public ICommand ShowSelectedProjectInFileExplorerCommand { get; private set; }

        /// <summary>
        /// Gets a list of projects in the project root.
        /// </summary>
        public List<String> Projects
        {
            get
            {
                return Directory.EnumerateDirectories(SettingsViewModel.GetInstance().ProjectRoot).Select(path => new DirectoryInfo(path).Name).ToList();
            }
        }

        /// <summary>
        /// Gets a list of projects in the project root, filtered by the current search term.
        /// </summary>
        public List<String> FilteredProjects
        {
            get
            {
                return this.ProjectSearchTerm == null ? this.Projects : this.Projects.Select(project => project).Where(project => project.ToLower().Contains(this.ProjectSearchTerm.ToLower())).ToList();
            }
        }

        /// <summary>
        /// Gets or sets the project search term to filter project results.
        /// </summary>
        public String ProjectSearchTerm
        {
            get
            {
                return this.searchTerm;
            }

            set
            {
                this.searchTerm = value;
                this.RaisePropertyChanged(nameof(this.ProjectSearchTerm));
                this.RaisePropertyChanged(nameof(this.FilteredProjects));
            }
        }

        /// <summary>
        /// Gets or sets the current selected project.
        /// </summary>
        public String SelectedProject
        {
            get
            {
                return this.selectedProject;
            }

            set
            {
                this.selectedProject = value;
                this.RaisePropertyChanged(nameof(this.SelectedProject));
            }
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
                this.RaisePropertyChanged(nameof(this.IsProjectNameValid));
                this.RaisePropertyChanged(nameof(this.NewProjectNameStatus));
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

        public String NewProjectNameStatus
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

        /// <summary>
        /// Gets or sets an object to watch for file system changes under the project root directory.
        /// </summary>
        private FileSystemWatcher FileSystemWatcher { get; set; }

        private SelectProjectDialog SelectProjectDialog { get; set; }

        /// <summary>
        /// Gets a singleton instance of the <see cref="SelectProjectDialogViewModel" /> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static SelectProjectDialogViewModel GetInstance()
        {
            return SelectProjectDialogViewModel.SelectProjectDialogViewModelInstance.Value;
        }

        public void ShowSelectProjectDialog(Window owner, Action<String> projectPathCallback)
        {
            this.SelectedProject = String.Empty;
            this.SelectProjectDialog = new SelectProjectDialog() { Owner = owner };

            if (this.SelectProjectDialog.ShowDialog() == true)
            {
                String projectPath = Path.Combine(SettingsViewModel.GetInstance().ProjectRoot, this.SelectedProject);

                if (!String.IsNullOrEmpty(projectPath))
                {
                    projectPathCallback?.Invoke(projectPath);
                }
            }
        }

        private void OnProjectDirectoriesChanged(Object sender, FileSystemEventArgs e)
        {
            // Refresh project list
            this.RaisePropertyChanged(nameof(this.ProjectSearchTerm));
            this.RaisePropertyChanged(nameof(this.FilteredProjects));
        }

        /// <summary>
        /// Opens the specified project.
        /// </summary>
        /// <param name="project">The project name to open.</param>
        private void OpenProject(String project)
        {
            this.SelectedProject = project;
            this.SelectProjectDialog.SelectProject(this.SelectedProject);
        }

        /// <summary>
        /// Opens a dialog to rename the given project.
        /// </summary>
        /// <param name="project">The project name to rename.</param>
        private void RenameProject(String project)
        {
            RenameProjectDialogViewModel renameProjectDialog = RenameProjectDialogViewModel.GetInstance();

            if (renameProjectDialog.ShowDialog(this.SelectProjectDialog, project) == true)
            {
                this.RaisePropertyChanged(nameof(this.Projects));
                this.RaisePropertyChanged(nameof(this.FilteredProjects));
            }
        }

        /// <summary>
        /// Creates a new project folder.
        /// </summary>
        private void CreateNewProject()
        {
            CreateProjectDialogViewModel createProjectDialog = CreateProjectDialogViewModel.GetInstance();

            if (createProjectDialog.ShowDialog(this.SelectProjectDialog) == true)
            {
                this.OpenProject(createProjectDialog.NewProjectName);
            }
        }

        /// <summary>
        /// Deletes the given project from disk.
        /// </summary>
        /// <param name="project">The name of the project to delete.</param>
        private void DeleteProject(String project)
        {
            if (String.IsNullOrEmpty(project))
            {
                Logger.Log(LogLevel.Warn, "No project was selected to delete.");
                return;
            }

            String projectPath = Path.Combine(SettingsViewModel.GetInstance().ProjectRoot, project);

            if (!Directory.Exists(projectPath))
            {
                Logger.Log(LogLevel.Error, "Project does not exist on disk.");
                return;
            }

            if (DeleteProjectDialogViewModel.GetInstance().ShowDialog(this.SelectProjectDialog, project))
            {
                this.RaisePropertyChanged(nameof(this.Projects));
                this.RaisePropertyChanged(nameof(this.FilteredProjects));
            }
        }

        /// <summary>
        /// Opens the given project in file explorer.
        /// </summary>
        /// <param name="project">The project name which will be opened in file explorer.</param>
        private void ShowProjectInFileExplorer(String project)
        {
            String projectPath = Path.Combine(SettingsViewModel.GetInstance().ProjectRoot, project);

            OSUtils.OpenPathInFileExplorer(projectPath);
        }
    }
    //// End class
}
//// End namespace