﻿namespace SqualrProxy
{
    using System;
    using System.Diagnostics;
    using System.Threading;
    using System.Threading.Tasks;

    /// <summary>
    /// Proxy service to be contained by a 32 and 64 bit service, with services exposed via IPC. Useful for certain things that
    /// Squalr requires, such as:
    /// - FASM Compiler, which can only be run in 32 bit mode
    /// - Microsoft.Diagnostics.Runtime, which can only be used on processes of the same bitness
    /// </summary>
    public class SqualrProxy
    {
        /// <summary>
        /// The delay in milliseconds to check if the parent process is still running.
        /// </summary>
        private const Int32 ParentCheckDelayMs = 500;

        /// <summary>
        /// Initializes a new instance of the <see cref="SqualrProxy" /> class.
        /// </summary>
        /// <param name="parentProcessId">The parent process id.</param>
        /// <param name="pipeName">The IPC pipe name.</param>
        /// <param name="waitEventName">The global wait event name signaled by the parent process, which allows us to signal when this process has started.</param>
        public SqualrProxy(Int32 parentProcessId, String pipeName, String waitEventName)
        {
            /*
            // Create an event to have the client wait until we are finished starting the service
            EventWaitHandle processStartingEvent = new EventWaitHandle(false, EventResetMode.ManualReset, waitEventName);

            this.InitializeAutoExit(parentProcessId);

            ServiceHost serviceHost = new ServiceHost(typeof(ProxyAssembler));
            serviceHost.Description.Behaviors.Remove(typeof(ServiceDebugBehavior));
            serviceHost.Description.Behaviors.Add(new ServiceDebugBehavior { IncludeExceptionDetailInFaults = true });
            NamedPipeServerStream binding = new NamedPipeServerStream(pipeName, PipeDirection.InOut);
            binding.ReadTimeout = Int32.MaxValue;
            serviceHost.AddServiceEndpoint(typeof(IProxyAssembler), binding, pipeName);
            serviceHost.Open();

            processStartingEvent.Set();

            Console.WriteLine("Squalr proxy library loaded");
            Console.ReadLine();
            */
        }

        /// <summary>
        /// Runs a loop constantly checking if the parent process still exists. This service closes when the parent is closed.
        /// </summary>
        /// <param name="parentProcessId">The process id of the parent process.</param>
        private void InitializeAutoExit(Int32 parentProcessId)
        {
            Task.Run(() =>
            {
                while (true)
                {
                    try
                    {
                        // Check if the process is still running
                        Process.GetProcessById(parentProcessId);
                    }
                    catch (ArgumentException)
                    {
                        // Could not find process
                        break;
                    }

                    Thread.Sleep(SqualrProxy.ParentCheckDelayMs);
                }

                Environment.Exit(0);
            });
        }
    }
    //// End class
}
//// End namespace