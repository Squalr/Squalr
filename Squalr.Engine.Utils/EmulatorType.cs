namespace Squalr.Engine.Common
{
    using System.Runtime.Serialization;

    /// <summary>
    /// An enum representing an emulator target.
    /// </summary>
    [DataContract]
    public enum EmulatorType
    {
        /// <summary>
        /// A value used to request that Squalr automatically detect if the target process is running a console emulator.
        /// </summary>
        AutoDetect,

        /// <summary>
        /// A value indicating that a process is normal, and not a console emulator.
        /// </summary>
        None,

        /// <summary>
        /// A value indicating that a process is the Dolphin Game Cube emulator.
        /// </summary>
        Dolphin,
    }
    //// End enum
}
//// End namespace