namespace Squalr.Engine.Input.HotKeys
{
    using Keyboard;
    using SharpDX.DirectInput;
    using Squalr.Engine.Common.Extensions;
    using System;
    using System.Runtime.Serialization;

    /// <summary>
    /// A keyboard hotkey builder, which is used to construct a keyboard hotkey.
    /// </summary>
    [DataContract]
    public class KeyboardHotkeyBuilder : HotkeyBuilder, IObserver<KeyStates>
    {
        /// <summary>
        /// The default delay in miliseconds between hotkey activations.
        /// </summary>
        private const Int32 DefaultActivationDelay = 150;

        /// <summary>
        /// Initializes a new instance of the <see cref="KeyboardHotkeyBuilder" /> class.
        /// </summary>
        /// <param name="callBackFunction">The callback function for this hotkey.</param>
        /// <param name="keyboardHotkey">The keyboard hotkey to edit.</param>
        public KeyboardHotkeyBuilder(Action callBackFunction, KeyboardHotkey keyboardHotkey = null) : base(callBackFunction)
        {
            this.SetHotkey(keyboardHotkey);

            this.KeyboardCapture = InputManager.GetInstance().GetKeyboardCapture().WeakSubscribe(this);

            // TODO: this in a Dispose call?
            // InputManager.GetInstance().GetKeyboardCapture().Unsubscribe(this.KeyboardCapture);
        }

        /// <summary>
        /// Gets or sets the hotkey being constructed.
        /// </summary>
        private KeyboardHotkey KeyboardHotkey { get; set; }

        /// <summary>
        /// Gets or sets an object to subscribe to keyboard capture events.
        /// </summary>
        private IDisposable KeyboardCapture { get; set; }

        /// <summary>
        /// Subscription event for when a keyboard event is fired.
        /// </summary>
        /// <param name="value">The current keyboard key states.</param>
        public void OnNext(KeyStates value)
        {
            if (value.DownKeys.IsNullOrEmpty())
            {
                return;
            }

            foreach (Key key in value.DownKeys)
            {
                this.KeyboardHotkey.AddKey(key);
            }

            this.OnHotkeysUpdated();
        }

        /// <summary>
        /// Notifies the observer that the provider has experienced an error condition.
        /// </summary>
        /// <param name="error">An object that provides additional information about the error.</param>
        public void OnError(Exception error)
        {
        }

        /// <summary>
        /// Notifies the observer that the provider has finished sending push-based notifications.
        /// </summary>
        public void OnCompleted()
        {
        }

        /// <summary>
        /// Clears the hotkeys associated with this builder.
        /// </summary>
        public void ClearHotkeys()
        {
            this.KeyboardHotkey.ClearHotkey();
            this.OnHotkeysUpdated();
        }

        /// <summary>
        /// Sets the hotkey associated with this hotkey builder.
        /// </summary>
        /// <param name="keyboardHotkey">The hotkey associated with this hotkey builder.</param>
        public void SetHotkey(KeyboardHotkey keyboardHotkey)
        {
            this.KeyboardHotkey = (keyboardHotkey?.CopyTo(this.KeyboardHotkey) as KeyboardHotkey) ?? new KeyboardHotkey();
        }

        /// <summary>
        /// Creates a hotkey from this hotkey builder.
        /// </summary>
        /// <param name="targetHotkey">The hotkey to build.</param>
        /// <returns>The built hotkey.</returns>
        public override Hotkey Build(Hotkey targetHotkey)
        {
            return this.KeyboardHotkey?.CopyTo(targetHotkey);
        }

        /// <summary>
        /// Gets the string representation of the hotkey inputs.
        /// </summary>
        /// <returns>The string representation of hotkey inputs.</returns>
        public override String ToString()
        {
            return this.KeyboardHotkey.ToString();
        }
    }
    //// End class
}
//// End namespace