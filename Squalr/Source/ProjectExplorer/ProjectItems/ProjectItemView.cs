namespace Squalr.Source.ProjectExplorer.ProjectItems
{
    using Squalr.Engine.Common.DataStructures;
    using Squalr.Engine.Projects.Items;
    using Squalr.Source.Mvvm.Converters;
    using System;
    using System.ComponentModel;
    using System.Globalization;
    using System.Windows.Media;

    public class ProjectItemView : INotifyPropertyChanged
    {
        private static readonly ProjectItemToIconConverter ProjectItemToIconConverter = new ProjectItemToIconConverter();

        private ProjectItem projectItem;

        private Boolean isSelected;

        /// <summary>
        /// An event that is raised when a property of this object changes.
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;

        [Browsable(false)]
        public virtual FullyObservableCollection<ProjectItem> ChildItems
        {
            get
            {
                return null;
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether this project item has been enabled.
        /// </summary>
        [Browsable(false)]
        public Boolean IsActivated
        {
            get
            {
                return this.ProjectItem.IsActivated;
            }

            set
            {
                this.ProjectItem.IsActivated = value;
                this.RaisePropertyChanged(nameof(this.IsActivated));
            }
        }

        [Browsable(false)]
        public Boolean IsSelected
        {
            get
            {
                return this.isSelected;
            }

            set
            {
                this.isSelected = value;
                this.RaisePropertyChanged(nameof(this.IsSelected));
            }
        }

        [Browsable(false)]
        public virtual Boolean IsExpanded
        {
            get
            {
                return false;
            }

            set
            {
            }
        }

        [Browsable(false)]
        public ProjectItem ProjectItem
        {
            get
            {
                return this.projectItem;
            }

            set
            {
                this.projectItem = value;

                if (this.projectItem != null)
                {
                    this.projectItem.PropertyChanged += this.ProjectItem_PropertyChanged;
                }

                this.RaisePropertyChanged(nameof(this.ProjectItem));
            }
        }

        /// <summary>
        /// Gets the icon associated with this project icon.
        /// </summary>
        [Browsable(false)]
        public ImageSource ImageSource
        {
            get
            {
                return ProjectItemToIconConverter.Convert(this, null, null, CultureInfo.InvariantCulture) as ImageSource;
            }
        }

        public virtual Object DisplayValue
        {
            get
            {
                return String.Empty;
            }

            set
            {
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

        private void ProjectItem_PropertyChanged(Object sender, PropertyChangedEventArgs e)
        {
            if (e.PropertyName == nameof(AddressItem.DataType))
            {
                this.RaisePropertyChanged(nameof(this.ImageSource));
            }

            if (e.PropertyName == nameof(PointerItem.PointerOffsets))
            {
                this.RaisePropertyChanged(nameof(this.ImageSource));
            }

            if (e.PropertyName == nameof(PointerItem.ModuleName))
            {
                this.RaisePropertyChanged(nameof(this.ImageSource));
            }
        }
    }
    //// End class
}
//// End namespace