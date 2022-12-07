namespace Squalr.Source.ScanResults
{
    using GalaSoft.MvvmLight.Command;
    using Squalr.Engine.Common;
    using Squalr.Engine.Common.DataStructures;
    using Squalr.Engine.Projects.Items;
    using Squalr.Engine.Scanning.Scanners.Pointers.Structures;
    using Squalr.Source.Docking;
    using Squalr.Source.ProjectExplorer;
    using System;
    using System.Collections;
    using System.Collections.Generic;
    using System.Linq;
    using System.Threading;
    using System.Threading.Tasks;
    using System.Windows.Input;

    /// <summary>
    /// View model for the pointer scan results.
    /// </summary>
    public class PointerScanResultsViewModel : ToolViewModel
    {
        /// <summary>
        /// The number of elements to display on each page.
        /// </summary>
        private const Int32 PageSize = 64;

        /// <summary>
        /// Singleton instance of the <see cref="PointerScanResultsViewModel" /> class.
        /// </summary>
        private static Lazy<PointerScanResultsViewModel> pointerScanResultsViewModelInstance = new Lazy<PointerScanResultsViewModel>(
                () => { return new PointerScanResultsViewModel(); },
                LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// The result display type.
        /// </summary>
        private Type activeType;

        /// <summary>
        /// The current page of scan results.
        /// </summary>
        private UInt64 currentPage;

        /// <summary>
        /// The total number of addresses.
        /// </summary>
        private UInt64 addressCount;

        /// <summary>
        /// The addresses on the current page.
        /// </summary>
        private FullyObservableCollection<PointerItem> pointers;

        /// <summary>
        /// The list of discovered pointers.
        /// </summary>
        private PointerBag discoveredPointers;

        /// <summary>
        /// The pointer read interval in milliseconds
        /// </summary>
        private const Int32 PointerReadIntervalMs = 1600;

        /// <summary>
        /// The selected scan results.
        /// </summary>
        private IEnumerable<PointerItem> selectedScanResults;

        /// <summary>
        /// Prevents a default instance of the <see cref="PointerScanResultsViewModel" /> class from being created.
        /// </summary>
        private PointerScanResultsViewModel() : base("Pointer Scan Results")
        {
            this.ExtractPointerCommand = new RelayCommand<Int32>((levelIndex) => this.ExtractPointer(levelIndex), (levelIndex) => true);
            this.SelectScanResultsCommand = new RelayCommand<Object>((selectedItems) => this.SelectScanResults(selectedItems), (selectedItems) => true);

            this.ChangeTypeCommand = new RelayCommand<ScannableType>((type) => Task.Run(() => this.ChangeType(type)), (type) => true);
            this.NewPointerScanCommand = new RelayCommand(() => Task.Run(() => this.DiscoveredPointers = null), () => true);

            this.ActiveType = ScannableType.Int32;
            this.pointers = new FullyObservableCollection<PointerItem>();

            DockingViewModel.GetInstance().RegisterViewModel(this);
        }

        /// <summary>
        /// Gets the command to extract a pointer.
        /// </summary>
        public ICommand ExtractPointerCommand { get; private set; }

        /// <summary>
        /// Gets the command to select scan results.
        /// </summary>
        public ICommand SelectScanResultsCommand { get; private set; }

        /// <summary>
        /// Gets the command to add a scan result to the project explorer.
        /// </summary>
        public ICommand AddScanResultCommand { get; private set; }

        /// <summary>
        /// Gets the command to add all selected scan results to the project explorer.
        /// </summary>
        public ICommand AddScanResultsCommand { get; private set; }

        /// <summary>
        /// Gets the command to change the active data type.
        /// </summary>
        public ICommand ChangeTypeCommand { get; private set; }

        /// <summary>
        /// Gets a command to clear the pointer scan results.
        /// </summary>
        public ICommand NewPointerScanCommand { get; private set; }

        /// <summary>
        /// Gets the command to go to the first page.
        /// </summary>
        public ICommand FirstPageCommand { get; private set; }

        /// <summary>
        /// Gets the command to go to the last page.
        /// </summary>
        public ICommand LastPageCommand { get; private set; }

        /// <summary>
        /// Gets the command to go to the previous page.
        /// </summary>
        public ICommand PreviousPageCommand { get; private set; }

        /// <summary>
        /// Gets the command to go to the next page.
        /// </summary>
        public ICommand NextPageCommand { get; private set; }

        /// <summary>
        /// Gets the command to select a target process.
        /// </summary>
        public ICommand AddAddressCommand { get; private set; }

        /// <summary>
        /// Gets or sets the selected scan results.
        /// </summary>
        public IEnumerable<PointerItem> SelectedScanResults
        {
            get
            {
                return this.selectedScanResults;
            }

            set
            {
                this.selectedScanResults = value;
                this.RaisePropertyChanged(nameof(this.SelectedScanResults));
            }
        }

        /// <summary>
        /// Gets or sets the active scan results data type.
        /// </summary>
        public ScannableType ActiveType
        {
            get
            {
                return this.activeType;
            }

            set
            {
                this.activeType = value;

                this.RaisePropertyChanged(nameof(this.ActiveType));
                this.RaisePropertyChanged(nameof(this.ActiveTypeName));
            }
        }

        /// <summary>
        /// Gets the name associated with the active data type.
        /// </summary>
        public String ActiveTypeName
        {
            get
            {
                return Conversions.DataTypeToName(this.ActiveType);
            }
        }

        public IEnumerable<Level> DiscoveredLevels { get; set; }

        /// <summary>
        /// Gets or sets the list of discovered pointers.
        /// </summary>
        public PointerBag DiscoveredPointers
        {
            get
            {
                return this.discoveredPointers;
            }

            set
            {
                this.discoveredPointers = value;
                this.DiscoveredLevels = this.DiscoveredPointers?.Levels;

                this.RaisePropertyChanged(nameof(this.DiscoveredPointers));
                this.RaisePropertyChanged(nameof(this.DiscoveredLevels));
            }
        }

        /// <summary>
        /// Gets a singleton instance of the <see cref="PointerScanResultsViewModel"/> class.
        /// </summary>
        /// <returns>A singleton instance of the class.</returns>
        public static PointerScanResultsViewModel GetInstance()
        {
            return PointerScanResultsViewModel.pointerScanResultsViewModelInstance.Value;
        }

        /// <summary>
        /// Selects the given scan results.
        /// </summary>
        /// <param name="selectedItems">The scan results to select.</param>
        private void SelectScanResults(Object selectedItems)
        {
            this.SelectedScanResults = (selectedItems as IList)?.Cast<PointerItem>();
        }

        private void ExtractPointer(Int32 levelIndex)
        {
            Pointer pointer = this.DiscoveredPointers.GetRandomPointer(SessionManager.Session.OpenedProcess, levelIndex);

            if (pointer != null)
            {
                PointerItem pointerItem = new PointerItem(SessionManager.Session, pointer.ModuleOffset, this.ActiveType, "New Pointer", pointer.ModuleName, pointer.Offsets);
                ProjectExplorerViewModel.GetInstance().AddProjectItems(pointerItem);
            }
        }

        /// <summary>
        /// Changes the active scan pointer results type.
        /// </summary>
        /// <param name="newType">The new pointer scan results type.</param>
        private void ChangeType(ScannableType newType)
        {
            this.ActiveType = newType;
        }
    }
    //// End class
}
//// End namespace