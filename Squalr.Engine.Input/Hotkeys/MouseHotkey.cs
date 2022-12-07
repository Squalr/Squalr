namespace Squalr.Engine.Input.HotKeys
{
    using System;
    using System.Runtime.Serialization;

    /// <summary>
    /// A mouse hotkey, which is activated by a given set of input.
    /// </summary>
    [DataContract]
    public class MouseHotkey : Hotkey
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="MouseHotkey" /> class.
        /// </summary>
        /// <param name="callBackFunction">The callback function for this hotkey.</param>
        public MouseHotkey(Action callBackFunction = null) : base(callBackFunction)
        {
        }

        /// <summary>
        /// Disposes of this hotkey object.
        /// </summary>
        public override void Dispose()
        {
        }

        /// <summary>
        /// Determines if the current set of activation hotkeys are empty.
        /// </summary>
        /// <returns>True if there are hotkeys, otherwise false.</returns>
        public override Boolean HasHotkey()
        {
            throw new NotImplementedException();
        }

        /// <summary>
        /// Clones the hotkey.
        /// </summary>
        /// <param name="copyCallBackFunction">A value indicating whether to copy the callback function from this hotkey to the clone.</param>
        /// <returns>A clone of the hotkey.</returns>
        public override Hotkey Clone(Boolean copyCallBackFunction = false)
        {
            throw new NotImplementedException();
        }

        /// <summary>
        /// Copies the hotkey to another hotkey. A new hotkey is created if null is provided.
        /// </summary>
        /// <param name="hotkey">The hotkey to which the properties of this hotkey are copied.</param>
        /// <param name="copyCallBackFunction">A value indicating whether to copy the callback function from this hotkey to the given one.</param>
        /// <returns>A copy of the hotkey.</returns>
        public override Hotkey CopyTo(Hotkey hotkey, Boolean copyCallBackFunction = false)
        {
            throw new NotImplementedException();
        }

        /// <summary>
        /// Gets the string representation of the hotkey inputs.
        /// </summary>
        /// <returns>The string representation of hotkey inputs.</returns>
        public override String ToString()
        {
            return base.ToString();
        }

        /// <summary>
        /// Determines if this hotkey is able to be triggered.
        /// </summary>
        /// <returns>True if able to be triggered, otherwise false.</returns>
        protected override Boolean IsReady()
        {
            throw new NotImplementedException();
        }

        /// <summary>
        /// Activates this hotkey, triggering the callback function.
        /// </summary>
        protected override void Activate()
        {
            throw new NotImplementedException();
        }
    }
    //// End class
}
//// End namespace