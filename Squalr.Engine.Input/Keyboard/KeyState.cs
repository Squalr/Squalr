namespace Squalr.Engine.Input.Keyboard
{
    using SharpDX.DirectInput;
    using System.Collections.Generic;

    /// <summary>
    /// An object that tracks the state of pressed, released, down, and held keys.
    /// </summary>
    public class KeyStates
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="KeyStates" /> class.
        /// </summary>
        /// <param name="pressedKeys">The set of currently pressed keys.</param>
        /// <param name="releasedKeys">The set of currently released keys.</param>
        /// <param name="downKeys">The set of currently down keys.</param>
        /// <param name="heldKeys">The set of currently held keys.</param>
        public KeyStates(HashSet<Key> pressedKeys, HashSet<Key> releasedKeys, HashSet<Key> downKeys, HashSet<Key> heldKeys)
        {
            this.PressedKeys = pressedKeys;
            this.ReleasedKeys = releasedKeys;
            this.DownKeys = downKeys;
            this.HeldKeys = heldKeys;
        }

        /// <summary>
        /// Gets the set of currently pressed keys.
        /// </summary>
        public HashSet<Key> PressedKeys { get; private set; }

        /// <summary>
        /// Gets the set of currently released keys.
        /// </summary>
        public HashSet<Key> ReleasedKeys { get; private set; }

        /// <summary>
        /// Gets the set of currently down keys.
        /// </summary>
        public HashSet<Key> DownKeys { get; private set; }

        /// <summary>
        /// Gets the set of currently held keys.
        /// </summary>
        public HashSet<Key> HeldKeys { get; private set; }
    }
    //// End class
}
//// End namespace