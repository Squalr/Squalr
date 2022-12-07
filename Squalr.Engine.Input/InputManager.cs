namespace Squalr.Engine.Input
{
    using Controller;
    using Keyboard;
    using Mouse;
    using System;
    using System.Threading;
    using System.Threading.Tasks;

    /// <summary>
    /// Manages all input devices and is responsible for updating them.
    /// </summary>
    public class InputManager : IInputManager
    {
        /// <summary>
        /// Singleton instance of the <see cref="WindowsAdapter"/> class
        /// </summary>
        private static readonly Lazy<InputManager> InputManagerInstance = new Lazy<InputManager>(
            () => { return new InputManager(); },
            LazyThreadSafetyMode.ExecutionAndPublication);

        /// <summary>
        /// The rate at which to collect input in ms. Currently 60 times per second.
        /// </summary>
        private static readonly TimeSpan InputCollectionInterval = TimeSpan.FromMilliseconds(1000.0 / 60.0);

        /// <summary>
        /// Prevents a default instance of the <see cref="InputManager" /> class from being created.
        /// </summary>
        private InputManager()
        {
            this.ControllerSubject = new ControllerCapture();
            this.KeyboardSubject = new KeyboardCapture();
            this.MouseSubject = new MouseCapture();

            this.PollUpdates();
        }

        /// <summary>
        /// Gets or sets the controller capture interface.
        /// </summary>
        private IControllerSubject ControllerSubject { get; set; }

        /// <summary>
        /// Gets or sets the keyboard capture interface.
        /// </summary>
        private KeyboardCapture KeyboardSubject { get; set; }

        /// <summary>
        /// Gets or sets the mouse capture interface.
        /// </summary>
        private IMouseSubject MouseSubject { get; set; }

        /// <summary>
        /// Gets an instance of the <see cref="InputManager"/> class.
        /// </summary>
        /// <returns>An instance of the <see cref="InputManager"/> class.</returns>
        public static InputManager GetInstance()
        {
            return InputManager.InputManagerInstance.Value;
        }

        /// <summary>
        /// Gets the keyboard capture interface.
        /// </summary>
        /// <returns>The keyboard capture interface.</returns>
        public KeyboardCapture GetKeyboardCapture()
        {
            return this.KeyboardSubject;
        }

        /// <summary>
        /// Gets the mouse capture interface.
        /// </summary>
        /// <returns>The mouse capture interface.</returns>
        public IControllerSubject GetControllerCapture()
        {
            return this.ControllerSubject;
        }

        /// <summary>
        /// Gets the controller capture interface.
        /// </summary>
        /// <returns>The controller capture interface.</returns>
        public IMouseSubject GetMouseCapture()
        {
            return this.MouseSubject;
        }

        /// <summary>
        /// Updates the input capture devices, polling the system for changes on each device.
        /// </summary>
        private void PollUpdates()
        {
            Task.Run(async () =>
            {
                while (true)
                {
                    this.ControllerSubject.Update();
                    this.KeyboardSubject.Update();
                    this.MouseSubject.Update();

                    await Task.Delay(InputManager.InputCollectionInterval);
                }
            });
        }
    }
    //// End class
}
//// End namespace