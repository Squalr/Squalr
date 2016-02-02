﻿using Binarysharp.MemoryManagement;
using Binarysharp.MemoryManagement.Memory;
using Binarysharp.MemoryManagement.Modules;
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Anathema
{
    /*
    Trace-Retrace Algorithm:
    0) Potential pre-processing -- no idea how many valid pointers exist in a process, but we may be able to:
        - Store all pointers to use
        - Store all regions that contain a pointer

    1) Start with a base address. Convert this to a range that spans 1024 in each direction, add this to the target list
    2) REPEAT FOR N LEVELS:
        - Search for all pointer values that fall in in the target list
        - Convert these pointers to spanning regions, and add them to the target list, clearing the old list

    3) Retrace pointers. We will not trace pointers with invalid bases. Loop from last level to first level:
        - Compare pointer to all pointers in the previous level. Store offsets from current level to all pointers in previous level.
    */

    class PointerScanner : IPointerScannerModel, IProcessObserver
    {
        private MemorySharp MemoryEditor;
        private Snapshot<Null> Snapshot;

        // As far as I can tell, no valid pointers will end up being less than 0x10000 (UInt16.MaxValue). Huge gains by filtering these.
        private const UInt64 InvalidPointerMin = unchecked((UInt64)Int64.MaxValue); // !TODO RemotePtr.MaxValue
        private const UInt64 InvalidPointerMax = UInt16.MaxValue;


        private ConcurrentDictionary<UInt64, UInt64> PointerPool;
        private List<ConcurrentDictionary<UInt64, UInt64>> ConnectedPointers;
        private Snapshot<Null> AcceptedBases;

        private List<Tuple<UInt64, Stack<Int32>>> AcceptedPointers;

        // User parameters
        private UInt64 TargetAddress;
        private Int32 MaxPointerLevel;
        private UInt64 MaxPointerOffset;

        private enum ScanModeEnum
        {
            ReadValues,
            Scan,
            Rescan
        }

        private ScanModeEnum ScanMode;

        public PointerScanner()
        {
            PointerPool = new ConcurrentDictionary<UInt64, UInt64>();
            ConnectedPointers = new List<ConcurrentDictionary<UInt64, UInt64>>();
            ScanMode = ScanModeEnum.ReadValues;

            InitializeProcessObserver();

            Begin();
        }

        public void InitializeProcessObserver()
        {
            ProcessSelector.GetInstance().Subscribe(this);
        }

        public void UpdateMemoryEditor(MemorySharp MemoryEditor)
        {
            this.MemoryEditor = MemoryEditor;
        }

        public override void SetTargetAddress(UInt64 Address)
        {
            TargetAddress = Address;
        }

        public override void SetMaxPointerLevel(Int32 MaxPointerLevel)
        {
            this.MaxPointerLevel = MaxPointerLevel;
        }

        public override void SetMaxPointerOffset(UInt64 MaxPointerOffset)
        {
            this.MaxPointerOffset = MaxPointerOffset;
        }

        private SnapshotRegion AddressToRegion(UInt64 Address)
        {
            return new SnapshotRegion<Null>(new RemoteRegion(null, unchecked((IntPtr)(Address - MaxPointerOffset)), unchecked((Int32)MaxPointerOffset * 2)));
        }

        private void UpdateDisplay()
        {
            PointerScannerEventArgs Args = new PointerScannerEventArgs();
            Args.ItemCount = AcceptedPointers.Count;
            Args.MaxPointerLevel = MaxPointerLevel;
            OnEventScanFinished(Args);
        }

        public override String GetValueAtIndex(Int32 Index)
        {
            return Index.ToString();
        }

        public override String GetBaseAddress(Int32 Index)
        {
            return Conversions.ToAddress(AcceptedPointers[Index].Item1.ToString());
        }

        public override String[] GetOffsets(Int32 Index)
        {
            List<String> Offsets = new List<String>();
            AcceptedPointers[Index].Item2.Reverse().ToList().ForEach(x => Offsets.Add((x < 0 ? "-" : "") + Math.Abs(x).ToString("X")));
            return Offsets.ToArray();
        }

        public override void Begin()
        {
            base.Begin();
        }

        public override void BeginPointerScan()
        {
            ScanMode = ScanModeEnum.Scan;
        }

        public override void BeginPointerRescan()
        {
            ScanMode = ScanModeEnum.Rescan;
        }

        protected override void Update()
        {
            base.Update();

            // Scan mode determines the action to make, such that the action always happens on this task thread
            switch (ScanMode)
            {
                case ScanModeEnum.ReadValues:
                    OnEventReadValues(new PointerScannerEventArgs());
                    break;
                case ScanModeEnum.Scan:
                    PointerScan();
                    ScanMode = ScanModeEnum.ReadValues;
                    break;
                case ScanModeEnum.Rescan:
                    PointerRescan();
                    ScanMode = ScanModeEnum.ReadValues;
                    break;
            }
        }

        public override void End()
        {
            base.End();
        }

        private void PointerScan()
        {
            // Clear current pointer pool
            PointerPool.Clear();

            // Collect memory regions
            Snapshot = new Snapshot<Null>(SnapshotManager.GetInstance().SnapshotAllRegions(true));

            // Set to type of a pointer
            Snapshot.SetElementType(typeof(UInt64));

            Parallel.ForEach(Snapshot.Cast<Object>(), (RegionObject) =>
            {
                SnapshotRegion Region = (SnapshotRegion)RegionObject;

                // Read the memory of this region
                try { Region.ReadAllSnapshotMemory(Snapshot.GetMemoryEditor(), true); }
                catch (ScanFailedException) { return; }

                if (!Region.HasValues())
                    return;

                foreach (SnapshotElement Element in Region)
                {
                    if (Element.LessThanValue(InvalidPointerMax))
                        continue;

                    if (Element.GreaterThanValue(InvalidPointerMin))
                        continue;

                    if (unchecked((UInt64)Element.BaseAddress) % 4 != 0)
                        continue;

                    if (Element.GetValue() % 4 != 0)
                        continue;

                    if (Snapshot.ContainsAddress(Element.GetValue()))
                        PointerPool[unchecked((UInt64)Element.BaseAddress)] = unchecked((UInt64)Element.GetValue());
                }

                // Clear the saved values, we do not need them now
                Region.SetCurrentValues(null);
            });

            TracePointers();
            BuildPointers();

            UpdateDisplay();
        }

        private void PointerRescan()
        {

        }

        private void SetAcceptedBases()
        {
            if (MemoryEditor == null)
                return;

            //List<RemoteModule> Modules = MemoryEditor.Modules.RemoteModules.ToList();
            List<RemoteModule> Modules = new List<RemoteModule>();
            Modules.Add(MemoryEditor.Modules.MainModule);

            List<SnapshotRegion> AcceptedBaseRegions = new List<SnapshotRegion>();

            // Gather regions from every module as valid base addresses
            Modules.ForEach(x => AcceptedBaseRegions.Add(new SnapshotRegion<Null>(new RemoteRegion(MemoryEditor, x.BaseAddress, x.Size))));

            // Convert regions into a snapshot
            AcceptedBases = new Snapshot<Null>(AcceptedBaseRegions.ToArray());
        }

        private void TracePointers()
        {
            List<SnapshotRegion> PreviousLevelRegions = new List<SnapshotRegion>();
            PreviousLevelRegions.Add(AddressToRegion(TargetAddress));

            ConnectedPointers.Clear();
            SetAcceptedBases();

            // Add the address we are looking for as the base
            ConnectedPointers.Add(new ConcurrentDictionary<UInt64, UInt64>());
            ConnectedPointers.Last()[TargetAddress] = 0;

            for (Int32 Level = 1; Level <= MaxPointerLevel; Level++)
            {
                // Create snapshot from previous level regions to leverage the merging and sorting capabilities of a snapshot
                Snapshot PreviousLevel = new Snapshot<Null>(PreviousLevelRegions.ToArray());
                ConcurrentDictionary<UInt64, UInt64> LevelPointers = new ConcurrentDictionary<UInt64, UInt64>();

                Parallel.ForEach(PointerPool, (Pointer) =>
                {
                    // Ensure if this is a max level pointer that it is from an acceptable base address (ie static)
                    if (Level == MaxPointerLevel && !AcceptedBases.ContainsAddress(Pointer.Key))
                        return;

                    // Accept this pointer if it is points to the previous level snapshot
                    if (PreviousLevel.ContainsAddress(Pointer.Value))
                        LevelPointers[Pointer.Key] = Pointer.Value;
                });

                // Add the pointers for this level to the global accepted list
                ConnectedPointers.Add(LevelPointers);

                PreviousLevelRegions.Clear();

                // Construct new target region list from this level of pointers
                foreach (KeyValuePair<UInt64, UInt64> Pointer in LevelPointers)
                    PreviousLevelRegions.Add(AddressToRegion(Pointer.Key));
            }

            PointerPool.Clear();
        }

        private void BuildPointers()
        {
            ConcurrentBag<Tuple<UInt64, Stack<Int32>>> DiscoveredPointers = new ConcurrentBag<Tuple<UInt64, Stack<Int32>>>();

            Parallel.ForEach(ConnectedPointers[MaxPointerLevel], (Base) =>
            {
                BuildPointers(DiscoveredPointers, MaxPointerLevel, Base.Key, Base.Value, new Stack<Int32>());
            });

            AcceptedPointers = DiscoveredPointers.ToList();
        }

        private void BuildPointers(ConcurrentBag<Tuple<UInt64, Stack<Int32>>> Pointers, Int32 Level, UInt64 Base, UInt64 PointerDestination, Stack<Int32> Offsets)
        {
            if (Level == 0)
            {
                Pointers.Add(new Tuple<UInt64, Stack<Int32>>(Base, Offsets));
                return;
            }

            foreach (KeyValuePair<UInt64, UInt64> Target in ConnectedPointers[Level - 1])
            {
                if (PointerDestination < unchecked(Target.Key - MaxPointerOffset))
                    continue;

                if (PointerDestination > unchecked(Target.Key + MaxPointerOffset))
                    continue;

                // Valid pointer, clone our current offset stack
                Stack<Int32> NewOffsets = new Stack<Int32>(Offsets.Reverse());

                // Calculate the offset for this level
                NewOffsets.Push(unchecked((Int32)((Int64)Target.Key - (Int64)PointerDestination)));

                // Recurse
                BuildPointers(Pointers, Level - 1, Base, Target.Value, NewOffsets);
            }
        }

    } // End class

} // End namespace