﻿namespace Ana.View.Controls
{
    using Source.Utils.Validation;
    using System;
    using System.Windows;
    using System.Windows.Controls;

    internal class HexDecBox : TextBox
    {
        public static readonly DependencyProperty ValueProperty =
            DependencyProperty.Register(nameof(HexDecBox.Value), typeof(String), typeof(HexDecBox));

        private Boolean isHex;

        private UInt64 value;

        private String lastValidValue;

        public HexDecBox()
        {
            this.TextChanged += HexDecBoxTextChanged;
        }

        public UInt64 Value
        {
            get
            {
                return this.value;
            }

            set
            {
                this.value = value;
            }
        }

        public Boolean IsHex
        {
            get
            {
                return this.isHex;
            }

            set
            {
                this.isHex = value;
                // this.RaisePropertyChanged(nameof(this.IsDec));
                // this.RaisePropertyChanged(nameof(this.IsHex));
            }
        }

        public Boolean IsDec
        {
            get
            {
                return !this.isHex;
            }

            set
            {
                this.isHex = !value;
                //    this.RaisePropertyChanged(nameof(this.IsDec));
                // this.RaisePropertyChanged(nameof(this.IsHex));
            }
        }

        private void HexDecBoxTextChanged(Object sender, TextChangedEventArgs e)
        {
            TextBox textbox = e.OriginalSource as TextBox;
            String rawValue = textbox.Text;

            if (this.IsDec && CheckSyntax.IsUInt64(rawValue))
            {
                this.Value = Conversions.ParseDecStringAsValue(typeof(UInt64), rawValue);
                this.lastValidValue = rawValue;
            }
            else if (this.IsHex && CheckSyntax.CanParseHex(typeof(UInt64), rawValue))
            {
                this.Value = Conversions.ParseHexStringAsValue(typeof(UInt64), rawValue);
                this.lastValidValue = rawValue;
            }
            else
            {
                textbox.Text = String.Empty;
                textbox.AppendText(this.lastValidValue);
            }
        }
    }
    //// End class
}
//// End namespace