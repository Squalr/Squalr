namespace Squalr.Engine.Input.HotKeys
{
    using System;
    using System.ComponentModel;
    using System.Runtime.Serialization;
    using System.Threading.Tasks;

    /// <summary>
    /// An interface defining a hotkey, which is activated by a given set of input.
    /// </summary>
    [KnownType(typeof(ControllerHotkey))]
    [KnownType(typeof(KeyboardHotkey))]
    [KnownType(typeof(MouseHotkey))]
    [DataContract]
    public abstract class Hotkey : INotifyPropertyChanged, IDisposable
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="Hotkey" /> class.
        /// </summary>
        /// <param name="callBackFunction">The callback function for this hotkey.</param>
        public Hotkey(Action callBackFunction = null)
        {
            this.CallBackFunction = callBackFunction;
        }

        /// <summary>
        /// An event that is raised when a property of this object changes.
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;

        /// <summary>
        /// Gets or sets the callback function of this hotkey.
        /// </summary>
        protected Action CallBackFunction { get; set; }

        /// <summary>
        /// Gets or sets the time since this hotkey was last activated.
        /// </summary>
        protected DateTime LastActivated { get; set; }

        /// <summary>
        /// Gets or sets the minimum delay between hotkey activations.
        /// </summary>
        protected Int32 ActivationDelay { get; set; }

        /// <summary>
        /// Disposes of this hotkey object.
        /// </summary>
        public abstract void Dispose();

        /// <summary>
        /// Sets the callback function to be triggered by this hotkey.
        /// </summary>
        /// <param name="callBackFunction">The new callback function for this hotkey.</param>
        public void SetCallBackFunction(Action callBackFunction)
        {
            this.CallBackFunction = callBackFunction;
        }

        /// <summary>
        /// Sets the minimum delay between hotkey activations.
        /// </summary>
        /// <param name="activationDelay">The minimum delay between hotkey activations.</param>
        public void SetActivationDelay(Int32 activationDelay)
        {
            this.ActivationDelay = activationDelay;
        }

        /// <summary>
        /// Determines if the current set of activation hotkeys are empty.
        /// </summary>
        /// <returns>True if there are hotkeys, otherwise false.</returns>
        public abstract Boolean HasHotkey();

        /// <summary>
        /// Clones the hotkey.
        /// </summary>
        /// <param name="copyCallBackFunction">A value indicating whether to copy the callback function from this hotkey to the clone.</param>
        /// <returns>A clone of the hotkey.</returns>
        public abstract Hotkey Clone(Boolean copyCallBackFunction = false);

        /// <summary>
        /// Copies the hotkey to another hotkey. A new hotkey is created if null is provided.
        /// </summary>
        /// <param name="hotkey">The hotkey to which the properties of this hotkey are copied.</param>
        /// <param name="copyCallBackFunction">A value indicating whether to copy the callback function from this hotkey to the given one.</param>
        /// <returns>A copy of the hotkey.</returns>
        public abstract Hotkey CopyTo(Hotkey hotkey, Boolean copyCallBackFunction = false);

        /// <summary>
        /// Activates this hotkey, triggering the callback function.
        /// </summary>
        protected virtual void Activate()
        {
            Task.Run(() => this.CallBackFunction?.Invoke());
        }

        /// <summary>
        /// Determines if this hotkey is able to be triggered.
        /// </summary>
        /// <returns>True if able to be triggered, otherwise false.</returns>
        protected abstract Boolean IsReady();
    }
    //// End interface
}
//// End namespace