namespace Squalr.Source.PropertyViewer
{
    using Squalr.Source.Controls;
    using Squalr.Source.Docking;
    using System;
    using System.Collections.Generic;
    using System.Drawing;
    using System.Linq;
    using System.Reflection;
    using System.Threading;
    using System.Windows.Forms;
    using System.Windows.Forms.Integration;

    /// <summary>
    /// View model for the Property Viewer.
    /// </summary>
    public class PropertyViewerViewModel : ToolViewModel
    {
        /// <summary>
        /// Singleton instance of the <see cref="PropertyViewerViewModel" /> class.
        /// </summary>
        private static Lazy<PropertyViewerViewModel> propertyViewerViewModelInstance = new Lazy<PropertyViewerViewModel>(
                () => { return new PropertyViewerViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// The property grid to display selected objects.
        /// </summary>
        private PropertyGrid propertyGrid = new PropertyGrid();

        /// <summary>
        /// The objects being viewed.
        /// </summary>
        private Object[] targetObjects;

        /// <summary>
        /// Prevents a default instance of the <see cref="PropertyViewerViewModel" /> class from being created.
        /// </summary>
        private PropertyViewerViewModel() : base("Property Viewer")
        {
            this.ObserverLock = new Object();
            this.PropertyViewerObservers = new List<IPropertyViewerObserver>();

            // Use reflection to set all propertygrid colors to dark, since some are otherwise not publically accessible
            PropertyInfo[] allProperties = this.propertyGrid.GetType().GetProperties();
            IEnumerable<PropertyInfo> colorProperties = allProperties.Select(x => x).Where(x => x.PropertyType == typeof(Color));

            foreach (PropertyInfo propertyInfo in colorProperties)
            {
                propertyInfo.SetValue(this.propertyGrid, DarkBrushes.SqualrColorPanel, null);
            }

            this.propertyGrid.BackColor = DarkBrushes.SqualrColorPanel;
            this.propertyGrid.CommandsBackColor = DarkBrushes.SqualrColorPanel;
            this.propertyGrid.HelpBackColor = DarkBrushes.SqualrColorPanel;
            this.propertyGrid.SelectedItemWithFocusBackColor = DarkBrushes.SqualrColorPanel;
            this.propertyGrid.ViewBackColor = DarkBrushes.SqualrColorPanel;

            this.propertyGrid.CommandsActiveLinkColor = DarkBrushes.SqualrColorPanel;
            this.propertyGrid.CommandsDisabledLinkColor = DarkBrushes.SqualrColorPanel;

            this.propertyGrid.CategorySplitterColor = DarkBrushes.SqualrColorWhite;

            this.propertyGrid.CommandsBorderColor = DarkBrushes.SqualrColorFrame;
            this.propertyGrid.HelpBorderColor = DarkBrushes.SqualrColorFrame;
            this.propertyGrid.ViewBorderColor = DarkBrushes.SqualrColorFrame;

            this.propertyGrid.CategoryForeColor = DarkBrushes.SqualrColorWhite;
            this.propertyGrid.CommandsForeColor = DarkBrushes.SqualrColorWhite;
            this.propertyGrid.DisabledItemForeColor = DarkBrushes.SqualrColorWhite;
            this.propertyGrid.HelpForeColor = DarkBrushes.SqualrColorWhite;
            this.propertyGrid.SelectedItemWithFocusForeColor = DarkBrushes.SqualrColorWhite;
            this.propertyGrid.ViewForeColor = DarkBrushes.SqualrColorWhite;

            DockingViewModel.GetInstance().RegisterViewModel(this);
        }

        /// <summary>
        /// Gets the objects being viewed.
        /// </summary>
        public Object[] TargetObjects
        {
            get
            {
                return this.targetObjects;
            }

            private set
            {
                this.targetObjects = value?.Where(x => x != null)?.ToArray();

                ControlThreadingHelper.InvokeControlAction(this.propertyGrid, () => { this.propertyGrid.SelectedObjects = targetObjects == null ? new Object[] { } : targetObjects; });

                this.RaisePropertyChanged(nameof(this.TargetObjects));
                this.NotifyObservers();
            }
        }

        /// <summary>
        /// Gets a hosting container for the property grid windows form object. This is done because there is no good WPF equivalent of this control.
        /// Fortunately, Windows Forms has a .Net Core implementation, so we do not rely on .Net Framework at all for this.
        /// </summary>
        public WindowsFormsHost WindowsFormsHost
        {
            get
            {
                return new WindowsFormsHost() { Child = this.propertyGrid };
            }
        }

        /// <summary>
        /// Gets or sets a lock that controls access to observing classes.
        /// </summary>
        private Object ObserverLock { get; set; }

        /// <summary>
        /// Gets or sets objects observing changes in the selected objects.
        /// </summary>
        private List<IPropertyViewerObserver> PropertyViewerObservers { get; set; }

        /// <summary>
        /// Gets a singleton instance of the <see cref="PropertyViewerViewModel"/> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static PropertyViewerViewModel GetInstance()
        {
            return PropertyViewerViewModel.propertyViewerViewModelInstance.Value;
        }

        /// <summary>
        /// Subscribes the given object to changes in the selected objects.
        /// </summary>
        /// <param name="propertyViewerObserver">The object to observe selected objects changes.</param>
        public void Subscribe(IPropertyViewerObserver propertyViewerObserver)
        {
            lock (this.ObserverLock)
            {
                if (!this.PropertyViewerObservers.Contains(propertyViewerObserver))
                {
                    this.PropertyViewerObservers.Add(propertyViewerObserver);
                    propertyViewerObserver.Update(this.TargetObjects);
                }
            }
        }

        /// <summary>
        /// Unsubscribes the given object from changes in the selected objects.
        /// </summary>
        /// <param name="propertyViewerObserver">The object to observe selected objects changes.</param>
        public void Unsubscribe(IPropertyViewerObserver propertyViewerObserver)
        {
            lock (this.ObserverLock)
            {
                if (this.PropertyViewerObservers.Contains(propertyViewerObserver))
                {
                    this.PropertyViewerObservers.Remove(propertyViewerObserver);
                }
            }
        }

        /// <summary>
        /// Sets the objects being viewed.
        /// </summary>
        /// <param name="targetObjects">The objects to view.</param>
        public void SetTargetObjects(params Object[] targetObjects)
        {
            this.TargetObjects = targetObjects;
        }

        /// <summary>
        /// Notify all observing objects of a change in the selected objects.
        /// </summary>
        private void NotifyObservers()
        {
            lock (this.ObserverLock)
            {
                foreach (IPropertyViewerObserver observer in this.PropertyViewerObservers)
                {
                    observer.Update(this.TargetObjects);
                }
            }
        }
    }
    //// End class
}
//// End namespace