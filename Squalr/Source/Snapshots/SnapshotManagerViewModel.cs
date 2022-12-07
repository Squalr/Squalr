namespace Squalr.Source.Snapshots
{
    using GalaSoft.MvvmLight.Command;
    using Squalr.Engine.Scanning.Snapshots;
    using Squalr.Source.Docking;
    using System;
    using System.Collections.Generic;
    using System.Media;
    using System.Threading;
    using System.Windows.Input;

    /// <summary>
    /// View model for the Snapshot Manager.
    /// </summary>
    public class SnapshotManagerViewModel : ToolViewModel
    {
        /// <summary>
        /// Singleton instance of the <see cref="SnapshotManagerViewModel"/> class.
        /// </summary>
        private static Lazy<SnapshotManagerViewModel> snapshotManagerViewModelInstance = new Lazy<SnapshotManagerViewModel>(
                () => { return new SnapshotManagerViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// Prevents a default instance of the <see cref="SnapshotManagerViewModel"/> class from being created.
        /// </summary>
        private SnapshotManagerViewModel() : base("Snapshot Manager")
        {
            // Note: Not async to avoid updates slower than the perception threshold
            this.ClearSnapshotsCommand = new RelayCommand(() => SessionManager.Session?.SnapshotManager?.ClearSnapshots(), () => true);
            this.UndoSnapshotCommand = new RelayCommand(() => SessionManager.Session?.SnapshotManager?.UndoSnapshot(), () => true);
            this.RedoSnapshotCommand = new RelayCommand(() => SessionManager.Session?.SnapshotManager?.RedoSnapshot(), () => true);

            SessionManager.Session.SnapshotManager.OnSnapshotsUpdatedEvent += SnapshotManagerOnSnapshotsUpdatedEvent;
            SessionManager.Session.SnapshotManager.OnNewSnapshotEvent += NewSnapshotEvent;

            DockingViewModel.GetInstance().RegisterViewModel(this);
        }

        /// <summary>
        /// Gets a command to start a new scan.
        /// </summary>
        public ICommand ClearSnapshotsCommand { get; private set; }

        /// <summary>
        /// Gets a command to undo the last scan.
        /// </summary>
        public ICommand UndoSnapshotCommand { get; private set; }

        /// <summary>
        /// Gets a command to redo the last scan.
        /// </summary>
        public ICommand RedoSnapshotCommand { get; private set; }

        /// <summary>
        /// Gets the enumeration of snapshots in the snapshot manager.
        /// </summary>
        public IEnumerable<Snapshot> Snapshots
        {
            get
            {
                return SessionManager.Session?.SnapshotManager?.Snapshots;
            }
        }

        /// <summary>
        /// Gets the enumeration of snapshots in the snapshot manager.
        /// </summary>
        public IEnumerable<Snapshot> DeletedSnapshots
        {
            get
            {
                return SessionManager.Session?.SnapshotManager?.DeletedSnapshots;
            }
        }

        /// <summary>
        /// Gets a singleton instance of the <see cref="SnapshotManagerViewModel"/> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static SnapshotManagerViewModel GetInstance()
        {
            return SnapshotManagerViewModel.snapshotManagerViewModelInstance.Value;
        }

        private void SnapshotManagerOnSnapshotsUpdatedEvent(SnapshotManager snapshotManager)
        {
            this.RaisePropertyChanged(nameof(this.Snapshots));
            this.RaisePropertyChanged(nameof(this.DeletedSnapshots));
        }

        private void NewSnapshotEvent(SnapshotManager snapshotManager)
        {
            SystemSounds.Exclamation.Play();
        }
    }
    //// End class
}
//// End namespace