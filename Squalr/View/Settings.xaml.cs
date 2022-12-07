﻿namespace Squalr.View
{
    using Squalr.Engine.Common;
    using Squalr.Source.Controls;
    using Squalr.Source.Settings;
    using System;
    using System.ComponentModel;
    using System.Windows.Controls;

    /// <summary>
    /// Interaction logic for Settings.xaml.
    /// </summary>
    public partial class Settings : UserControl
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="Settings" /> class.
        /// </summary>
        public Settings()
        {
            this.InitializeComponent();

            this.ScanRangeStartHexDecBoxViewModel = this.ScanRangeStartHexDecBox.DataContext as HexDecBoxViewModel;
            this.ScanRangeStartHexDecBoxViewModel.PropertyChanged += this.StartRangeUpdated;
            this.ScanRangeStartHexDecBoxViewModel.DataType = ScannableType.UInt64;
            this.ScanRangeStartHexDecBoxViewModel.IsHex = true;
            this.ScanRangeStartHexDecBoxViewModel.SetValue(this.SettingsViewModel.StartAddress);

            this.ScanRangeEndHexDecBoxViewModel = this.ScanRangeEndHexDecBox.DataContext as HexDecBoxViewModel;
            this.ScanRangeEndHexDecBoxViewModel.DataType = ScannableType.UInt64;
            this.ScanRangeEndHexDecBoxViewModel.PropertyChanged += this.EndRangeUpdated;
            this.ScanRangeEndHexDecBoxViewModel.IsHex = true;
            this.ScanRangeEndHexDecBoxViewModel.SetValue(this.SettingsViewModel.EndAddress);

            this.FreezeIntervalHexDecBoxViewModel = this.FreezeIntervalHexDecBox.DataContext as HexDecBoxViewModel;
            this.FreezeIntervalHexDecBoxViewModel.DataType = ScannableType.Int32;
            this.FreezeIntervalHexDecBoxViewModel.PropertyChanged += this.FreezeIntervalUpdated;
            this.FreezeIntervalHexDecBoxViewModel.SetValue(this.SettingsViewModel.FreezeInterval);

            this.RescanIntervalHexDecBoxViewModel = this.RescanIntervalHexDecBox.DataContext as HexDecBoxViewModel;
            this.RescanIntervalHexDecBoxViewModel.DataType = ScannableType.Int32;
            this.RescanIntervalHexDecBoxViewModel.PropertyChanged += this.RescanIntervalUpdated;
            this.RescanIntervalHexDecBoxViewModel.SetValue(this.SettingsViewModel.RescanInterval);

            this.TableReadIntervalHexDecBoxViewModel = this.TableReadIntervalHexDecBox.DataContext as HexDecBoxViewModel;
            this.TableReadIntervalHexDecBoxViewModel.DataType = ScannableType.Int32;
            this.TableReadIntervalHexDecBoxViewModel.PropertyChanged += this.TableReadIntervalUpdated;
            this.TableReadIntervalHexDecBoxViewModel.SetValue(this.SettingsViewModel.TableReadInterval);

            this.ResultReadIntervalHexDecBoxViewModel = this.ResultReadIntervalHexDecBox.DataContext as HexDecBoxViewModel;
            this.ResultReadIntervalHexDecBoxViewModel.PropertyChanged += this.ResultReadIntervalUpdated;
            this.ResultReadIntervalHexDecBoxViewModel.DataType = ScannableType.Int32;
            this.ResultReadIntervalHexDecBoxViewModel.SetValue(this.SettingsViewModel.ResultReadInterval);
        }

        /// <summary>
        /// Gets the view model associated with this view.
        /// </summary>
        public SettingsViewModel SettingsViewModel
        {
            get
            {
                return this.DataContext as SettingsViewModel;
            }
        }

        /// <summary>
        /// Gets or sets the hex dec box for the scan range start.
        /// </summary>
        private HexDecBoxViewModel ScanRangeStartHexDecBoxViewModel { get; set; }

        /// <summary>
        /// Gets or sets the hex dec box for the scan range end.
        /// </summary>
        private HexDecBoxViewModel ScanRangeEndHexDecBoxViewModel { get; set; }

        /// <summary>
        /// Gets or sets the hex dec box for the freeze interval.
        /// </summary>
        private HexDecBoxViewModel FreezeIntervalHexDecBoxViewModel { get; set; }

        /// <summary>
        /// Gets or sets the hex dec box for the rescan interval.
        /// </summary>
        private HexDecBoxViewModel RescanIntervalHexDecBoxViewModel { get; set; }

        /// <summary>
        /// Gets or sets the hex dec box for the table read interval.
        /// </summary>
        private HexDecBoxViewModel TableReadIntervalHexDecBoxViewModel { get; set; }

        /// <summary>
        /// Gets or sets the hex dec box for the input correlation timeout.
        /// </summary>
        private HexDecBoxViewModel ResultReadIntervalHexDecBoxViewModel { get; set; }

        /// <summary>
        /// Invoked when the scan start range is changed, and informs the viewmodel.
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="args">Event args.</param>
        private void StartRangeUpdated(Object sender, PropertyChangedEventArgs args)
        {
            if (args.PropertyName == nameof(HexDecBoxViewModel.Text))
            {
                Object value = this.ScanRangeStartHexDecBoxViewModel.GetValue();

                if (value == null)
                {
                    return;
                }

                this.SettingsViewModel.StartAddress = (UInt64)value;
            }
        }

        /// <summary>
        /// Invoked when the scan end range is changed, and informs the viewmodel.
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="args">Event args.</param>
        private void EndRangeUpdated(Object sender, PropertyChangedEventArgs args)
        {
            if (args.PropertyName == nameof(HexDecBoxViewModel.Text))
            {
                Object value = this.ScanRangeEndHexDecBoxViewModel.GetValue();

                if (value == null)
                {
                    return;
                }

                this.SettingsViewModel.EndAddress = (UInt64)value;
            }
        }

        /// <summary>
        /// Invoked when the freeze interval is changed, and informs the viewmodel.
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="args">Event args.</param>
        private void FreezeIntervalUpdated(Object sender, PropertyChangedEventArgs args)
        {
            if (args.PropertyName == nameof(HexDecBoxViewModel.Text))
            {
                Object value = this.FreezeIntervalHexDecBoxViewModel.GetValue();

                if (value == null)
                {
                    return;
                }

                this.SettingsViewModel.FreezeInterval = (Int32)value;
            }
        }

        /// <summary>
        /// Invoked when the rescan interval is changed, and informs the viewmodel.
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="args">Event args.</param>
        private void RescanIntervalUpdated(Object sender, PropertyChangedEventArgs args)
        {
            if (args.PropertyName == nameof(HexDecBoxViewModel.Text))
            {
                Object value = this.RescanIntervalHexDecBoxViewModel.GetValue();

                if (value == null)
                {
                    return;
                }

                this.SettingsViewModel.RescanInterval = (Int32)value;
            }
        }

        /// <summary>
        /// Invoked when the table read interval is changed, and informs the viewmodel.
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="args">Event args.</param>
        private void TableReadIntervalUpdated(Object sender, PropertyChangedEventArgs args)
        {
            if (args.PropertyName == nameof(HexDecBoxViewModel.Text))
            {
                Object value = this.TableReadIntervalHexDecBoxViewModel.GetValue();

                if (value == null)
                {
                    return;
                }

                this.SettingsViewModel.TableReadInterval = (Int32)value;
            }
        }

        /// <summary>
        /// Invoked when the result read interval is changed, and informs the viewmodel.
        /// </summary>
        /// <param name="sender">Sending object.</param>
        /// <param name="args">Event args.</param>
        private void ResultReadIntervalUpdated(Object sender, PropertyChangedEventArgs args)
        {
            if (args.PropertyName == nameof(HexDecBoxViewModel.Text))
            {
                Object value = this.ResultReadIntervalHexDecBoxViewModel.GetValue();

                if (value == null)
                {
                    return;
                }

                this.SettingsViewModel.ResultReadInterval = (Int32)value;
            }
        }
    }
    //// End class
}
//// End namespace