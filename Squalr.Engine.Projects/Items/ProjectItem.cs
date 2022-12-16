namespace Squalr.Engine.Projects.Items
{
    using SharpDX.DirectInput;
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Extensions;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Input.HotKeys;
    using Squalr.Engine.Processes;
    using System;
    using System.Collections.Generic;
    using System.ComponentModel;
    using System.IO;
    using System.Linq;
    using System.Runtime.Serialization;
    using System.Runtime.Serialization.Json;

    /// <summary>
    /// A base class for all project items that can be added to the project explorer.
    /// </summary>
    [KnownType(typeof(ProjectItem))]
    [KnownType(typeof(ScriptItem))]
    [KnownType(typeof(AddressItem))]
    [KnownType(typeof(InstructionItem))]
    [KnownType(typeof(PointerItem))]
    [KnownType(typeof(DotNetItem))]
    [KnownType(typeof(JavaItem))]
    [DataContract]
    public class ProjectItem : INotifyPropertyChanged, IDisposable
    {
        /// <summary>
        /// The name of this project item.
        /// </summary>
        [Browsable(false)]
        [DataMember]
        protected String name;

        /// <summary>
        /// The process session reference for accessing the current opened process.
        /// </summary>
        [Browsable(false)]
        protected ProcessSession processSession;

        /// <summary>
        /// The description of this project item.
        /// </summary>
        [Browsable(false)]
        [DataMember]
        private String description;

        /// <summary>
        /// The hotkey associated with this project item.
        /// </summary>
        [Browsable(false)]
        private Hotkey hotkey;

        /// <summary>
        /// A value indicating whether this project item has been activated.
        /// </summary>
        [Browsable(false)]
        private Boolean isActivated;

        /// <summary>
        /// The parent directory item that contains this project item.
        /// </summary>
        private DirectoryItem parent;

        /// <summary>
        /// Initializes a new instance of the <see cref="ProjectItem" /> class.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        internal ProjectItem(ProcessSession processSession) : this(processSession, String.Empty)
        {
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="ProjectItem" /> class.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        /// <param name="name">The name of the project item.</param>
        internal ProjectItem(ProcessSession processSession, String name)
        {
            // Bypass setters/getters to avoid triggering any view updates in constructor
            this.processSession = processSession;
            this.name = name ?? String.Empty;
            this.isActivated = false;
            this.ActivationLock = new Object();
        }

        /// <summary>
        /// An event that is raised when a property of this object changes.
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;

        /// <summary>
        /// Gets or sets the parent of this project item. This is also used to determine if this project item exists on disk.
        /// </summary>
        public DirectoryItem Parent
        {
            get
            {
                return this.parent;
            }

            set
            {
                this.parent = value;

                // Bypass normal setter to avoid calling rename logic
                this.name = this.MakeNameUnique(this.Name);
                this.RaisePropertyChanged(nameof(this.Name));
            }
        }

        /// <summary>
        /// Gets or sets a view that has been mapped onto this item. This is an abstraction violation, but a useful optimization.
        /// </summary>
        public Object MappedView { get; set; }

        /// <summary>
        /// Gets a value indicating whether this project item is represented on disk.
        /// </summary>
        public Boolean HasAssociatedFileOrFolder
        {
            get
            {
                return this.Parent != null || this as DirectoryItem != null;
            }
        }

        /// <summary>
        /// Gets or sets the file name for this project item.
        /// </summary>
        public virtual String Name
        {
            get
            {
                return this.name;
            }

            set
            {
                if (this.Name == value)
                {
                    return;
                }

                this.Rename(value);
            }
        }

        /// <summary>
        /// Gets or sets the description for this object.
        /// </summary>
        public virtual String Description
        {
            get
            {
                return this.description;
            }

            set
            {
                if (this.description == value)
                {
                    return;
                }

                this.description = value;
                this.RaisePropertyChanged(nameof(this.Description));
                this.Save();
            }
        }

        /// <summary>
        /// Gets or sets the hotkey for this project item.
        /// </summary>
        public virtual Hotkey HotKey
        {
            get
            {
                return this.hotkey;
            }

            set
            {
                if (this.hotkey == value)
                {
                    return;
                }

                this.hotkey = value;
                this.HotKey?.SetCallBackFunction(() => this.IsActivated = !this.IsActivated);
                this.RaisePropertyChanged(nameof(this.HotKey));
                this.Save();
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not this item is activated.
        /// </summary>
        [Browsable(false)]
        public Boolean IsActivated
        {
            get
            {
                return this.isActivated;
            }

            set
            {
                lock (this.ActivationLock)
                {
                    if (this.isActivated == value)
                    {
                        return;
                    }

                    // Change activation state
                    Boolean previousValue = this.isActivated;
                    this.isActivated = value;
                    this.OnActivationChanged();

                    // Activation failed
                    if (this.isActivated == previousValue)
                    {
                        return;
                    }

                    this.RaisePropertyChanged(nameof(this.IsActivated));
                }
            }
        }

        /// <summary>
        /// Gets a value indicating whether this project item is enabled.
        /// </summary>
        [Browsable(false)]
        public virtual Boolean IsEnabled
        {
            get
            {
                return true;
            }
        }

        /// <summary>
        /// Gets or sets the display value to represent this project item.
        /// </summary>
        public virtual String DisplayValue
        {
            get
            {
                return String.Empty;
            }

            set
            {
                throw new NotImplementedException();
            }
        }

        /// <summary>
        /// Gets the full path for this project item.
        /// </summary>
        [Browsable(false)]
        public String FullPath
        {
            get
            {
                return this.HasAssociatedFileOrFolder ? this.GetFilePathForName(this.Name) : this.Name;
            }
        }

        /// <summary>
        /// Gets or sets a lock for activating project items.
        /// </summary>
        private Object ActivationLock { get; set; }

        /// <summary>
        /// Deserializes a project item from the given file path.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        /// <param name="filePath">The file path of the project item to deserialize.</param>
        /// <param name="parent">The parent directory item that contains this project item.</param>
        /// <returns>The deserialized project item.</returns>
        public static ProjectItem FromFile(ProcessSession processSession, String filePath, DirectoryItem parent)
        {
            try
            {
                if (!File.Exists(filePath))
                {
                    throw new Exception("File does not exist: " + filePath);
                }

                using (FileStream fileStream = new FileStream(filePath, FileMode.Open, FileAccess.Read, FileShare.ReadWrite))
                {
                    if (fileStream.Length == 0)
                    {
                        return null;
                    }

                    Type type = null;

                    switch (new FileInfo(filePath).Extension.ToLower())
                    {
                        case ScriptItem.Extension:
                            type = typeof(ScriptItem);
                            break;
                        case PointerItem.Extension:
                            type = typeof(PointerItem);
                            break;
                        case InstructionItem.Extension:
                            type = typeof(InstructionItem);
                            break;
                        case DotNetItem.Extension:
                            type = typeof(DotNetItem);
                            break;
                        case JavaItem.Extension:
                            type = typeof(JavaItem);
                            break;
                        default:
                            return null;
                    }

                    DataContractJsonSerializer serializer = new DataContractJsonSerializer(type);

                    ProjectItem projectItem = serializer.ReadObject(fileStream) as ProjectItem;

                    // Bypass setters to avoid triggering write-back to disk
                    projectItem.name = Path.GetFileNameWithoutExtension(filePath);
                    projectItem.parent = parent;
                    projectItem.processSession = processSession;

                    return projectItem;
                }
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error loading file", ex);
                throw ex;
            }
        }

        /// <summary>
        /// Gets the extension for this project item.
        /// </summary>
        /// <returns>The extension for this project item.</returns>
        public virtual String GetExtension()
        {
            return String.Empty;
        }

        /// <summary>
        /// Saves this project item by serializing it to disk.
        /// </summary>
        public virtual void Save()
        {
            try
            {
                using (FileStream fileStream = new FileStream(this.FullPath, FileMode.Create, FileAccess.Write, FileShare.ReadWrite))
                {
                    DataContractJsonSerializer serializer = new DataContractJsonSerializer(this.GetType());
                    serializer.WriteObject(fileStream, this);
                }
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error saving file", ex);
            }
        }

        /// <summary>
        /// Invoked when this object is deserialized.
        /// </summary>
        /// <param name="streamingContext">Streaming context.</param>
        [OnDeserialized]
        public void OnDeserialized(StreamingContext streamingContext)
        {
            this.ActivationLock = new Object();
        }

        /// <summary>
        /// Updates this project item. Resolves addresses and values.
        /// </summary>
        public virtual void Update()
        {
        }

        /// <summary>
        /// Clones the project item.
        /// </summary>
        /// <param name="rename">A value indicating whether to rename this project item to a default after cloning.</param>
        /// <returns>The clone of the project item.</returns>
        public virtual ProjectItem Clone(Boolean rename)
        {
            // Serialize this project item to a byte array
            using (MemoryStream serializeMemoryStream = new MemoryStream())
            {
                DataContractJsonSerializer serializer = new DataContractJsonSerializer(typeof(ProjectItem));
                serializer.WriteObject(serializeMemoryStream, this);

                // Deserialize the array to clone the item
                using (MemoryStream deserializeMemoryStream = new MemoryStream(serializeMemoryStream.ToArray()))
                {
                    ProjectItem result = serializer.ReadObject(deserializeMemoryStream) as ProjectItem;

                    if (rename)
                    {
                        result.Name = this.MakeNameUnique(this.Name);
                    }

                    return result;
                }
            }
        }

        /// <summary>
        /// Updates the hotkey, bypassing setters to avoid triggering view updates.
        /// </summary>
        /// <param name="hotkey">The hotkey for this project item.</param>
        public void LoadHotkey(Hotkey hotkey)
        {
            this.hotkey = hotkey;

            this.HotKey?.SetCallBackFunction(() => this.IsActivated = !this.IsActivated);
        }

        /// <summary>
        /// Disposes of this project item.
        /// </summary>
        public void Dispose()
        {
            this.HotKey?.Dispose();
        }

        /// <summary>
        /// Event received when a key is released.
        /// </summary>
        /// <param name="key">The key that was released.</param>
        public void OnKeyPress(Key key)
        {
        }

        /// <summary>
        /// Event received when a key is down.
        /// </summary>
        /// <param name="key">The key that is down.</param>
        public void OnKeyRelease(Key key)
        {
        }

        /// <summary>
        /// Event received when a key is down.
        /// </summary>
        /// <param name="key">The key that is down.</param>
        public void OnKeyDown(Key key)
        {
        }

        /// <summary>
        /// Event received when a set of keys are down.
        /// </summary>
        /// <param name="pressedKeys">The down keys.</param>
        public void OnUpdateAllDownKeys(HashSet<Key> pressedKeys)
        {
            if (this.HotKey is KeyboardHotkey)
            {
                KeyboardHotkey keyboardHotkey = this.HotKey as KeyboardHotkey;
            }
        }

        /// <summary>
        /// Indicates that a given property in this project item has changed.
        /// </summary>
        /// <param name="propertyName">The name of the changed property.</param>
        protected void RaisePropertyChanged(String propertyName)
        {
            this.PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }

        /// <summary>
        /// Deactivates this item without triggering the <see cref="OnActivationChanged" /> function.
        /// </summary>
        protected void ResetActivation()
        {
            lock (this.ActivationLock)
            {
                this.isActivated = false;
                this.RaisePropertyChanged(nameof(this.IsActivated));
            }
        }

        /// <summary>
        /// Called when the activation state changes.
        /// </summary>
        protected virtual void OnActivationChanged()
        {
        }

        /// <summary>
        /// Overridable function indicating if this script can be activated.
        /// </summary>
        /// <returns>True if the script can be activated, otherwise false.</returns>
        protected virtual Boolean IsActivatable()
        {
            return true;
        }

        /// <summary>
        /// Renames this project item. If there is a conflict, the provided name may change to ensure uniqueness.
        /// </summary>
        /// <param name="newName">The new name for this project item.</param>
        private void Rename(String newName)
        {
            if (!this.HasAssociatedFileOrFolder || this.Name.Equals(newName, StringComparison.OrdinalIgnoreCase))
            {
                this.name = newName;
                return;
            }

            newName = this.MakeNameUnique(newName);
            String newPath = this.GetFilePathForName(newName);

            // Attempt to move the existing associated file if possible
            try
            {
                File.Move(this.FullPath, newPath);
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error moving existing project file during rename. The old file may still exist.", ex);
            }

            this.name = newName;
            this.RaisePropertyChanged(nameof(this.Name));
            this.Save();
        }

        /// <summary>
        /// Resolves the name conflict for this unassociated project item.
        /// </summary>
        /// <param name="originalName">The project name to make unique on disk.</param>
        /// <returns>The resolved name, which appends a number at the end of the name to ensure uniqueness.</returns>
        private String MakeNameUnique(String originalName)
        {
            String newName = originalName;

            if (this.Parent == null || this.Parent.ChildItems == null || !this.HasAssociatedFileOrFolder)
            {
                return newName;
            }

            String newFilePath = this.GetFilePathForName(newName);

            try
            {
                if (this.Parent.ChildItems.Any(childItem => childItem.Value?.Name?.Equals(newName, StringComparison.OrdinalIgnoreCase) ?? false))
                {
                    // Find all files that match the pattern of {newfilename #}, and extract the numbers
                    IEnumerable<String> numberedSuffixStrings = this.Parent.ChildItems
                        .Where(childItem => childItem.Value?.Name?.StartsWith(newName, StringComparison.OrdinalIgnoreCase) ?? false)
                        .Select(childItem => childItem.Value.Name.Substring(newName.Length).Trim());
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
            }

            return newName;
        }

        /// <summary>
        /// Gets the file path for the given project name.
        /// </summary>
        /// <param name="name">The proejct name.</param>
        /// <returns>The file path for the given project name.</returns>
        private String GetFilePathForName(String name)
        {
            return Path.Combine(this.Parent?.FullPath ?? ProjectSettings.ProjectRoot, name + this.GetExtension());
        }
    }
    //// End class
}
//// End namespace