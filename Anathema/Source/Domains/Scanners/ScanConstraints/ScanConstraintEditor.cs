﻿using System;

namespace Anathema.Scanners.ScanConstraints
{
    class ScanConstraintEditor : IScanConstraintEditorModel
    {
        // User controlled variables
        private ScanConstraintManager ScanConstraints;

        public event ScanConstraintEditorEventHandler EventUpdateDisplay;

        public ScanConstraintEditor()
        {
            ScanConstraints = new ScanConstraintManager();
        }

        private void UpdateDisplay()
        {
            ScanConstraintEditorEventArgs FilterManualScanEventArgs = new ScanConstraintEditorEventArgs();
            FilterManualScanEventArgs.ScanConstraints = ScanConstraints;
            EventUpdateDisplay(this, FilterManualScanEventArgs);
        }

        public ScanConstraintManager GetScanConstraintManager()
        {
            return ScanConstraints;
        }

        public Type GetElementType()
        {
            return ScanConstraints.GetElementType();
        }

        public void SetElementType(Type ElementType)
        {
            ScanConstraints.SetElementType(ElementType);
            UpdateDisplay();
        }

        public ScanConstraint GetConstraintAt(Int32 Index)
        {
            return ScanConstraints[Index];
        }

        public void AddConstraint(ConstraintsEnum ValueConstraint, dynamic Value)
        {
            ScanConstraints.AddConstraint(new ScanConstraint(ValueConstraint, Value));
            UpdateDisplay();
        }

        public void UpdateConstraint(Int32 Index, dynamic Value)
        {
            ScanConstraints[Index].Value = Value;
            UpdateDisplay();
        }

        public void RemoveConstraints(Int32[] ConstraintIndicies)
        {
            ScanConstraints.RemoveConstraints(ConstraintIndicies);
            UpdateDisplay();
        }

        public void ClearConstraints()
        {
            ScanConstraints.ClearConstraints();
            UpdateDisplay();
        }

    } // End class

} // End namespace