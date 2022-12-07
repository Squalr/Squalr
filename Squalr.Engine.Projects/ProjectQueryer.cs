namespace Squalr.Engine.Projects
{
    using Squalr.Engine.Common.Logging;
    using System;
    using System.Collections.Generic;
    using System.IO;
    using System.Linq;

    /// <summary>
    /// A class for querying saved projects.
    /// </summary>
    public static class ProjectQueryer
    {
        /// <summary>
        /// Gets the list of all saved project names under the project root.
        /// </summary>
        /// <returns>The list of all saved project names under the project root.</returns>
        public static IEnumerable<String> GetProjectNames()
        {
            try
            {
                return Directory.EnumerateDirectories(ProjectSettings.ProjectRoot).Select(ProjectQueryer.ProjectPathToName);
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error reading project names from disk.", ex);

                return new List<String>();
            }
        }

        /// <summary>
        /// Gets the list of all saved project paths under the project root.
        /// </summary>
        /// <returns>The list of all saved project paths under the project root.</returns>
        public static IEnumerable<String> GetProjectPaths()
        {
            try
            {
                return Directory.EnumerateDirectories(ProjectSettings.ProjectRoot);
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error reading project paths from disk.", ex);

                return new List<String>();
            }
        }

        /// <summary>
        /// Converts a project name into the associated project path.
        /// </summary>
        /// <param name="projectName">The name of the project.</param>
        /// <returns>The project path associated with the given project name.</returns>
        public static String ProjectNameToPath(String projectName)
        {
            try
            {
                return Path.Combine(ProjectSettings.ProjectRoot, projectName);
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error converting project name to path.", ex);

                return String.Empty;
            }
        }

        /// <summary>
        /// Converts a project path into the associated project name.
        /// </summary>
        /// <param name="projectPath">The path of the project.</param>
        /// <returns>The project name associated with the given project path.</returns>
        public static String ProjectPathToName(String projectPath)
        {
            try
            {
                return new DirectoryInfo(projectPath).Name;
            }
            catch (Exception ex)
            {
                Logger.Log(LogLevel.Error, "Error converting project path to name.", ex);

                return String.Empty;
            }
        }
    }
    //// End class
}
//// End namespace