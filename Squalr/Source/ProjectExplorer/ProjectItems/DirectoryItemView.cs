namespace Squalr.Source.ProjectExplorer.ProjectItems
{
    using Squalr.Engine.Common.DataStructures;
    using Squalr.Engine.Projects.Items;
    using Squalr.Source.Controls;
    using System;
    using System.ComponentModel;
    using static Squalr.Engine.Projects.Items.DirectoryItem;

    /// <summary>
    /// Decorates the base project item class with annotations for use in the view.
    /// </summary>
    public class DirectoryItemView : ProjectItemView
    {
        /// <summary>
        /// A value indicating whether this directory item is expanded.
        /// </summary>
        private Boolean isExpanded;

        /// <summary>
        /// The directory item associated with this view.
        /// </summary>
        private DirectoryItem directoryItem;

        /// <summary>
        /// The child project items contained by the directory item associated with this view.
        /// </summary>
        private FullyObservableCollection<ProjectItem> childItems;

        /// <summary>
        /// Initializes a new instance of the <see cref="DirectoryItemView"/> class.
        /// </summary>
        /// <param name="directoryItem">The directory item associated with this view.</param>
        public DirectoryItemView(DirectoryItem directoryItem)
        {
            this.childItems = new FullyObservableCollection<ProjectItem>();
            this.DirectoryItem = directoryItem;

            if (this.DirectoryItem != null)
            {
                foreach (ProjectItem projectItem in this.DirectoryItem.ChildItems.Values)
                {
                    this.childItems.Add(projectItem);
                }

                this.DirectoryItem.ProjectItemAddedEvent = this.ProjectItemAdded;
                this.DirectoryItem.ProjectItemDeletedEvent = this.ProjectItemDeleted;
            }
        }

        /// <summary>
        /// Gets or sets the description for this object.
        /// </summary>
        [SortedCategory(SortedCategory.CategoryType.Common), DisplayName("Name"), Description("The name of this folder")]
        public String Name
        {
            get
            {
                return this.DirectoryItem.Name;
            }

            set
            {
                this.DirectoryItem.Name = value;
            }
        }

        /// <summary>
        /// Gets the directory path associated with the directory item associated with this view.
        /// </summary>
        public String FilePath
        {
            get
            {
                return this.DirectoryItem.FullPath;
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether this directory item is expanded.
        /// </summary>
        public override Boolean IsExpanded
        {
            get
            {
                return this.isExpanded;
            }

            set
            {
                this.isExpanded = value;
                this.RaisePropertyChanged(nameof(this.IsExpanded));
            }
        }

        /// <summary>
        /// Gets the child project items contained by the directory item associated with this view.
        /// </summary>
        [Browsable(false)]
        public override FullyObservableCollection<ProjectItem> ChildItems
        {
            get
            {
                return this.childItems;
            }
        }

        /// <summary>
        /// Gets or sets the directory item associated with this view.
        /// </summary>
        [Browsable(false)]
        private DirectoryItem DirectoryItem
        {
            get
            {
                return this.directoryItem;
            }

            set
            {
                this.directoryItem = value;
                this.ProjectItem = value;
                this.RaisePropertyChanged(nameof(this.DirectoryItem));
            }
        }

        /// <summary>
        /// Adds a child project item to the directory item associated with this view.
        /// </summary>
        /// <param name="projectItem">The project item to add to the directory item associated with this view.</param>
        public void AddChild(ProjectItem projectItem)
        {
            this.DirectoryItem?.AddChild(projectItem);
            this.IsExpanded = true;
            this.RaisePropertyChanged(nameof(this.ChildItems));
        }

        /// <summary>
        /// Removes a child project item to the directory item associated with this view.
        /// </summary>
        /// <param name="projectItem">The project item to remove from the directory item associated with this view.</param>
        public void RemoveChild(ProjectItem projectItem)
        {
            this.DirectoryItem?.DeleteChild(projectItem);
            this.RaisePropertyChanged(nameof(this.ChildItems));
        }

        /// <summary>
        /// An event that is fired when a project item is deleted from the directory item associated with this view.
        /// </summary>
        private void ProjectItemDeleted(ProjectItem projectItem)
        {
            this.childItems.Remove(projectItem);
            this.RaisePropertyChanged(nameof(this.ChildItems));
        }

        /// <summary>
        /// An event that is fired when a project item is added to the directory item associated with this view.
        /// </summary>
        private void ProjectItemAdded(ProjectItem projectItem)
        {
            this.childItems.Add(projectItem);
            this.RaisePropertyChanged(nameof(this.ChildItems));
        }
    }
    //// End class
}
//// End namespace