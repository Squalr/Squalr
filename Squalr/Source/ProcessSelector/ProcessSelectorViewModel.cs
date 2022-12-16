namespace Squalr.Source.ProcessSelector
{
    using GalaSoft.MvvmLight.Command;
    using Squalr.Content;
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.Extensions;
    using Squalr.Engine.Processes;
    using Squalr.Engine.Scanning;
    using Squalr.Engine.Scanning.Scanners;
    using Squalr.Source.Docking;
    using Squalr.Source.Tasks;
    using System;
    using System.Collections.Generic;
    using System.Diagnostics;
    using System.Linq;
    using System.Threading;
    using System.Threading.Tasks;
    using System.Windows.Input;

    /// <summary>
    /// View model for the Process Selector.
    /// </summary>
    public class ProcessSelectorViewModel : ToolViewModel
    {
        /// <summary>
        /// Singleton instance of the <see cref="ProcessSelectorViewModel" /> class.
        /// </summary>
        private static Lazy<ProcessSelectorViewModel> processSelectorViewModelInstance = new Lazy<ProcessSelectorViewModel>(
                () => { return new ProcessSelectorViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// The list of running processes.
        /// </summary>
        private IEnumerable<Process> processList;

        /// <summary>
        /// Prevents a default instance of the <see cref="ProcessSelectorViewModel" /> class from being created.
        /// </summary>
        private ProcessSelectorViewModel() : base("Process Selector")
        {
            this.IconSource = Images.SelectProcess;
            this.RefreshProcessListCommand = new RelayCommand(() => Task.Run(() => this.RefreshProcessList()), () => true);
            this.SelectProcessCommand = new RelayCommand<Process>((process) => Task.Run(() => this.SelectProcess(process)), (process) => true);

            this.StartAutomaticProcessListRefresh();

            DockingViewModel.GetInstance().RegisterViewModel(this);
        }

        /// <summary>
        /// Gets the command to refresh the process list.
        /// </summary>
        public ICommand RefreshProcessListCommand { get; private set; }

        /// <summary>
        /// Gets the command to select a target process.
        /// </summary>
        public ICommand SelectProcessCommand { get; private set; }

        /// <summary>
        /// Gets or sets the list of processes running on the machine.
        /// </summary>
        public IEnumerable<Process> ProcessList
        {
            get
            {
                return this.processList;
            }

            set
            {
                this.processList = value;

                this.RaisePropertyChanged(nameof(this.ProcessList));
                this.RaisePropertyChanged(nameof(this.WindowedProcessList));
            }
        }

        /// <summary>
        /// Gets the processes with a window running on the machine, as well as the selected process.
        /// </summary>
        public IEnumerable<Process> WindowedProcessList
        {
            get
            {
                ProcessList.First().GetIcon();

                // Create a process list with the selected process at the top, and a detach option as the 2nd element
                return this.ProcessList?.Where(process => process.HasWindow() && (process?.Id ?? 0) != (this.SelectedProcess?.Id ?? 0)).Select(process => process)
                    .PrependIfNotNull(this.SelectedProcess == null ? null : DetachProcess.Instance)
                    .PrependIfNotNull(this.SelectedProcess)
                    .Distinct();
            }
        }

        /// <summary>
        /// Gets or sets the selected process.
        /// </summary>
        public Process SelectedProcess
        {
            get
            {
                return SessionManager.Session?.OpenedProcess;
            }

            set
            {
                if (value != this.SelectedProcess)
                {
                    SessionManager.Session.OpenedProcess = value;
                    this.RaisePropertyChanged(nameof(this.SelectedProcess));
                    this.RaisePropertyChanged(nameof(this.WindowedProcessList));

                    if (SessionManager.Session.OpenedProcess != null)
                    {
                        if (ScanSettings.EmulatorType == EmulatorType.AutoDetect)
                        {
                            TrackableTask<EmulatorType> emulatorDector = EmulatorDetector.DetectEmulator(value);

                            emulatorDector.OnCompletedEvent += (completedTask) =>
                            {
                                SessionManager.Session.DetectedEmulator = emulatorDector.Result;
                            };

                            TaskTrackerViewModel.GetInstance().TrackTask(emulatorDector);
                        }
                        else
                        {
                            SessionManager.Session.DetectedEmulator = ScanSettings.EmulatorType;
                        }
                    }
                }
            }
        }

        /// <summary>
        /// Gets the name of the selected process.
        /// </summary>
        public String ProcessName
        {
            get
            {
                String processName = SessionManager.Session?.OpenedProcess?.ProcessName;
                return String.IsNullOrEmpty(processName) ? "Please Select a Process" : processName;
            }
        }

        /// <summary>
        /// Gets a singleton instance of the <see cref="ProcessSelectorViewModel"/> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static ProcessSelectorViewModel GetInstance()
        {
            return processSelectorViewModelInstance.Value;
        }

        /// <summary>
        /// Recieves a process update.
        /// </summary>
        /// <param name="process">The newly selected process.</param>>
        public void Update(Process process)
        {
            // Raise event to update process name in the view
            this.RaisePropertyChanged(nameof(this.ProcessName));

            this.RefreshProcessList();
        }

        /// <summary>
        /// Called when the visibility of this tool is changed.
        /// </summary>
        protected override void OnVisibilityChanged()
        {
            if (this.IsVisible)
            {
                this.RefreshProcessList();
            }
        }

        /// <summary>
        /// Makes the target process selection.
        /// </summary>
        /// <param name="process">The process being selected.</param>
        private void SelectProcess(Process process)
        {
            this.SelectedProcess = process;
            this.IsVisible = false;
        }

        /// <summary>
        /// Starts a task that automatically refreshes the process list.
        /// </summary>
        private void StartAutomaticProcessListRefresh()
        {
            Task.Run(async () =>
            {
                while (true)
                {
                    this.RefreshProcessList();
                    await Task.Delay(5000);
                }
            });
        }

        /// <summary>
        /// Refreshes the process list.
        /// </summary>
        private void RefreshProcessList()
        {
            this.ProcessList = ProcessQuery.Instance.GetProcesses();
        }
    }
    //// End class
}
//// End namespace