﻿namespace Ana.Source.Utils.ScriptEditor
{
    using Docking;
    using Main;
    using System;
    using System.Threading.Tasks;

    /// <summary>
    /// View model for the Script Editor
    /// </summary>
    internal class ScriptEditorViewModel : ToolViewModel
    {
        /// <summary>
        /// The content id for the docking library associated with this view model
        /// </summary>
        public const String ToolContentId = nameof(ScriptEditorViewModel);

        /// <summary>
        /// Initializes a new instance of the <see cref="ScriptEditorViewModel" /> class
        /// </summary>
        public ScriptEditorViewModel() : base("Script Editor")
        {
            this.ContentId = ScriptEditorViewModel.ToolContentId;

            Task.Run(() => MainViewModel.GetInstance().Subscribe(this));
        }

        public String Script { get; set; }
    }
    //// End class
}
//// End namespace