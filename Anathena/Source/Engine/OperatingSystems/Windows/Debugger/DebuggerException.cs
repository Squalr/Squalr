﻿using System;

namespace Anathena.Source.Engine.OperatingSystems.Windows.Debugger
{
    class DebuggerException : Exception
    {
        public DebuggerException(String Message) : base(Message) { }

    } // End class

} // End namespace