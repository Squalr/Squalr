namespace Squalr.Source.ProjectExplorer
{
    using CSScripting;
    using GalaSoft.MvvmLight.Command;
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.DataStructures;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Common.OS;
    using Squalr.Engine.Memory;
    using Squalr.Engine.Projects;
    using Squalr.Engine.Projects.Items;
    using Squalr.Source.Docking;
    using Squalr.Source.Editors.RenameEditor;
    using Squalr.Source.Editors.ScriptEditor;
    using Squalr.Source.Editors.ValueEditor;
    using Squalr.Source.ProjectExplorer.Dialogs;
    using Squalr.Source.ProjectExplorer.ProjectItems;
    using Squalr.Source.Settings;
    using System;
    using System.Collections.Generic;
    using System.IO;
    using System.Linq;
    using System.Net;
    using System.Threading;
    using System.Threading.Tasks;
    using System.Windows;
    using System.Windows.Input;

    public class ProjectExplorerViewModel : ToolViewModel
    {
        /// <summary>
        /// Singleton instance of the <see cref="ProjectExplorerViewModel" /> class.
        /// </summary>
        private static Lazy<ProjectExplorerViewModel> projectExplorerViewModelInstance = new Lazy<ProjectExplorerViewModel>(
                () => { return new ProjectExplorerViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        private FullyObservableCollection<DirectoryItemView> projectRoot;

        /// <summary>
        /// The primary selected project item.
        /// </summary>
        private ProjectItemView selectedProjectItem;

        /// <summary>
        /// The selected project items.
        /// </summary>
        private List<ProjectItemView> selectedProjectItems;

        private ProjectExplorerViewModel() : base("Project Explorer")
        {
            this.SetProjectRootCommand = new RelayCommand(() => this.SetProjectRoot());
            this.SelectProjectCommand = new RelayCommand(() => this.SelectProject());
            this.SelectProjectItemCommand = new RelayCommand<Object>((selectedItem) => this.SelectedProjectItem = selectedItem as ProjectItemView, (selectedItem) => true);
            this.ToggleSelectionActivationCommand = new RelayCommand(() => this.ToggleSelectionActivation());
            this.EditProjectItemValueCommand = new RelayCommand<ProjectItemView>((projectItem) => this.EditProjectItemValue(projectItem), (projectItem) => true);
            this.RenameProjectItemCommand = new RelayCommand<ProjectItemView>((projectItem) => this.RenameProjectItem(projectItem), (projectItem) => true);
            this.RenameSelectedProjectItemsCommand = new RelayCommand(() => this.RenameSelectedProjectItems(), () => true);
            this.AddNewFolderItemCommand = new RelayCommand(() => this.AddNewProjectItem(typeof(DirectoryItem)), () => true);
            this.AddNewAddressItemCommand = new RelayCommand(() => this.AddNewProjectItem(typeof(PointerItem)), () => true);
            this.AddNewScriptItemCommand = new RelayCommand(() => this.AddNewProjectItem(typeof(ScriptItem)), () => true);
            this.AddNewInstructionItemCommand = new RelayCommand(() => this.AddNewProjectItem(typeof(InstructionItem)), () => true);
            this.ResolveModuleNameCommand = new RelayCommand(() => this.ResolveModuleNamesForSelectedItems(), () => true);
            this.OpenFileExplorerCommand = new RelayCommand<ProjectItemView>((projectItem) => this.OpenFileExplorer(projectItem), (projectItem) => true);
            this.CopySelectionCommand = new RelayCommand(() => this.CopySelection(), () => true);
            this.CutSelectionCommand = new RelayCommand(() => this.CutSelection(), () => true);
            this.PasteSelectionCommand = new RelayCommand(() => this.PasteSelection(), () => true);
            this.DeleteSelectionCommand = new RelayCommand(() => this.DeleteSelection(), () => true);

            DockingViewModel.GetInstance().RegisterViewModel(this);
            this.RunUpdateLoop();
        }

        /// <summary>
        /// Gets the command to set the project root.
        /// </summary>
        public ICommand SetProjectRootCommand { get; private set; }

        /// <summary>
        /// Gets the command to open a project.
        /// </summary>
        public ICommand SelectProjectCommand { get; private set; }

        /// <summary>
        /// Gets the command to select a project item.
        /// </summary>
        public ICommand SelectProjectItemCommand { get; private set; }

        /// <summary>
        /// Gets the command to add a new folder.
        /// </summary>
        public ICommand AddNewFolderItemCommand { get; private set; }

        /// <summary>
        /// Gets the command to add a new address.
        /// </summary>
        public ICommand AddNewAddressItemCommand { get; private set; }

        /// <summary>
        /// Gets the command to add a new instruction.
        /// </summary>
        public ICommand AddNewInstructionItemCommand { get; private set; }

        /// <summary>
        /// Gets the command to add a new script.
        /// </summary>
        public ICommand AddNewScriptItemCommand { get; private set; }

        /// <summary>
        /// Gets the command to edit a project item value.
        /// </summary>
        public ICommand EditProjectItemValueCommand { get; private set; }

        /// <summary>
        /// Gets the command to rename a project item.
        /// </summary>
        public ICommand RenameProjectItemCommand { get; private set; }

        /// <summary>
        /// Gets the command to rename all selected project items.
        /// </summary>
        public ICommand RenameSelectedProjectItemsCommand { get; private set; }

        /// <summary>
        /// Gets the command to toggle the activation the selected project explorer items.
        /// </summary>
        public ICommand ToggleSelectionActivationCommand { get; private set; }

        /// <summary>
        /// Gets the command to delete the selected project explorer items.
        /// </summary>
        public ICommand DeleteSelectionCommand { get; private set; }

        /// <summary>
        /// Gets the command to copy the selection to the clipboard.
        /// </summary>
        public ICommand CopySelectionCommand { get; private set; }

        /// <summary>
        /// Gets the command to paste the selection to the clipboard.
        /// </summary>
        public ICommand PasteSelectionCommand { get; private set; }

        /// <summary>
        /// Gets the command to cut the selection to the clipboard.
        /// </summary>
        public ICommand CutSelectionCommand { get; private set; }

        /// <summary>
        /// Gets a command to view a file or directory in the native file explorer.
        /// </summary>
        public ICommand OpenFileExplorerCommand { get; private set; }

        /// <summary>
        /// Gets a command to resolve the module base of an address item if possible.
        /// </summary>
        public ICommand ResolveModuleNameCommand { get; private set; }

        /// <summary>
        /// Gets or sets the project root tree of the current project.
        /// </summary>
        public FullyObservableCollection<DirectoryItemView> ProjectRoot
        {
            get
            {
                return projectRoot;
            }

            set
            {
                projectRoot = value;
                RaisePropertyChanged(nameof(this.ProjectRoot));
                RaisePropertyChanged(nameof(this.HasProjectRoot));
            }
        }

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
        /// Gets or sets the selected project item.
        /// </summary>
        public ProjectItemView SelectedProjectItem
        {
            get
            {
                return this.selectedProjectItem;
            }

            set
            {
                this.selectedProjectItem = value;
                this.RaisePropertyChanged(nameof(this.SelectedProjectItem));
            }
        }

        /// <summary>
        /// Gets or sets the selected project item.
        /// </summary>
        public List<ProjectItemView> SelectedProjectItems
        {
            get
            {
                return this.selectedProjectItems;
            }

            set
            {
                this.selectedProjectItems = value;
                this.RaisePropertyChanged(nameof(this.SelectedProjectItems));
            }
        }

        public Boolean IsSelectionAddress
        {
            get
            {
                return this.selectedProjectItem?.ProjectItem is AddressItem;
            }
        }

        public Boolean HasProjectRoot
        {
            get
            {
                return (this.ProjectRoot != null && (this.ProjectRoot?.Count ?? 0) > 0) ? true : false;
            }
        }

        private List<ProjectItemView> ClipBoard { get; set; }

        /// <summary>
        /// Gets a singleton instance of the <see cref="ProjectExplorerViewModel" /> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static ProjectExplorerViewModel GetInstance()
        {
            return ProjectExplorerViewModel.projectExplorerViewModelInstance.Value;
        }

        /// <summary>
        /// Runs the update loop, updating all scan results.
        /// </summary>
        public void RunUpdateLoop()
        {
            Task.Run(() =>
            {
                while (true)
                {
                    ProjectItem projectRoot = this.ProjectRoot?.FirstOrDefault()?.ProjectItem;

                    try
                    {
                        projectRoot?.Update();
                    }
                    catch (Exception ex)
                    {
                        Logger.Log(LogLevel.Warn, "Error updating project", ex);
                    }

                    // TODO: Probably get this from user settings and clamp it
                    Thread.Sleep(50);
                }
            });
        }

        public void AddProjectItems(params ProjectItem[] projectItems)
        {
            if (projectItems == null)
            {
                return;
            }

            this.CreateProjectIfNone();

            DirectoryItemView directoryItemView = this.SelectedProjectItem as DirectoryItemView ?? this.ProjectRoot?.FirstOrDefault();

            foreach (ProjectItem projectItem in projectItems)
            {
                directoryItemView?.AddChild(projectItem);
            }
        }

        /// <summary>
        /// Prompts the user to set a new project root.
        /// </summary>
        private void SetProjectRoot()
        {
            try
            {
                using (System.Windows.Forms.FolderBrowserDialog folderBrowserDialog = new System.Windows.Forms.FolderBrowserDialog())
                {
                    folderBrowserDialog.SelectedPath = SettingsViewModel.GetInstance().ProjectRoot;

                    if (folderBrowserDialog.ShowDialog() == System.Windows.Forms.DialogResult.OK && !String.IsNullOrWhiteSpace(folderBrowserDialog.SelectedPath))
                    {
                        if (Directory.Exists(folderBrowserDialog.SelectedPath))
                        {
                            SettingsViewModel.GetInstance().ProjectRoot = folderBrowserDialog.SelectedPath;
                            SessionManager.Project = null;
                            this.ProjectRoot = null;
                        }
                    }
                    else
                    {
                        throw new Exception("Folder not found");
                    }
                }
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Unable to open project", ex);
            }
        }

        /// <summary>
        /// Prompts the user to open a project.
        /// </summary>
        private void SelectProject()
        {
            try
            {
                Application.Current.Dispatcher.Invoke(new Action(() =>
                {
                    SelectProjectDialogViewModel.GetInstance().ShowSelectProjectDialog(Application.Current.MainWindow, this.DoOpenProject);

                    if (!Directory.Exists(this.ProjectRoot?.FirstOrDefault()?.FilePath))
                    {
                        this.ProjectRoot = null;
                    }
                }));
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Unable to open project", ex);
            }
        }

        private void DoOpenProject(String projectPath)
        {
            if (!Directory.Exists(projectPath))
            {
                throw new Exception("Folder not found");
            }

            SessionManager.Project = new Project(SessionManager.Session, projectPath);

            // Create project root folder (initialize expanded for better UX)
            DirectoryItemView projectRootFolder = new DirectoryItemView(SessionManager.Project)
            {
                IsExpanded = true
            };

            this.ProjectRoot = new FullyObservableCollection<DirectoryItemView> { projectRootFolder };
        }

        /// <summary>
        /// Adds a new address to the project items.
        /// </summary>
        /// <param name="projectItemType"></param>
        /// <param name="emulatorType"></param>
        private void AddNewProjectItem(Type projectItemType)
        {
            this.CreateProjectIfNone();

            DirectoryItemView directoryItemView = this.SelectedProjectItem as DirectoryItemView ?? this.ProjectRoot.FirstOrDefault();

            switch (projectItemType)
            {
                case Type _ when projectItemType == typeof(DirectoryItem):
                    // Create directories slightly differently. Just create the directory on disk via this call, then it should get picked up.
                    DirectoryItem.CreateNewDirectory(directoryItemView?.ProjectItem as DirectoryItem);
                    break;
                case Type _ when projectItemType == typeof(PointerItem):
                    directoryItemView?.AddChild(new PointerItem(SessionManager.Session));
                    break;
                case Type _ when projectItemType == typeof(ScriptItem):
                    directoryItemView?.AddChild(new ScriptItem(SessionManager.Session));
                    break;
                case Type _ when projectItemType == typeof(InstructionItem):
                    directoryItemView?.AddChild(new InstructionItem(SessionManager.Session));
                    break;
                default:
                    Logger.Log(LogLevel.Error, "Unknown project item type - " + projectItemType.ToString());
                    break;
            }
        }

        /// <summary>
        /// Edits a project item value based on the project item type.
        /// </summary>
        /// <param name="projectItemView">The project item to edit.</param>
        private void EditProjectItemValue(ProjectItemView projectItemView)
        {
            ProjectItem projectItem = projectItemView?.ProjectItem;

            if (projectItem is AddressItem)
            {
                ValueEditorModel valueEditor = new ValueEditorModel();
                AddressItem addressItem = projectItem as AddressItem;
                dynamic result = valueEditor.EditValue(null, null, addressItem);

                if (SyntaxChecker.CanParseValue(addressItem.DataType, result?.ToString()))
                {
                    addressItem.AddressValue = result;
                }
            }
            else if (projectItem is ScriptItem)
            {
                ScriptEditorModel scriptEditor = new ScriptEditorModel();
                ScriptItem scriptItem = projectItem as ScriptItem;
                scriptItem.Script = scriptEditor.EditValue(null, null, scriptItem.Script) as String;
            }
        }

        /// <summary>
        /// Opens a dialog to rename a given project item.
        /// </summary>
        /// <param name="projectItemView">The project item to rename.</param>
        private void RenameProjectItem(ProjectItemView projectItemView)
        {
            ProjectItem projectItem = projectItemView?.ProjectItem;

            if (projectItem != null)
            {
                RenameProjectItemDialogViewModel.GetInstance().ShowDialog(projectItem);
            }
        }

        /// <summary>
        /// Opens a dialog to rename all selected project items.
        /// </summary>
        private void RenameSelectedProjectItems()
        {
            if (this.SelectedProjectItem != null)
            {
                RenameProjectItemDialogViewModel.GetInstance().ShowDialog(this.SelectedProjectItems?.Select(next => next?.ProjectItem)?.ToArray());
            }
        }

        private void CreateProjectIfNone()
        {
            if (this.ProjectRoot == null)
            {
                try
                {
                    String newProjectDirectory = String.Empty;
                    IEnumerable<String> projects = this.Projects;

                    for (Int32 appendedNumber = 0; appendedNumber < Int32.MaxValue; appendedNumber++)
                    {
                        String suffix = appendedNumber == 0 ? String.Empty : " " + appendedNumber.ToString();
                        newProjectDirectory = Path.Combine(SettingsViewModel.GetInstance().ProjectRoot, "New Project" + suffix);

                        if (!Directory.Exists(newProjectDirectory))
                        {
                            Directory.CreateDirectory(newProjectDirectory);
                            this.RaisePropertyChanged(nameof(this.Projects));
                            this.DoOpenProject(newProjectDirectory);
                            break;
                        }
                    }
                }
                catch (Exception ex)
                {
                    Logger.Log(LogLevel.Error, "Error creating new project directory", ex);
                }
            }
        }

        /// <summary>
        /// Toggles the activation the selected project explorer items.
        /// </summary>
        private void ToggleSelectionActivation()
        {
            if (this.SelectedProjectItems == null)
            {
                return;
            }

            foreach (ProjectItemView projectItem in this.SelectedProjectItems.ToArray())
            {
                if (projectItem != null)
                {
                    projectItem.IsActivated = !projectItem.IsActivated;
                }
            }
        }

        private void ResolveModuleNamesForSelectedItems()
        {
            if (this.SelectedProjectItems == null)
            {
                return;
            }

            foreach (PointerItem addressItem in this.SelectedProjectItems.Select(view => view.ProjectItem).Cast<PointerItem>().ToArray())
            {
                if (addressItem != null && (addressItem.ModuleName == null || addressItem.ModuleName == String.Empty))
                {
                    String moduleName;
                    UInt64 address = MemoryQueryer.Instance.AddressToModule(SessionManager.Session.OpenedProcess, addressItem.ModuleOffset, out moduleName, SessionManager.Session.DetectedEmulator);

                    addressItem.ModuleName = moduleName;
                    addressItem.ModuleOffset = address;
                }
            }
        }

        /// <summary>
        /// Deletes the selected project explorer items.
        /// </summary>
        /// <param name="promptUser">A value indicating whether to prompt the user when deleting the selected project items.</param>
        private void DeleteSelection(Boolean promptUser = true)
        {
            if (this.SelectedProjectItems == null)
            {
                return;
            }

            System.Windows.Window mainWindow = System.Windows.Application.Current.MainWindow;

            if (promptUser && !TwoChoiceDialogViewModel.GetInstance().ShowDialog(owner: mainWindow, headerText: "Confirm", bodyText: "Delete selected items?", optionOkayText: "Okay", optionCancelText: "Cancel"))
            {
                return;
            }

            foreach (ProjectItemView projectItemView in this.SelectedProjectItems.ToArray())
            {
                if (projectItemView != null)
                {
                    projectItemView?.ProjectItem?.Parent?.DeleteChild(projectItemView?.ProjectItem);
                }
            }

            this.SelectedProjectItems = null;
        }

        /// <summary>
        /// Copies the selected project explorer items.
        /// </summary>
        private void CopySelection()
        {
            this.ClipBoard = this.SelectedProjectItems?.ToList();
        }

        /// <summary>
        /// Pastes the copied project explorer items.
        /// </summary>
        private void PasteSelection()
        {
            if (this.ClipBoard == null || this.ClipBoard.Count() <= 0)
            {
                 return;
            }

            DirectoryItemView directoryItemView = this.SelectedProjectItem as DirectoryItemView ?? this.ProjectRoot.FirstOrDefault();

            if (directoryItemView != null)
            {
                foreach (ProjectItemView next in this.ClipBoard)
                {
                    if (next?.ProjectItem is DirectoryItem)
                    {
                        // Directories require special treatment here. Rather than attempting to clone all contents, we simply copy the folder and changes will be picked up.
                        DirectoryItem directoryItem = next.ProjectItem as DirectoryItem;
                        directoryItem.Clone(directoryItemView?.ProjectItem as DirectoryItem);
                    }
                    else
                    {
                        directoryItemView.AddChild(next?.ProjectItem?.Clone(true));
                    }
                }
            }
        }

        /// <summary>
        /// Cuts the selected project explorer items.
        /// </summary>
        private void CutSelection()
        {
            this.ClipBoard = this.SelectedProjectItems?.ToList();
            this.DeleteSelection(promptUser: false);
        }

        private void OpenFileExplorer(ProjectItemView projectItemView)
        {
            String directory = projectItemView.ProjectItem is DirectoryItem ? projectItemView.ProjectItem.FullPath : Path.GetDirectoryName(projectItemView.ProjectItem.FullPath);

            OSUtils.OpenPathInFileExplorer(directory);
        }
    }
    //// End class
}
//// End namespace