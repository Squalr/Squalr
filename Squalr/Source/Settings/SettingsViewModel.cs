namespace Squalr.Source.Settings
{
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Projects;
    using Squalr.Engine.Scanning;
    using Squalr.Source.Docking;
    using System;
    using System.ComponentModel;
    using System.IO;
    using System.Threading;

    /// <summary>
    /// View model for the Settings.
    /// </summary>
    public class SettingsViewModel : ToolViewModel
    {
        /// <summary>
        /// Singleton instance of the <see cref="SettingsViewModel"/> class.
        /// </summary>
        private static Lazy<SettingsViewModel> settingsViewModelInstance = new Lazy<SettingsViewModel>(
                () => { return new SettingsViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// Prevents a default instance of the <see cref="SettingsViewModel"/> class from being created.
        /// </summary>
        private SettingsViewModel() : base("Settings")
        {
            // ProjectSettings.PropertyChanged += ProjectSettingsPropertyChanged;
            //  ScanSettings.PropertyChanged += ScanSettingsPropertyChanged;
            DockingViewModel.GetInstance().RegisterViewModel(this);
        }

        /// <summary>
        /// Gets or sets the root directory for all projects.
        /// </summary>
        public String ProjectRoot
        {
            get
            {
                String savedPath = ProjectSettings.ProjectRoot;

                if (!Directory.Exists(savedPath))
                {
                    savedPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments), "Squalr");
                    this.ProjectRoot = savedPath;
                }

                return ProjectSettings.ProjectRoot;
            }

            set
            {
                try
                {
                    if (!Directory.Exists(value))
                    {
                        Directory.CreateDirectory(value);
                    }
                }
                catch (Exception ex)
                {
                    Logger.Log(LogLevel.Error, "Unable to set project root", ex);
                }

                ProjectSettings.ProjectRoot = value;
                this.RaisePropertyChanged(nameof(this.ProjectRoot));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether Squalr should perform automatic updates.
        /// </summary>
        public Boolean AutomaticUpdates
        {
            get
            {
                return SqualrSettings.AutomaticUpdates;
            }

            set
            {
                SqualrSettings.AutomaticUpdates = value;
                this.RaisePropertyChanged(nameof(this.AutomaticUpdates));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not 'write' flags are required in retrieved virtual memory pages.
        /// </summary>
        public Boolean RequiredProtectionWrite
        {
            get
            {
                return ScanSettings.RequiredWrite;
            }

            set
            {
                ScanSettings.RequiredWrite = value;
                this.RaisePropertyChanged(nameof(this.RequiredProtectionWrite));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not 'execute' flags are required in retrieved virtual memory pages.
        /// </summary>
        public Boolean RequiredProtectionExecute
        {
            get
            {
                return ScanSettings.RequiredExecute;
            }

            set
            {
                ScanSettings.RequiredExecute = value;
                this.RaisePropertyChanged(nameof(this.RequiredProtectionExecute));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not 'copy on write' flags are required in retrieved virtual memory pages.
        /// </summary>
        public Boolean RequiredProtectionCopyOnWrite
        {
            get
            {
                return ScanSettings.RequiredCopyOnWrite;
            }

            set
            {
                ScanSettings.RequiredCopyOnWrite = value;
                this.RaisePropertyChanged(nameof(this.RequiredProtectionCopyOnWrite));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not retrieved virtual memory pages exclude those with 'write' flags.
        /// </summary>
        public Boolean ExcludedProtectionWrite
        {
            get
            {
                return ScanSettings.ExcludedWrite;
            }

            set
            {
                ScanSettings.ExcludedWrite = value;
                this.RaisePropertyChanged(nameof(this.ExcludedProtectionWrite));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not retrieved virtual memory pages exclude those with 'execute' flags.
        /// </summary>
        public Boolean ExcludedProtectionExecute
        {
            get
            {
                return ScanSettings.ExcludedExecute;
            }

            set
            {
                ScanSettings.ExcludedExecute = value;
                this.RaisePropertyChanged(nameof(this.ExcludedProtectionExecute));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not retrieved virtual memory pages exclude those with 'copy on write' flags.
        /// </summary>
        public Boolean ExcludedProtectionCopyOnWrite
        {
            get
            {
                return ScanSettings.ExcludedCopyOnWrite;
            }

            set
            {
                ScanSettings.ExcludedCopyOnWrite = value;
                this.RaisePropertyChanged(nameof(this.ExcludedProtectionCopyOnWrite));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not retrieved virtual memory pages allow 'none' memory type.
        /// </summary>
        public Boolean MemoryTypeNone
        {
            get
            {
                return ScanSettings.MemoryTypeNone;
            }

            set
            {
                ScanSettings.MemoryTypeNone = value;
                this.RaisePropertyChanged(nameof(this.MemoryTypeNone));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not retrieved virtual memory pages allow 'private' memory type.
        /// </summary>
        public Boolean MemoryTypePrivate
        {
            get
            {
                return ScanSettings.MemoryTypePrivate;
            }

            set
            {
                ScanSettings.MemoryTypePrivate = value;
                this.RaisePropertyChanged(nameof(this.MemoryTypePrivate));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not retrieved virtual memory pages allow 'mapped' memory type.
        /// </summary>
        public Boolean MemoryTypeMapped
        {
            get
            {
                return ScanSettings.MemoryTypeMapped;
            }

            set
            {
                ScanSettings.MemoryTypeMapped = value;
                this.RaisePropertyChanged(nameof(this.MemoryTypeMapped));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not retrieved virtual memory pages allow 'image' memory type.
        /// </summary>
        public Boolean MemoryTypeImage
        {
            get
            {
                return ScanSettings.MemoryTypeImage;
            }

            set
            {
                ScanSettings.MemoryTypeImage = value;
                this.RaisePropertyChanged(nameof(this.MemoryTypeImage));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not retrieved virtual memory pages must be in usermode range.
        /// </summary>
        public Boolean IsUserMode
        {
            get
            {
                return ScanSettings.IsUserMode;
            }

            set
            {
                ScanSettings.IsUserMode = value;
                this.RaisePropertyChanged(nameof(this.IsUserMode));
                this.RaisePropertyChanged(nameof(this.IsNotUserMode));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not retrieved virtual memory pages can be in any address range.
        /// </summary>
        public Boolean IsNotUserMode
        {
            get
            {
                return !ScanSettings.IsUserMode;
            }

            set
            {
                ScanSettings.IsUserMode = !value;
                this.RaisePropertyChanged(nameof(this.IsUserMode));
                this.RaisePropertyChanged(nameof(this.IsNotUserMode));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether the current alignment is 1.
        /// </summary>
        public Boolean IsAlignment1
        {
            get
            {
                return ScanSettings.Alignment == MemoryAlignment.Alignment1;
            }

            set
            {
                if (value)
                {
                    ScanSettings.Alignment = MemoryAlignment.Alignment1;
                }

                this.RaisePropertyChanged(nameof(this.IsAlignment1));
                this.RaisePropertyChanged(nameof(this.IsAlignment2));
                this.RaisePropertyChanged(nameof(this.IsAlignment4));
                this.RaisePropertyChanged(nameof(this.IsAlignment8));
                this.RaisePropertyChanged(nameof(this.IsAlignmentAuto));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether the current alignment is 1.
        /// </summary>
        public Boolean IsAlignment2
        {
            get
            {
                return ScanSettings.Alignment == MemoryAlignment.Alignment2;
            }

            set
            {
                if (value)
                {
                    ScanSettings.Alignment = MemoryAlignment.Alignment2;
                }

                this.RaisePropertyChanged(nameof(this.IsAlignment1));
                this.RaisePropertyChanged(nameof(this.IsAlignment2));
                this.RaisePropertyChanged(nameof(this.IsAlignment4));
                this.RaisePropertyChanged(nameof(this.IsAlignment8));
                this.RaisePropertyChanged(nameof(this.IsAlignmentAuto));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether the current alignment is 1.
        /// </summary>
        public Boolean IsAlignment4
        {
            get
            {
                return ScanSettings.Alignment == MemoryAlignment.Alignment4;
            }

            set
            {
                if (value)
                {
                    ScanSettings.Alignment = MemoryAlignment.Alignment4;
                }

                this.RaisePropertyChanged(nameof(this.IsAlignment1));
                this.RaisePropertyChanged(nameof(this.IsAlignment2));
                this.RaisePropertyChanged(nameof(this.IsAlignment4));
                this.RaisePropertyChanged(nameof(this.IsAlignment8));
                this.RaisePropertyChanged(nameof(this.IsAlignmentAuto));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether the current alignment is 8.
        /// </summary>
        public Boolean IsAlignment8
        {
            get
            {
                return ScanSettings.Alignment == MemoryAlignment.Alignment8;
            }

            set
            {
                if (value)
                {
                    ScanSettings.Alignment = MemoryAlignment.Alignment8;
                }

                this.RaisePropertyChanged(nameof(this.IsAlignment1));
                this.RaisePropertyChanged(nameof(this.IsAlignment2));
                this.RaisePropertyChanged(nameof(this.IsAlignment4));
                this.RaisePropertyChanged(nameof(this.IsAlignment8));
                this.RaisePropertyChanged(nameof(this.IsAlignmentAuto));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether the current alignment is automatic (matches data type).
        /// </summary>
        public Boolean IsAlignmentAuto
        {
            get
            {
                return ScanSettings.Alignment == MemoryAlignment.Auto;
            }

            set
            {
                if (value)
                {
                    ScanSettings.Alignment = MemoryAlignment.Auto;
                }

                this.RaisePropertyChanged(nameof(this.IsAlignment1));
                this.RaisePropertyChanged(nameof(this.IsAlignment2));
                this.RaisePropertyChanged(nameof(this.IsAlignment4));
                this.RaisePropertyChanged(nameof(this.IsAlignment8));
                this.RaisePropertyChanged(nameof(this.IsAlignmentAuto));
            }
        }

        /// <summary>
        /// Gets or sets a the interval of reupdating frozen values.
        /// </summary>
        public Int32 FreezeInterval
        {
            get
            {
                return ScanSettings.FreezeInterval;
            }

            set
            {
                ScanSettings.FreezeInterval = value;
                this.RaisePropertyChanged(nameof(this.FreezeInterval));
            }
        }

        /// <summary>
        /// Gets or sets a the interval between repeated scans.
        /// </summary>
        public Int32 RescanInterval
        {
            get
            {
                // TODO
                return 100;
            }

            set
            {
                // TODO
                this.RaisePropertyChanged(nameof(this.RescanInterval));
            }
        }

        /// <summary>
        /// Gets or sets a the interval between reading scan results.
        /// </summary>
        public Int32 ResultReadInterval
        {
            get
            {
                return ScanSettings.ResultReadInterval;
            }

            set
            {
                ScanSettings.ResultReadInterval = value;
                this.RaisePropertyChanged(nameof(this.ResultReadInterval));
            }
        }

        /// <summary>
        /// Gets or sets a the interval between reading values in the table.
        /// </summary>
        public Int32 TableReadInterval
        {
            get
            {
                return ScanSettings.TableReadInterval;
            }

            set
            {
                ScanSettings.TableReadInterval = value;
                this.RaisePropertyChanged(nameof(this.TableReadInterval));
            }
        }

        /// <summary>
        /// Gets or sets the virtual memory alignment required in scans.
        /// </summary>
        public MemoryAlignment Alignment
        {
            get
            {
                return ScanSettings.Alignment;
            }

            set
            {
                ScanSettings.Alignment = value;
                this.RaisePropertyChanged(nameof(this.Alignment));
            }
        }

        /// <summary>
        /// Gets or sets the start address of virtual memory scans.
        /// </summary>
        public UInt64 StartAddress
        {
            get
            {
                return ScanSettings.StartAddress;
            }

            set
            {
                ScanSettings.StartAddress = value;
                this.RaisePropertyChanged(nameof(this.StartAddress));
            }
        }

        /// <summary>
        /// Gets or sets the end address of virtual memory scans.
        /// </summary>
        public UInt64 EndAddress
        {
            get
            {
                return ScanSettings.EndAddress;
            }

            set
            {
                ScanSettings.EndAddress = value;
                this.RaisePropertyChanged(nameof(this.EndAddress));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not emulators are automatically detected.
        /// </summary>
        public Boolean IsEmulatorTypeAuto
        {
            get
            {
                return ScanSettings.EmulatorType == EmulatorType.AutoDetect;
            }

            set
            {
                ScanSettings.EmulatorType = EmulatorType.AutoDetect;
                this.RaisePropertyChanged(nameof(this.IsEmulatorTypeAuto));
                this.RaisePropertyChanged(nameof(this.IsEmulatorTypeNone));
                this.RaisePropertyChanged(nameof(this.IsEmulatorTypeDolphin));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not emulators are disabled.
        /// </summary>
        public Boolean IsEmulatorTypeNone
        {
            get
            {
                return ScanSettings.EmulatorType == EmulatorType.None;
            }

            set
            {
                ScanSettings.EmulatorType = EmulatorType.None;
                this.RaisePropertyChanged(nameof(this.IsEmulatorTypeAuto));
                this.RaisePropertyChanged(nameof(this.IsEmulatorTypeNone));
                this.RaisePropertyChanged(nameof(this.IsEmulatorTypeDolphin));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether or not the target process is a Dolphin emulator.
        /// </summary>
        public Boolean IsEmulatorTypeDolphin
        {
            get
            {
                return ScanSettings.EmulatorType == EmulatorType.Dolphin;
            }

            set
            {
                ScanSettings.EmulatorType = EmulatorType.Dolphin;
                this.RaisePropertyChanged(nameof(this.IsEmulatorTypeAuto));
                this.RaisePropertyChanged(nameof(this.IsEmulatorTypeNone));
                this.RaisePropertyChanged(nameof(this.IsEmulatorTypeDolphin));
            }
        }

        /// <summary>
        /// Gets or sets a value indicating whether scans should use multi-threading. Mostly for testing purposes, this should almost always be turned on.
        /// </summary>
        public Boolean UseMultiThreadScans
        {
            get
            {
                return ScanSettings.UseMultiThreadScans;
            }

            set
            {
                ScanSettings.UseMultiThreadScans = value;
                this.RaisePropertyChanged(nameof(this.UseMultiThreadScans));
            }
        }

        /// <summary>
        /// Gets a singleton instance of the <see cref="SettingsViewModel"/> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static SettingsViewModel GetInstance()
        {
            return SettingsViewModel.settingsViewModelInstance.Value;
        }

        private void ProjectSettingsPropertyChanged(Object sender, PropertyChangedEventArgs e)
        {
           // ProjectSettings.Save();
        }

        private void ScanSettingsPropertyChanged(Object sender, PropertyChangedEventArgs e)
        {
            // ScanSettings.Save();
        }
    }
    //// End class
}
//// End namespace