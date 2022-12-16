namespace Squalr
{
    using System;
    
    /// <summary>
    /// A static class for interfacing with Squalr non-engine settings.
    /// </summary>
    public static class SqualrSettings
    {
        /// <summary>
        /// Gets or sets a value indicating whether Squalr should check for automatic updates.
        /// </summary>
        public static Boolean AutomaticUpdates
        {
            get
            {
                return Properties.Settings.Default.AutomaticUpdates;
            }

            set
            {
                Properties.Settings.Default.AutomaticUpdates = value;
            }
        }
    }
    //// End class
}
//// End namespace