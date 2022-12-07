namespace Squalr.Source.Debugger
{
    using Squalr.Engine.Debuggers;
    using System;
    using System.ComponentModel;

    public class CodeTraceResult : INotifyPropertyChanged
    {
        private UInt64 address;

        private String instruction;

        private Int32 count;

        private CodeTraceInfo codeTraceInfo;

        public CodeTraceResult(CodeTraceInfo codeTraceInfo)
        {
            this.codeTraceInfo = codeTraceInfo;

            this.Address = codeTraceInfo.Instruction.Address;
            this.Instruction = codeTraceInfo.Instruction.Mnemonic;
            this.Count = 1;
        }

        /// <summary>
        /// An event that is raised when a property of this object changes.
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;

        public UInt64 Address
        {
            get
            {
                return this.address;
            }

            set
            {
                this.address = value;
                this.RaisePropertyChanged(nameof(this.Address));
            }
        }

        public String Instruction
        {
            get
            {
                return this.instruction;
            }

            set
            {
                this.instruction = value;
                this.RaisePropertyChanged(nameof(this.Instruction));
            }
        }

        public Int32 Count
        {
            get
            {
                return this.count;
            }

            set
            {
                this.count = value;
                this.RaisePropertyChanged(nameof(this.Count));
            }
        }

        /// <summary>
        /// Indicates that a given property in this project item has changed.
        /// </summary>
        /// <param name="propertyName">The name of the changed property.</param>
        protected void RaisePropertyChanged(String propertyName)
        {
            this.PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
    //// End interface
}
//// End namespace
