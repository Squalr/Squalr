namespace Squalr.Engine.Projects
{
    using Squalr.Engine.Processes;
    using Squalr.Engine.Projects.Items;
    using System;
    using System.IO;

    /// <summary>
    /// Defines a Squalr project. This is the root directory that contains all other project items.
    /// </summary>
    public class Project : DirectoryItem
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="Project" /> class.
        /// </summary>
        /// <param name="processSession">A process session reference for accessing the current opened process.</param>
        /// <param name="projectFilePathOrName">The project path, or the project name.</param>
        public Project(ProcessSession processSession, String projectFilePathOrName) : base(processSession, Project.ToDirectory(projectFilePathOrName), null)
        {
        }

        /// <summary>
        /// Converts a project name into a project path, if necessary.
        /// </summary>
        /// <param name="projectFilePathOrName">The project path, or the project name.</param>
        /// <returns>The full path for this project file name.</returns>
        private static String ToDirectory(String projectFilePathOrName)
        {
            if (!Path.IsPathRooted(projectFilePathOrName))
            {
                projectFilePathOrName = Path.Combine(ProjectSettings.ProjectRoot, projectFilePathOrName);
            }

            return projectFilePathOrName;
        }
    }
    //// End class
}
//// End namespace