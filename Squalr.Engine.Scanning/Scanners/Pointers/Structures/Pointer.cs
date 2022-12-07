namespace Squalr.Engine.Scanning.Scanners.Pointers.Structures
{
    using System;

    public class Pointer
    {
        public Pointer(String moduleName, UInt64 moduleOffset, PointerSize pointerSize, Int32[] offsets = null)
        {
            this.ModuleName = moduleName;
            this.ModuleOffset = moduleOffset;
            this.PointerSize = pointerSize;
            this.Offsets = offsets;
        }

        public String ModuleName { get; private set; }

        public UInt64 ModuleOffset { get; private set; }

        public Int32[] Offsets { get; private set; }

        public PointerSize PointerSize { get; private set; }
    }
    //// End class
}
//// End namespace