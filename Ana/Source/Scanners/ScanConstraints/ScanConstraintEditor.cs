﻿using System;
using System.Collections.Generic;
using System.Reflection;

namespace Ana.Source.Scanners.ScanConstraints
{
    class ScanConstraintEditor : IScannerModel
    {
        // User controlled variables
        private ScanConstraintManager ScanConstraints;

        public ScanConstraintEditor()
        {
            ScanConstraints = new ScanConstraintManager();
        }

        public void OnGUIOpen() { }

        private void UpdateDisplay()
        {

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

        [Obfuscation(Exclude = true)]
        public void AddConstraint(ConstraintsEnum ValueConstraint, dynamic Value)
        {
            ScanConstraints.AddConstraint(new ScanConstraint(ValueConstraint, Value));
            UpdateDisplay();
        }

        [Obfuscation(Exclude = true)]
        public void UpdateConstraint(Int32 Index, dynamic Value)
        {
            ScanConstraints[Index].Value = Value;
            UpdateDisplay();
        }

        public void RemoveConstraints(IEnumerable<Int32> ConstraintIndicies)
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