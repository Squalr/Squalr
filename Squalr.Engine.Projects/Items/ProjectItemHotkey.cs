namespace Squalr.Engine.Projects.Items
{
    using Squalr.Engine.Input.HotKeys;
    using System;
    using System.ComponentModel;
    using System.Runtime.Serialization;

    /// <summary>
    /// Defines a hotkey for a project item.
    /// </summary>
    [KnownType(typeof(Hotkey))]
    [DataContract]
    public class ProjectItemHotkey : INotifyPropertyChanged
    {
        /// <summary>
        /// The target guid, from which the target project is derived.
        /// </summary>
        private Guid projectItemGuid;

        /// <summary>
        /// The hotkey bound to the project item.
        /// </summary>
        private Hotkey hotkey;

        /// <summary>
        /// The stream command bound to the project item.
        /// </summary>
        private String streamCommand;

        /// <summary>
        /// Initializes a new instance of the <see cref="ProjectItemHotkey" /> class.
        /// </summary>
        /// <param name="hotkey">The initial hotkey bound to the project item.</param>
        /// <param name="projectItemGuid">The guid identifying the project item to which this hotkey is bound.</param>
        public ProjectItemHotkey(Hotkey hotkey, Guid projectItemGuid)
        {
            this.ProjectItemGuid = projectItemGuid;
            this.Hotkey = hotkey;
        }

        /// <summary>
        /// An event that is raised when a property of this object changes.
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;

        /// <summary>
        /// Gets or sets the target project item guid.
        /// </summary>
        [DataMember]
        public Guid ProjectItemGuid
        {
            get
            {
                return this.projectItemGuid;
            }

            set
            {
                this.projectItemGuid = value;
                this.NotifyPropertyChanged(nameof(this.ProjectItemGuid));
            }
        }

        /// <summary>
        /// Gets or sets the hotkey bound to the project item.
        /// </summary>
        [DataMember]
        public Hotkey Hotkey
        {
            get
            {
                return this.hotkey;
            }

            set
            {
                this.hotkey = value;
                this.NotifyPropertyChanged(nameof(this.Hotkey));
            }
        }

        /// <summary>
        /// Gets or sets the stream command bound to the project item.
        /// </summary>
        [DataMember]
        public String StreamCommand
        {
            get
            {
                return this.streamCommand;
            }

            set
            {
                this.streamCommand = value;
                this.NotifyPropertyChanged(nameof(this.StreamCommand));
            }
        }

        /// <summary>
        /// Indicates that a given property in this project item has changed.
        /// </summary>
        /// <param name="propertyName">The name of the changed property.</param>
        protected void NotifyPropertyChanged(String propertyName)
        {
            this.PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
    //// End class
}
//// End namespace