namespace Squalr.Source.Editors.RenameEditor
{
    using GalaSoft.MvvmLight;
    using Squalr.Engine.Projects.Items;
    using Squalr.View.Dialogs;
    using System;
    using System.Threading;
    using System.Windows;

    /// <summary>
    /// View model for the Rename Project Item Dialog.
    /// </summary>
    public class RenameProjectItemDialogViewModel : ViewModelBase
    {
        /// <summary>
        /// Singleton instance of the <see cref="RenameProjectItemDialogViewModel" /> class.
        /// </summary>
        private static Lazy<RenameProjectItemDialogViewModel> RenameEditorViewModelInstance = new Lazy<RenameProjectItemDialogViewModel>(
                () => { return new RenameProjectItemDialogViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// Initializes a new instance of the <see cref="RenameProjectItemDialogViewModel" /> class.
        /// </summary>
        public RenameProjectItemDialogViewModel()
        {
        }

        /// <summary>
        /// Gets or sets the name being edited.
        /// </summary>
        public String NewName { get; set; }

        /// <summary>
        /// Gets a singleton instance of the <see cref="RenameProjectItemDialogViewModel" /> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static RenameProjectItemDialogViewModel GetInstance()
        {
            return RenameProjectItemDialogViewModel.RenameEditorViewModelInstance.Value;
        }

        public void ShowDialog(params ProjectItem[] projectItems)
        {
            if (projectItems.Length <= 0 || projectItems[0] == null)
            {
                return;
            }

            this.NewName = projectItems[0].Name;
            this.RaisePropertyChanged(nameof(this.NewName));

            RenameProjectItemDialog renameDialog = new RenameProjectItemDialog() { Owner = Application.Current.MainWindow };

            if (renameDialog.ShowDialog() == true)
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