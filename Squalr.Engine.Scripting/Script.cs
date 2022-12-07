namespace Squalr.Engine.Scripting
{
    using Squalr.Engine.Common.Logging;
    using System;
    using System.Reflection;
    using System.Security;
    using System.Threading;
    using System.Threading.Tasks;

    /// <summary>
    /// Instance of a single script.
    /// </summary>
    public abstract class Script
    {
        /// <summary>
        /// Time to wait for the update loop to finish on deactivation.
        /// </summary>
        private const Int32 AbortTime = 500;

        /// <summary>
        /// Update time in milliseconds.
        /// </summary>
        private const Int32 UpdateTime = 1000 / 15;

        /// <summary>
        /// Initializes a new instance of the <see cref="Script" /> class.
        /// </summary>
        protected Script()
        {
        }

        /// <summary>
        /// Gets or sets the text of this script.
        /// </summary>
        public String Text { get; set; }

        /// <summary>
        /// Gets or sets the name of this script.
        /// </summary>
        public String Name { get; set; }

        /// <summary>
        /// Gets or sets a value indicating whether this script is active.
        /// </summary>
        public Boolean IsActivated { get; set; }

        /// <summary>
        /// Gets the compiled script assembly.
        /// </summary>
        public String CompiledAssembly { get; private set; }

        /// <summary>
        /// Gets or sets the task for the update loops.
        /// </summary>
        private Task Task { get; set; }

        /// <summary>
        /// Gets or sets a cancelation request for the update loop.
        /// </summary>
        private CancellationTokenSource CancelRequest { get; set; }

        /// <summary>
        /// Gets or sets the compiled assembly object of a script.
        /// </summary>
        private dynamic ScriptObject { get; set; }

        /// <summary>
        /// Creates a script object from the given assembly.
        /// </summary>
        /// <param name="assembly">The assembly from which to create the script.</param>
        /// <returns>The script created from the assembly.</returns>
        public static Script FromAssembly(Assembly assembly)
        {
            dynamic script = assembly?.CreateInstance("Squalr.Engine.Scripting.Script");

            return script;
        }

        /// <summary>
        /// Runs the activation function in the script.
        /// </summary>
        /// <returns>Returns true if the activation function successfully ran, otherwise false.</returns>
        public Boolean RunActivationFunction()
        {
            try
            {
                // Bind the deactivation function such that scripts can deactivate themselves
                this.ScriptObject.Deactivate = new Action(() => this.IsActivated = false);

                // Call OnActivate function in the script
                this.ScriptObject.OnActivate();

                Logger.Log(LogLevel.Info, "Script activated: " + this.Name);
            }
            catch (SecurityException ex)
            {
                Logger.Log(LogLevel.Error, "Invalid operation in sandbox environment", ex);
                return false;
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Unable to activate script", ex);
                return false;
            }

            return true;
        }

        /// <summary>
        /// Continously runs the update function in the script.
        /// </summary>
        public void RunUpdateFunction()
        {
            this.CancelRequest = new CancellationTokenSource();

            try
            {
                this.Task = Task.Run(
                async () =>
                {
                    TimeSpan elapsedTime;
                    DateTime previousTime = DateTime.Now;

                    while (true)
                    {
                        DateTime currentTime = DateTime.Now;
                        elapsedTime = currentTime - previousTime;

                        try
                        {
                            // Call the update function, giving the elapsed milliseconds since the previous call
                            ScriptObject.OnUpdate((Single)elapsedTime.TotalMilliseconds);
                        }
                        catch (Exception ex)
                        {
                            String exception = ex.ToString();

                            if (exception.ToString().Contains("does not contain a definition for 'OnUpdate'"))
                            {
                                Logger.Log(LogLevel.Warn, "Optional update function not executed");
                            }
                            else
                            {
                                Logger.Log(LogLevel.Error, "Error running update function: ", ex);
                            }

                            return;
                        }

                        previousTime = currentTime;

                        // Await with cancellation
                        await Task.Delay(Script.UpdateTime, this.CancelRequest.Token);
                    }
                },
                this.CancelRequest.Token);

                return;
            }
            catch
            {
                Logger.Log(LogLevel.Error, "Error executing update loop.");
            }
        }

        /// <summary>
        /// Runs the deactivation function in the script.
        /// </summary>
        public void RunDeactivationFunction()
        {
            // Abort the update loop
            try
            {
                this.ScriptObject.OnDeactivate();

                Logger.Log(LogLevel.Info, "Script deactivated: " + this.Name);

                try
                {
                    this.CancelRequest?.Cancel();
                    this.Task?.Wait(Script.AbortTime);
                }
                catch
                {
                }
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error when deactivating script", ex);
            }

            return;
        }
    }
    //// End class
}
//// End namespace