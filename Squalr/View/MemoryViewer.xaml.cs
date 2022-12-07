namespace Squalr.View
{
    using System;
    using System.Windows;
    using System.Windows.Controls;

    /// <summary>
    /// A memory viewer user control.
    /// </summary>
    public partial class MemoryViewer : UserControl
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="MemoryViewer" /> class.
        /// </summary>
        public MemoryViewer()
        {
            this.InitializeComponent();
        }

        private void HexEditorSizeChanged(Object sender, SizeChangedEventArgs e)
        {
            const Double AddressColumnSize = 64.0; // True for 8 byte addresses
            const Double SeperationBufferSize = 30.0;
            const Double HexColumnSize = 168.0;
            const Double AsciiColumnSize = 86.0;

            Double width = e.NewSize.Width - AddressColumnSize - SeperationBufferSize;
            Double sections = width / (HexColumnSize + AsciiColumnSize);
            Int32 sectionsRounded = Math.Clamp((Int32)sections, 1, 8);

            this.hexEditor.BytePerLine = sectionsRounded * 8;
        }
    }
    //// End class
}
//// End namespace