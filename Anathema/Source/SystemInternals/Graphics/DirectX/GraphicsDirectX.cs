﻿using Anathema.Source.Graphics;
using Anathema.Source.Utils.Extensions;
using DirectXShell;
using DirectXShell.Hook;
using DirectXShell.Interface;
using System;
using System.Diagnostics;
using System.Drawing;
using System.Threading;

namespace Anathema.Source.SystemInternals.Graphics.DirectX
{
    class GraphicsDirextX : IGraphicsInterface
    {
        private DirectXShell.Capture DirectXShell;

        public GraphicsDirextX()
        {

        }

        public void Inject(Process Process)
        {
            if (DirectXShell != null)
                return;

            // Must be running as Administrator to allow dynamic registration in GAC
            // Config.Register("Name", "Name.dll");

            // Skip if the process is already hooked (and we want to hook multiple applications)
            if (Process.MainWindowHandle == IntPtr.Zero || HookManager.IsHooked(Process.Id))
            {
                return;
            }

            Direct3DVersionEnum Direct3DVersion = Direct3DVersionEnum.AutoDetect;

            CaptureConfig CaptureConfig = new CaptureConfig()
            {
                Direct3DVersion = Direct3DVersion,
                ShowOverlay = true
            };

            CaptureInterface CaptureInterface = new CaptureInterface();
            CaptureInterface.RemoteMessage += new MessageReceivedEvent(CaptureInterface_RemoteMessage);
            DirectXShell = new DirectXShell.Capture(Process, CaptureConfig, CaptureInterface);
        }

        public void Uninject()
        {
            if (DirectXShell == null)
                return;

            HookManager.RemoveHookedProcess(DirectXShell.Process.Id);
            DirectXShell.CaptureInterface.Disconnect();
            DirectXShell = null;
        }

        /// <summary>
        /// Create the screen shot request
        /// </summary>
        public void DoRequest()
        {
            Size? resize = new Size(1000, 1000);

            DirectXShell.CaptureInterface.BeginGetScreenshot(new Rectangle(0, 0, 1000, 100),
                new TimeSpan(0, 0, 2), Callback, resize, ImageFormatEnum.Bitmap);
        }

        public void DrawLine(int StartX, int StartY, int EndX, int EndY)
        {
            throw new NotImplementedException();
        }

        /// <summary>
        /// Display messages from the target process
        /// </summary>
        /// <param name="Message"></param>
        public void CaptureInterface_RemoteMessage(MessageReceivedEventArgs Message)
        {
            // k
        }

        /// <summary>
        /// The callback for when the screenshot has been taken
        /// </summary>
        /// <param name="clientPID"></param>
        /// <param name="status"></param>
        /// <param name="screenshotResponse"></param>
        public void Callback(IAsyncResult result)
        {
            this.PrintDebugTag();

            if (DirectXShell == null)
                return;

            using (Screenshot Screenshot = DirectXShell.CaptureInterface.EndGetScreenshot(result))

                try
                {
                    DirectXShell.CaptureInterface.DisplayInGameText("Screenshot captured...");

                    if (Screenshot != null && Screenshot.Data != null)
                    {

                    }

                    Thread Thread = new Thread(new ThreadStart(DoRequest));
                    Thread.Start();
                }
                catch
                {

                }
        }

    } // End class

} // End namespace