namespace Squalr.Engine.Projects.Items
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Extensions;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Processes;
    using System;
    using System.Collections.Generic;
    using System.IO;
    using System.Linq;

    /// <summary>
    /// Defines a directory in the project.
    /// </summary>
    public class DirectoryItem : ProjectItem
    {
        /// <summary>
        /// A lock for accessing the <see cref="childItems"/> map.
        /// </summary>
        private readonly Object itemsLock = new Object();

        /// <summary>
        /// The child project items under this directory.
        /// </summary>
        private Dictionary<String, ProjectItem> childItems;

        /// <summary>
        /// Initializes a new instance of the <see cref="DirectoryItem" /> class.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        /// <param name="directoryPath">The path to the directory that this dirctory item represents.</param>
        /// <param name="parent">The parent directory containing this directory item.</param>
        public DirectoryItem(ProcessSession processSession, String directoryPath, DirectoryItem parent) : base(processSession, directoryPath)
        {
            // Bypass setters to avoid re-saving
            this.Parent = parent;
            this.childItems = new Dictionary<String, ProjectItem>();
            this.name = new DirectoryInfo(directoryPath).Name;

            try
            {
                this.LoadAllChildProjectItems();
                this.WatchForUpdates();
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error initializing project directory", ex);
            }
        }

        /// <summary>
        /// A function definition for when a project item is deleted.
        /// </summary>
        /// <param name="projectItem">The project item that was deleted.</param>
        public delegate void ProjectItemDeleted(ProjectItem projectItem);

        /// <summary>
        /// A function definition for when a project item is added.
        /// </summary>
        /// <param name="projectItem">The project item that was added.</param>
        public delegate void ProjectItemAdded(ProjectItem projectItem);

        /// <summary>
        /// Gets or sets an event that is fired when a project item is deleted.
        /// </summary>
        public ProjectItemDeleted ProjectItemDeletedEvent { get; set; }

        /// <summary>
        /// Gets or sets an event that is fired when a project item is added.
        /// </summary>
        public ProjectItemDeleted ProjectItemAddedEvent { get; set; }

        /// <summary>
        /// Gets or sets the file name for this project item.
        /// </summary>
        public override String Name
        {
            get
            {
                return base.Name;
            }

            set
            {
                if (this.Name == value)
                {
                    return;
                }

                this.Rename(value);
                this.RaisePropertyChanged(nameof(this.Name));
            }
        }

        /// <summary>
        /// Gets the child project items under this directory.
        /// </summary>
        public Dictionary<String, ProjectItem> ChildItems
        {
            get
            {
                return this.childItems;
            }

            private set
            {
                this.childItems = value;
                this.RaisePropertyChanged(nameof(this.ChildItems));
            }
        }

        /// <summary>
        /// Gets a lock for accessing the <see cref="childItems"/> map.
        /// </summary>
        private Object ItemsLock
        {
            get
            {
                return this.itemsLock;
            }
        }

        /// <summary>
        /// Gets or sets an object to watch for file system changes under this directory.
        /// </summary>
        private FileSystemWatcher FileSystemWatcher { get; set; }

        /// <summary>
        /// Creates a directory item from the specified project directory path, instantiating all children.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        /// <param name="directoryPath">The path to the project directory or subdirectory.</param>
        /// <param name="parent">The parent directory item containing the new directory item.</param>
        /// <returns>The instantiated directory item.</returns>
        public static DirectoryItem FromDirectory(ProcessSession processSession, String directoryPath, DirectoryItem parent)
        {
            try
            {
                if (!Directory.Exists(directoryPath))
                {
                    throw new Exception("Directory does not exist: " + directoryPath);
                }

                return new DirectoryItem(processSession, directoryPath, parent);
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error loading file", ex);
                throw ex;
            }
        }

        /// <summary>
        /// Creates a new folder under the given parent directory.
        /// </summary>
        /// <param name="parent">The parent which will contain the new folder.</param>
        public static void CreateNewDirectory(DirectoryItem parent)
        {
            try
            {
                String newName = DirectoryItem.MakeNameUnique("New Folder", parent);
                String result = Path.Combine(parent.FullPath, newName);
                Directory.CreateDirectory(result);
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error creating new directory.", ex);
            }
        }

        /// <summary>
        /// Clones this directory to a new folder under the given parent.
        /// </summary>
        /// <param name="destinationDirectory">The parent directory to which this directory is cloned.</param>
        public void Clone(DirectoryItem destinationDirectory)
        {
            try
            {
                String tempPath = Path.Combine(Path.GetTempPath(), Path.GetRandomFileName());
                DirectoryInfo sourceDirectory = new DirectoryInfo(this.FullPath);
                String uniqueName = DirectoryItem.MakeNameUnique(sourceDirectory.Name, destinationDirectory);
                String uniquePath = Path.Combine(destinationDirectory?.FullPath, uniqueName);
                DirectoryInfo tempDirectory = new DirectoryInfo(tempPath);
                DirectoryInfo targetDirectory = new DirectoryInfo(uniquePath);
                DirectoryItem.CopyAll(sourceDirectory, tempDirectory);

                // This is done in one step to prevent picking up intermediate changes
                Directory.Move(tempPath, uniquePath);
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error cloning directory.", ex);
            }
        }

        /// <summary>
        /// Updates all project items under this directory.
        /// </summary>
        public override void Update()
        {
            lock (this.ItemsLock)
            {
                foreach (KeyValuePair<String, ProjectItem> child in this.ChildItems)
                {
                    child.Value?.Update();
                }
            }
        }

        /// <summary>
        /// Renames this directory item to a new specified name.
        /// </summary>
        /// <param name="newName">The new name for this directory item.</param>
        /// <returns>A value indicating whether or not the rename was successful.</returns>
        public Boolean Rename(String newName)
        {
            try
            {
                this.StopWatchingForUpdates();

                if (!Path.IsPathRooted(newName))
                {
                    String root = this.Parent?.FullPath ?? ProjectSettings.ProjectRoot ?? String.Empty;
                    newName = Path.Combine(root, newName);
                }

                Directory.Move(this.FullPath, newName);

                return true;
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Unable to rename directory", ex);

                return false;
            }
            finally
            {
                if (Directory.Exists(this.FullPath))
                {
                    this.WatchForUpdates();
                }
            }
        }

        /// <summary>
        /// Adds the specified project item to this directory.
        /// </summary>
        /// <param name="projectItem">The project item to add.</param>
        public void AddChild(ProjectItem projectItem)
        {
            projectItem.Parent = this;
            projectItem.Save();

            // Force load rather than waiting on the directory watcher to avoid timing issues of resolving name conflicts when adding multiple children in quick succession.
            this.LoadProjectItem(projectItem.FullPath, supressWarnings: true);
        }

        /// <summary>
        /// Removes the specified project item from this directory.
        /// </summary>
        /// <param name="projectItem">The project item to remove.</param>
        public void DeleteChild(ProjectItem projectItem)
        {
            if (projectItem == null)
            {
                return;
            }

            try
            {
                if (this.ChildItems.ContainsKey(projectItem.FullPath))
                {
                    if (projectItem is DirectoryItem)
                    {
                        Directory.Delete(projectItem.FullPath, recursive: true);
                    }
                    else
                    {
                        File.Delete(projectItem.FullPath);
                    }
                }
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Unable to delete project item", ex);
            }
        }

        /// <summary>
        /// Saves this directory to disk.
        /// </summary>
        public override void Save()
        {
            try
            {
                if (!Directory.Exists(this.FullPath))
                {
                    Directory.CreateDirectory(this.FullPath);
                }
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error creating directory within project.", ex);
            }
        }

        /// <summary>
        /// Given a desired folder name, find a unique variant of this name by appending increasing numerals to guarantee the folder exists.
        /// </summary>
        /// <param name="newName">The desired name.</param>
        /// <param name="parent">The parent which will contain the new folder.</param>
        /// <returns>The new name from the given desired new name, potentially with numerals appended to ensure a unique directory path.</returns>
        private static String MakeNameUnique(String newName, DirectoryItem parent)
        {
            if (parent == null)
            {
                Logger.Log(LogLevel.Error, "Unable to create new directory, no parent folder provided.");
                return String.Empty;
            }

            String root = parent.FullPath;

            try
            {
                DirectoryInfo rootInfo = new DirectoryInfo(root);
                IEnumerable<DirectoryInfo> subDirectories = Directory.GetDirectories(rootInfo.FullName).Select(directory => new DirectoryInfo(directory));

                if (subDirectories.Any(directory => newName.Equals(directory?.Name, StringComparison.OrdinalIgnoreCase)))
                {
                    // Find all files that match the pattern of {newfilename #}, and extract the numbers
                    IEnumerable<String> numberedSuffixStrings = subDirectories
                        .Where(directory => directory.Name?.StartsWith(newName, StringComparison.OrdinalIgnoreCase) ?? false)
                        .Select(directory => directory.Name?.Substring(newName.Length).Trim());
                    IEnumerable<Int32> neighboringNumberedFiles = numberedSuffixStrings
                        .Where(childSuffix => SyntaxChecker.CanParseValue(ScannableType.Int32, childSuffix))
                        .Select(childSuffix => (Int32)Conversions.ParsePrimitiveStringAsPrimitive(ScannableType.Int32, childSuffix));

                    Int32 neighboringNumberedFileCount = neighboringNumberedFiles.Count();
                    IEnumerable<Int32> missingNumbersInSequence = Enumerable.Range(1, neighboringNumberedFileCount).Except(neighboringNumberedFiles);

                    // Find the first gap in the numbers. If no gap, just take the next number in the sequence
                    Int32 numberToAppend = missingNumbersInSequence.IsNullOrEmpty() ? neighboringNumberedFileCount + 1 : missingNumbersInSequence.First();
                    String suffix = numberToAppend == 0 ? String.Empty : " " + numberToAppend.ToString();

                    newName = newName + suffix;
                }
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error resolving conflicting project name.", ex);
                return String.Empty;
            }

            return newName;
        }

        /// <summary>
        /// Utility function for copying the contents of one directory to another.
        /// </summary>
        /// <param name="source">The source directory.</param>
        /// <param name="target">The destination directory.</param>
        private static void CopyAll(DirectoryInfo source, DirectoryInfo target)
        {
            Directory.CreateDirectory(target.FullName);

            // Copy each file into the new directory.
            foreach (FileInfo fileInfo in source.GetFiles())
            {
                fileInfo.CopyTo(Path.Combine(target.FullName, fileInfo.Name), false);
            }

            // Copy each subdirectory using recursion.
            foreach (DirectoryInfo subDirectory in source.GetDirectories())
            {
                DirectoryInfo nextTargetSubDir = target.CreateSubdirectory(subDirectory.Name);
                DirectoryItem.CopyAll(subDirectory, nextTargetSubDir);
            }
        }

        /// <summary>
        /// Gets the list of files in the directory Name passed.
        /// </summary>
        private void LoadAllChildProjectItems()
        {
            lock (this.ItemsLock)
            {
                this.childItems?.Clear();
            }

            try
            {
                IEnumerable<DirectoryInfo> subdirectories = Directory.GetDirectories(this.FullPath).Select(subdirectory => new DirectoryInfo(subdirectory));

                foreach (DirectoryInfo subdirectory in subdirectories)
                {
                    this.LoadProjectItem(subdirectory.FullName);
                }
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error fetching directories", ex);
            }

            try
            {
                foreach (FileInfo file in Directory.GetFiles(this.FullPath).Select(directoryFile => new FileInfo(directoryFile)))
                {
                    this.LoadProjectItem(file.FullName);
                }
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error fetching files", ex);
            }

            // Notify changes after everything finished loading
            this.RaisePropertyChanged(nameof(this.ChildItems));
        }

        /// <summary>
        /// Deserializes a project or directory item from disk given a path to the item.
        /// </summary>
        /// <param name="projectItemPath">The path to the project item.</param>
        /// <param name="supressWarnings">Whether warnings should be supressed for any failed operations.</param>
        /// <returns>The deserialized project item.</returns>
        private ProjectItem LoadProjectItem(String projectItemPath, Boolean supressWarnings = false)
        {
            if (Directory.Exists(projectItemPath))
            {
                try
                {
                    String lastFolderName = new DirectoryInfo(projectItemPath).Name;

                    // Do not load any directories prefixed with a '.', as these are often system or version control folders.
                    if (lastFolderName.StartsWith("."))
                    {
                        return null;
                    }

                    DirectoryItem childDirectory = DirectoryItem.FromDirectory(this.processSession, projectItemPath, this);

                    if (childDirectory != null)
                    {
                        lock (this.ItemsLock)
                        {
                            this.childItems?.Add(childDirectory.FullPath, childDirectory);
                        }

                        this.RaisePropertyChanged(nameof(this.ChildItems));
                        this.ProjectItemAddedEvent?.Invoke(childDirectory);
                    }

                    return childDirectory;
                }
                catch (Exception ex)
                {
                    if (!supressWarnings)
                    {
                        Logger.Log(LogLevel.Error, "Error loading directory", ex);
                    }
                }
            }
            else if (File.Exists(projectItemPath))
            {
                try
                {
                    ProjectItem projectItem = ProjectItem.FromFile(this.processSession, projectItemPath, this);

                    if (projectItem != null)
                    {
                        lock (this.ItemsLock)
                        {
                            this.childItems?.Add(projectItem.FullPath, projectItem);
                        }

                        this.RaisePropertyChanged(nameof(this.ChildItems));
                        this.ProjectItemAddedEvent?.Invoke(projectItem);

                        return projectItem;
                    }
                }
                catch (Exception ex)
                {
                    if (!supressWarnings)
                    {
                        Logger.Log(LogLevel.Error, "Error reading project item", ex);
                    }
                }
            }

            if (!supressWarnings)
            {
                Logger.Log(LogLevel.Error, "Unable to read project item from path: " + (projectItemPath ?? String.Empty));
            }

            return null;
        }

        /// <summary>
        /// Removes the child project item with the given path if it exists.
        /// </summary>
        /// <param name="projectItemPath">The path of the child project item to remove.</param>
        private void RemoveProjectItem(String projectItemPath)
        {
            lock (this.ItemsLock)
            {
                if (this.ChildItems.ContainsKey(projectItemPath))
                {
                    ProjectItem deletedProjectItem = this.ChildItems[projectItemPath];
                    if (deletedProjectItem != null)
                    {
                        deletedProjectItem.Parent = null;
                        this.ChildItems?.Remove(projectItemPath);
                        this.RaisePropertyChanged(nameof(this.ChildItems));
                        this.ProjectItemDeletedEvent?.Invoke(deletedProjectItem);
                    }
                }
            }
        }

        /// <summary>
        /// Initializes the filesystem watcher to listen for filesystem changes.
        /// </summary>
        private void WatchForUpdates()
        {
            this.StopWatchingForUpdates();

            try
            {
                this.FileSystemWatcher = new FileSystemWatcher(this.FullPath, "*.*")
                {
                    NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.DirectoryName,
                    EnableRaisingEvents = true,
                };

                this.FileSystemWatcher.Deleted += OnFilesOrDirectoriesChanged;
                this.FileSystemWatcher.Changed += OnFilesOrDirectoriesChanged;
                this.FileSystemWatcher.Renamed += OnFilesOrDirectoriesChanged;
                this.FileSystemWatcher.Created += OnFilesOrDirectoriesChanged;
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error watching project subdirectory " + this.FullPath + ". Project items may not properly refresh.", ex);
            }
        }

        /// <summary>
        /// Cancels and removes the current filesystem watcher.
        /// </summary>
        private void StopWatchingForUpdates()
        {
            if (this.FileSystemWatcher != null)
            {
                this.FileSystemWatcher.Deleted -= OnFilesOrDirectoriesChanged;
                this.FileSystemWatcher.Changed -= OnFilesOrDirectoriesChanged;
                this.FileSystemWatcher.Renamed -= OnFilesOrDirectoriesChanged;
                this.FileSystemWatcher.Created -= OnFilesOrDirectoriesChanged;
                this.FileSystemWatcher = null;
            }
        }

        /// <summary>
        /// Method invoked when files or directories change under the project root.
        /// </summary>
        /// <param name="source">The source object.</param>
        /// <param name="args">The filesystem change event args.</param>
        private void OnFilesOrDirectoriesChanged(Object source, FileSystemEventArgs args)
        {
            Boolean isDirectory = Directory.Exists(args.FullPath);

            switch (args.ChangeType)
            {
                case WatcherChangeTypes.Created:
                    if (!this.childItems.ContainsKey(args.FullPath))
                    {
                        // Supress warnings since sometimes the file is created as a 0b file, and then written to later
                        this.LoadProjectItem(args.FullPath, supressWarnings: true);
                    }
                    else
                    {
                        // TODO: Reread data from disc?
                    }

                    break;
                case WatcherChangeTypes.Deleted:
                    this.RemoveProjectItem(args.FullPath);
                    break;
                case WatcherChangeTypes.Changed:
                    if (!this.childItems.ContainsKey(args.FullPath))
                    {
                        this.LoadProjectItem(args.FullPath);
                    }
                    else
                    {
                        // TODO: Reread data from disc?
                    }

                    break;
                case WatcherChangeTypes.Renamed:
                    RenamedEventArgs renameArgs = args as RenamedEventArgs;

                    if (renameArgs != null)
                    {
                        this.RemoveProjectItem(renameArgs.OldFullPath);

                        // Supress warnings since sometimes the file is created as a 0b file, and then written to later
                        this.LoadProjectItem(args.FullPath, true);
                    }

                    break;
            }
        }
    }
    //// End class
}
//// End namespace