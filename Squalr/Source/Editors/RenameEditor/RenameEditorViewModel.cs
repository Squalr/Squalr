﻿namespace Squalr.Source.Editors.RenameEditor
{
    using GalaSoft.MvvmLight;
    using Squalr.Engine.Projects.Items;
    using System;
    using System.Threading;
    using System.Windows;

    /// <summary>
    /// View model for the Rename Editor.
    /// </summary>
    public class RenameEditorViewModel : ViewModelBase
    {
        /// <summary>
        /// Singleton instance of the <see cref="RenameEditorViewModel" /> class.
        /// </summary>
        private static Lazy<RenameEditorViewModel> RenameEditorViewModelInstance = new Lazy<RenameEditorViewModel>(
                () => { return new RenameEditorViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// Initializes a new instance of the <see cref="RenameEditorViewModel" /> class.
        /// </summary>
        public RenameEditorViewModel()
        {
        }

        /// <summary>
        /// Gets or sets the name being edited.
        /// </summary>
        public String NewName { get; set; }

        /// <summary>
        /// Gets a singleton instance of the <see cref="RenameEditorViewModel" /> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static RenameEditorViewModel GetInstance()
        {
            return RenameEditorViewModel.RenameEditorViewModelInstance.Value;
        }

        public void ShowDialog(params ProjectItem[] projectItems)
        {
            if (projectItems.Length <= 0 || projectItems[0] == null)
            {
                return;
            }

            View.Editors.RenameEditor valueEditor = new View.Editors.RenameEditor(projectItems[0]) { Owner = Application.Current.MainWindow };

            this.NewName = projectItems[0].Name;
            this.RaisePropertyChanged(nameof(this.NewName));

            if (valueEditor.ShowDialog() == true)
            {
                foreach (ProjectItem projectItem in projectItems)
                {
                    if (projectItem != null)
                    {
                        projectItem.Name = this.NewName;
                    }
                }
            }
        }
    }
    //// End class
}
//// End namespace