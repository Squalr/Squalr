namespace Squalr.Engine.Input.HotKeys
{
    using Keyboard;
    using SharpDX.DirectInput;
    using Squalr.Engine.Common.Extensions;
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using System.Runtime.Serialization;

    /// <summary>
    /// A keyboard hotkey, which is activated by a given set of input.
    /// </summary>
    [DataContract]
    public class KeyboardHotkey : Hotkey, IObserver<KeyStates>
    {
        /// <summary>
        /// The default delay in miliseconds between hotkey activations.
        /// </summary>
        private const Int32 DefaultActivationDelay = 150;

        /// <summary>
        /// Initializes a new instance of the <see cref="KeyboardHotkey" /> class.
        /// </summary>
        /// <param name="callBackFunction">The callback function for this hotkey.</param>
        /// <param name="activationKeys">Initial activation keys.</param>
        public KeyboardHotkey(Action callBackFunction = null, params Key[] activationKeys) : base(callBackFunction)
        {
            this.ActivationKeys = new HashSet<Key>(activationKeys);
            this.LastActivated = DateTime.MinValue;
            this.ActivationDelay = KeyboardHotkey.DefaultActivationDelay;
            this.ActivationKeysLock = new Object();
            this.KeyboardCapture = InputManager.GetInstance().GetKeyboardCapture().WeakSubscribe(this);
        }

        /// <summary>
        /// Gets or sets the set of inputs corresponding to this hotkey.
        /// </summary>
        [DataMember]
        private HashSet<Key> ActivationKeys { get; set; }

        /// <summary>
        /// Gets or sets a lock object to provide thread-safe access to the ActivationKeys object.
        /// </summary>
        private Object ActivationKeysLock { get; set; }

        /// <summary>
        /// Gets or sets an object to subscribe to keyboard capture events.
        /// </summary>
        private IDisposable KeyboardCapture { get; set; }

        /// <summary>
        /// Invoked when this object is deserialized.
        /// </summary>
        /// <param name="streamingContext">Streaming context.</param>
        [OnDeserialized]
        public void OnDeserialized(StreamingContext streamingContext)
        {
            this.LastActivated = DateTime.MinValue;
            this.ActivationDelay = KeyboardHotkey.DefaultActivationDelay;
            this.ActivationKeysLock = new object();

            this.KeyboardCapture = InputManager.GetInstance().GetKeyboardCapture().WeakSubscribe(this);
        }

        /// <summary>
        /// Disposes of this hotkey object.
        /// </summary>
        public override void Dispose()
        {
            InputManager.GetInstance().GetKeyboardCapture().Unsubscribe(this.KeyboardCapture);
        }

        /// <summary>
        /// Subscription event for when a keyboard event is fired.
        /// </summary>
        /// <param name="value">The current keyboard key states.</param>
        public void OnNext(KeyStates value)
        {
            lock (this.ActivationKeysLock)
            {
                if (this.ActivationKeys.IsNullOrEmpty())
                {
                    return;
                }

                // Check if one of the keys in the hotkey was released early
                if (!this.IsReady())
                {
                    foreach (Key key in value.DownKeys)
                    {
                        if (this.ActivationKeys.Any(x => key == x))
                        {
                            // Reset the activation timer so that this hotkey can be triggered again immediately
                            this.LastActivated = DateTime.MinValue;
                        }
                    }
                }

                if (this.IsReady() && this.ActivationKeys.All(x => value.PressedKeys.Contains(x)))
                {
                    this.Activate();
                }
            }
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
        /// Gets the set of activation keys needed to activate this hotkey.
        /// </summary>
        /// <returns>The set of activation keys needed to activate this hotkey.</returns>
        public IEnumerable<Key> GetActivationKeys()
        {
            lock (this.ActivationKeysLock)
            {
                return this.ActivationKeys.ToArray();
            }
        }

        /// <summary>
        /// Adds a key needed to activate this hotkey.
        /// </summary>
        /// <param name="hotkey">The key needed to activate this hotkey.</param>
        public void AddKey(Key hotkey)
        {
            lock (this.ActivationKeysLock)
            {
                this.ActivationKeys.Add(hotkey);
            }
        }

        /// <summary>
        /// Clears the set of keys needed to activate this hotkey.
        /// </summary>
        public void ClearHotkey()
        {
            lock (this.ActivationKeysLock)
            {
                this.ActivationKeys.Clear();
            }
        }

        /// <summary>
        /// Determines if the current set of activation hotkeys are empty.
        /// </summary>
        /// <returns>True if there are hotkeys, otherwise false.</returns>
        public override Boolean HasHotkey()
        {
            lock (this.ActivationKeysLock)
            {
                return this.ActivationKeys == null ? false : this.ActivationKeys.Count > 0;
            }
        }

        /// <summary>
        /// Clones the hotkey.
        /// </summary>
        /// <param name="copyCallBackFunction">A value indicating whether to copy the callback function from this hotkey to the clone.</param>
        /// <returns>A clone of the hotkey.</returns>
        public override Hotkey Clone(Boolean copyCallBackFunction = false)
        {
            lock (this.ActivationKeysLock)
            {
                KeyboardHotkey hotkey = new KeyboardHotkey(copyCallBackFunction ? this.CallBackFunction : null);
                hotkey.ActivationKeys = new HashSet<Key>(this.ActivationKeys);
                return hotkey;
            }
        }

        /// <summary>
        /// Copies the hotkey to another hotkey. A new hotkey is created if null is provided.
        /// </summary>
        /// <param name="hotkey">The hotkey to which the properties of this hotkey are copied.</param>
        /// <param name="copyCallBackFunction">A value indicating whether to copy the callback function from this hotkey to the given one.</param>
        /// <returns>A copy of the hotkey.</returns>
        public override Hotkey CopyTo(Hotkey hotkey, Boolean copyCallBackFunction = false)
        {
            lock (this.ActivationKeysLock)
            {
                KeyboardHotkey keyboardHotkey = hotkey as KeyboardHotkey;

                if (keyboardHotkey == null)
                {
                    return this.Clone(copyCallBackFunction);
                }

                keyboardHotkey.ActivationKeys = new HashSet<Key>(this.ActivationKeys);

                if (copyCallBackFunction)
                {
                    keyboardHotkey.SetCallBackFunction(this.CallBackFunction);
                }

                return keyboardHotkey;
            }
        }

        /// <summary>
        /// Gets the string representation of the hotkey inputs.
        /// </summary>
        /// <returns>The string representation of hotkey inputs.</returns>
        public override String ToString()
        {
            lock (this.ActivationKeysLock)
            {
                String hotKeyString = String.Empty;

                if (this.ActivationKeys.IsNullOrEmpty())
                {
                    return hotKeyString;
                }

                foreach (Key key in this.ActivationKeys)
                {
                    hotKeyString += key.ToString() + "+";
                }

                return hotKeyString.TrimEnd('+');
            }
        }

        /// <summary>
        /// Activates this hotkey, triggering the callback function.
        /// </summary>
        protected override void Activate()
        {
            this.LastActivated = DateTime.Now;
            base.Activate();
        }

        /// <summary>
        /// Determines if this hotkey is able to be triggered.
        /// </summary>
        /// <returns>True if able to be triggered, otherwise false.</returns>
        protected override Boolean IsReady()
        {
            if (DateTime.Now - this.LastActivated > TimeSpan.FromMilliseconds(this.ActivationDelay))
            {
                return true;
            }

            return false;
        }
    }
    //// End class
}
//// End namespace