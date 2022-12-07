namespace Squalr.Engine.Projects
{
    using Squalr.Engine.Common.Extensions;
    using Squalr.Engine.Projects.Properties;
    using System;
    using System.IO;

    /// <summary>
    /// A class for interfacing with saved project settings.
    /// </summary>
    public static class ProjectSettings
    {
        /// <summary>
        /// Gets or sets the project root from which all projects are saved and read.
        /// </summary>
        public static String ProjectRoot
        {
            get
            {
                if (Settings.Default.ProjectRoot.IsNullOrEmpty())
                {
                    ProjectSettings.ProjectRoot = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments), "Squalr");
                }

                return Settings.Default.ProjectRoot;
            }

            set
            {
                Settings.Default.ProjectRoot = value;
            }
        }
    }
    //// End class
}
//// End namespace
