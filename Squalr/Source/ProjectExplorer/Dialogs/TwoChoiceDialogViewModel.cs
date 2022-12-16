namespace Squalr.Source.ProjectExplorer.Dialogs
{
    using GalaSoft.MvvmLight;
    using Squalr.View.Dialogs;
    using System;
    using System.Media;
    using System.Threading;
    using System.Windows;

    /// <summary>
    /// The view model for the project deletion dialog.
    /// </summary>
    public class TwoChoiceDialogViewModel : ViewModelBase
    {
        /// <summary>
        /// Singleton instance of the <see cref="TwoChoiceDialogViewModel" /> class.
        /// </summary>
        private static readonly Lazy<TwoChoiceDialogViewModel> TwoChoiceDialogViewModelInstance = new Lazy<TwoChoiceDialogViewModel>(
                () => { return new TwoChoiceDialogViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        private String headerText = "Preview Header";

        private String bodyText = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.";

        private String optionOkayText = "Okay";

        private String optionCancelText = "Cancel";

        private TwoChoiceDialogViewModel() : base()
        {
        }

        public String HeaderText
        {
            get
            {
                return this.headerText;
            }

            set
            {
                this.headerText = value;
                this.RaisePropertyChanged(nameof(this.HeaderText));
            }
        }

        public String BodyText
        {
            get
            {
                return this.bodyText;
            }

            set
            {
                this.bodyText = value;
                this.RaisePropertyChanged(nameof(this.BodyText));
            }
        }

        public String OptionOkayText
        {
            get
            {
                return this.optionOkayText;
            }

            set
            {
                this.optionOkayText = value;
                this.RaisePropertyChanged(nameof(this.OptionOkayText));
            }
        }

        public String OptionCancelText
        {
            get
            {
                return this.optionCancelText;
            }

            set
            {
                this.optionCancelText = value;
                this.RaisePropertyChanged(nameof(this.OptionCancelText));
            }
        }

        /// <summary>
        /// Gets a singleton instance of the <see cref="TwoChoiceDialogViewModel" /> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static TwoChoiceDialogViewModel GetInstance()
        {
            return TwoChoiceDialogViewModel.TwoChoiceDialogViewModelInstance.Value;
        }

        /// <summary>
        /// Shows a generic two choice dialog box with the given parameters.
        /// </summary>
        /// <param name="owner">The window that owns this dialog.</param>
        /// <param name="headerText">The dialog header text.</param>
        /// <param name="bodyText">The dialog body text.</param>
        /// <param name="optionOkayText">The dialog 'Okay' button text.</param>
        /// <param name="optionCancelText">The dialog 'Cancel' button text.</param>
        /// <returns></returns>
        public Boolean ShowDialog(Window owner, String headerText, String bodyText, String optionOkayText, String optionCancelText)
        {
            this.HeaderText = headerText;
            this.BodyText = bodyText;
            this.OptionOkayText = optionOkayText;
            this.OptionCancelText = optionCancelText;

            TwoChoiceDialog twoChoiceDialog = new TwoChoiceDialog() { Owner = owner };

            SystemSounds.Asterisk.Play();

            // Explicit compare against true since this is nullable
            return twoChoiceDialog.ShowDialog() == true;
        }
    }
    //// End class
}
//// End namespace