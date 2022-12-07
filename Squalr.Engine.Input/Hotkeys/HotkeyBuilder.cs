namespace Squalr.Engine.Input.HotKeys
{
    using System;

    /// <summary>
    /// An interface defining a hotkey, which is activated by a given set of input.
    /// </summary>
    public abstract class HotkeyBuilder
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="HotkeyBuilder" /> class.
        /// </summary>
        /// <param name="callBackFunction">The callback function for this hotkey.</param>
        public HotkeyBuilder(Action callBackFunction = null)
        {
            this.CallBackFunction = callBackFunction;
        }

        /// <summary>
        /// Gets or sets the callback function of this hotkey.
        /// </summary>
        protected Action CallBackFunction { get; set; }

        /// <summary>
        /// Creates a hotkey from this hotkey builder.
        /// </summary>
        /// <param name="targetHotkey">The hotkey to build.</param>
        /// <returns>The built hotkey.</returns>
        public abstract Hotkey Build(Hotkey targetHotkey);

        /// <summary>
        /// Event triggered when the hotkeys are updated for this keyboard hotkey.
        /// </summary>
        protected void OnHotkeysUpdated()
        {
            this.CallBackFunction?.Invoke();
        }
    }
    //// End interface
}
//// End namespace