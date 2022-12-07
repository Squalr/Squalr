namespace Squalr.Source.Scanning
{
    using GalaSoft.MvvmLight.Command;
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Logging;
    using Squalr.Engine.Scanning;
    using Squalr.Engine.Scanning.Scanners;
    using Squalr.Engine.Scanning.Scanners.Constraints;
    using Squalr.Engine.Scanning.Snapshots;
    using Squalr.Source.Docking;
    using Squalr.Source.ScanResults;
    using Squalr.Source.Tasks;
    using System;
    using System.Threading;
    using System.Windows.Input;

    /// <summary>
    /// View model for the Manual Scanner.
    /// </summary>
    public class ManualScannerViewModel : ToolViewModel
    {
        /// <summary>
        /// Singleton instance of the <see cref="ManualScannerViewModel" /> class.
        /// </summary>
        private static Lazy<ManualScannerViewModel> manualScannerViewModelInstance = new Lazy<ManualScannerViewModel>(
                () => { return new ManualScannerViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// The current scan constraint.
        /// </summary>
        private ScanConstraint scanConstraint;

        /// <summary>
        /// Prevents a default instance of the <see cref="ManualScannerViewModel" /> class from being created.
        /// </summary>
        private ManualScannerViewModel() : base("Manual Scanner")
        {
            this.StartScanCommand = new RelayCommand(() => this.StartScan(), () => true);

            // Not async for faster UI feedback
            this.UpdateActiveValueCommand = new RelayCommand<Object>((newValue) => this.UpdateActiveValue(newValue), (newValue) => true);
            this.UpdateActiveArgsCommand = new RelayCommand<Object>((newArgs) => this.UpdateActiveArgs(newArgs), (newArgs) => true);
            this.SelectChangedCommand = new RelayCommand(() => this.ChangeScanConstraintSelection(ScanConstraint.ConstraintType.Changed), () => true);
            this.SelectDecreasedCommand = new RelayCommand(() => this.ChangeScanConstraintSelection(ScanConstraint.ConstraintType.Decreased), () => true);
            this.SelectDecreasedByXCommand = new RelayCommand(() => this.ChangeScanConstraintSelection(ScanConstraint.ConstraintType.DecreasedByX), () => true);
            this.SelectEqualCommand = new RelayCommand(() => this.ChangeScanConstraintSelection(ScanConstraint.ConstraintType.Equal), () => true);
            this.SelectGreaterThanCommand = new RelayCommand(() => this.ChangeScanConstraintSelection(ScanConstraint.ConstraintType.GreaterThan), () => true);
            this.SelectGreaterThanOrEqualCommand = new RelayCommand(() => this.ChangeScanConstraintSelection(ScanConstraint.ConstraintType.GreaterThanOrEqual), () => true);
            this.SelectIncreasedCommand = new RelayCommand(() => this.ChangeScanConstraintSelection(ScanConstraint.ConstraintType.Increased), () => true);
            this.SelectIncreasedByXCommand = new RelayCommand(() => this.ChangeScanConstraintSelection(ScanConstraint.ConstraintType.IncreasedByX), () => true);
            this.SelectLessThanCommand = new RelayCommand(() => this.ChangeScanConstraintSelection(ScanConstraint.ConstraintType.LessThan), () => true);
            this.SelectLessThanOrEqualCommand = new RelayCommand(() => this.ChangeScanConstraintSelection(ScanConstraint.ConstraintType.LessThanOrEqual), () => true);
            this.SelectNotEqualCommand = new RelayCommand(() => this.ChangeScanConstraintSelection(ScanConstraint.ConstraintType.NotEqual), () => true);
            this.SelectUnchangedCommand = new RelayCommand(() => this.ChangeScanConstraintSelection(ScanConstraint.ConstraintType.Unchanged), () => true);

            this.ScanConstraint = new ScanConstraint(ScanConstraint.ConstraintType.Equal, ScannableType.Int32);

            // Not registering this as a dockable window, since it is just part of the top bar now
            // DockingViewModel.GetInstance().RegisterViewModel(this);
        }

        /// <summary>
        /// Gets the command begin the scan.
        /// </summary>
        public ICommand StartScanCommand { get; private set; }

        /// <summary>
        /// Gets the command to update the value of the active scan constraint.
        /// </summary>
        public ICommand UpdateActiveValueCommand { get; private set; }

        /// <summary>
        /// Gets the command to update the args of the active scan constraint.
        /// </summary>
        public ICommand UpdateActiveArgsCommand { get; private set; }

        /// <summary>
        /// Gets the command to select the <see cref="ScanConstraint.ConstraintType.Changed"/> constraint.
        /// </summary>
        public ICommand SelectChangedCommand { get; private set; }

        /// <summary>
        /// Gets the command to select the <see cref="ScanConstraint.ConstraintType.Decreased"/> constraint.
        /// </summary>
        public ICommand SelectDecreasedCommand { get; private set; }

        /// <summary>
        /// Gets the command to select the <see cref="ScanConstraint.ConstraintType.DecreasedByX"/> constraint.
        /// </summary>
        public ICommand SelectDecreasedByXCommand { get; private set; }

        /// <summary>
        /// Gets the command to select the <see cref="ScanConstraint.ConstraintType.Equal"/> constraint.
        /// </summary>
        public ICommand SelectEqualCommand { get; private set; }

        /// <summary>
        /// Gets the command to select the <see cref="ScanConstraint.ConstraintType.GreaterThan"/> constraint.
        /// </summary>
        public ICommand SelectGreaterThanCommand { get; private set; }

        /// <summary>
        /// Gets the command to select the <see cref="ScanConstraint.ConstraintType.GreaterThanOrEqual"/> constraint.
        /// </summary>
        public ICommand SelectGreaterThanOrEqualCommand { get; private set; }

        /// <summary>
        /// Gets the command to select the <see cref="ScanConstraint.ConstraintType.Increased"/> constraint.
        /// </summary>
        public ICommand SelectIncreasedCommand { get; private set; }

        /// <summary>
        /// Gets the command to select the <see cref="ScanConstraint.ConstraintType.IncreasedByX"/> constraint.
        /// </summary>
        public ICommand SelectIncreasedByXCommand { get; private set; }

        /// <summary>
        /// Gets the command to select the <see cref="ScanConstraint.ConstraintType.LessThan"/> constraint.
        /// </summary>
        public ICommand SelectLessThanCommand { get; private set; }

        /// <summary>
        /// Gets the command to select the <see cref="ScanConstraint.ConstraintType.LessThanOrEqual"/> constraint.
        /// </summary>
        public ICommand SelectLessThanOrEqualCommand { get; private set; }

        /// <summary>
        /// Gets the command to select the <see cref="ScanConstraint.ConstraintType.NotEqual"/> constraint.
        /// </summary>
        public ICommand SelectNotEqualCommand { get; private set; }

        /// <summary>
        /// Gets the command to select the <see cref="ScanConstraint.ConstraintType.Unchanged"/> constraint.
        /// </summary>
        public ICommand SelectUnchangedCommand { get; private set; }

        /// <summary>
        /// Gets or sets the current scan constraint.
        /// </summary>
        public ScanConstraint ScanConstraint
        {
            get
            {
                return this.scanConstraint;
            }

            set
            {
                this.scanConstraint = value;
                this.UpdateAllProperties();
            }
        }

        /// <summary>
        /// Gets a value indicating whether the current scan constraint requires a value.
        /// </summary>
        public Boolean IsScanConstraintValued
        {
            get
            {
                return this.ScanConstraint?.IsValuedConstraint() ?? true;
            }
        }

        /// <summary>
        /// Gets a singleton instance of the <see cref="ManualScannerViewModel"/> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static ManualScannerViewModel GetInstance()
        {
            return ManualScannerViewModel.manualScannerViewModelInstance.Value;
        }

        /// <summary>
        /// Starts the scan using the current constraints.
        /// </summary>
        private void StartScan()
        {
            // Create a constraint manager that includes the current active constraint
            ScannableType dataType = ScanResultsViewModel.GetInstance().ActiveType;
            ScanConstraints scanConstraints = new ScanConstraints(dataType, this.ScanConstraint?.Clone(), ScanSettings.Alignment);

            if (!scanConstraints.IsValid())
            {
                Logger.Log(LogLevel.Warn, "Unable to start scan with given constraints");
                return;
            }

            try
            {
                // Collect values
                TrackableTask<Snapshot> valueCollectorTask = ValueCollector.CollectValues(
                    SessionManager.Session.OpenedProcess,
                    SessionManager.Session.SnapshotManager.GetActiveSnapshotCreateIfNone(SessionManager.Session.OpenedProcess, SessionManager.Session.DetectedEmulator),
                    TrackableTask.UniversalIdentifier);

                TaskTrackerViewModel.GetInstance().TrackTask(valueCollectorTask);

                // Perform manual scan on value collection complete
                valueCollectorTask.OnCompletedEvent += (completedValueCollectionTask) =>
                {
                    Snapshot snapshot = valueCollectorTask.Result;
                    TrackableTask<Snapshot> scanTask = ManualScanner.Scan(
                        snapshot,
                        scanConstraints,
                        TrackableTask.UniversalIdentifier);

                    TaskTrackerViewModel.GetInstance().TrackTask(scanTask);
                    SessionManager.Session.SnapshotManager.SaveSnapshot(scanTask.Result);
                };
            }
            catch (TaskConflictException)
            {
            }
        }

        /// <summary>
        /// Updates the value of the current scan constraint.
        /// </summary>
        /// <param name="newValue">The new value of the scan constraint.</param>
        private void UpdateActiveValue(Object newValue)
        {
            this.ScanConstraint.ConstraintValue = newValue;
            this.UpdateAllProperties();
        }

        /// <summary>
        /// Updates the args of the current scan constraint.
        /// </summary>
        /// <param name="newArgs">The new args of the scan constraint.</param>
        private void UpdateActiveArgs(Object newArgs)
        {
            this.ScanConstraint.ConstraintArgs = newArgs;
            this.UpdateAllProperties();
        }

        /// <summary>
        /// Changes the current scan constraint.
        /// </summary>
        /// <param name="constraint">The new scan constraint.</param>
        private void ChangeScanConstraintSelection(ScanConstraint.ConstraintType constraint)
        {
            this.ScanConstraint.Constraint = constraint;
            this.UpdateAllProperties();
        }

        /// <summary>
        /// Raises property changed events for all used properties. This is convenient since there are several interdependencies between these.
        /// </summary>
        private void UpdateAllProperties()
        {
            this.RaisePropertyChanged(nameof(this.ScanConstraint));
            this.RaisePropertyChanged(nameof(this.IsScanConstraintValued));
        }
    }
    //// End class
}
//// End namespace