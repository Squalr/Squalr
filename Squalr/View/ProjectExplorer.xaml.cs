namespace Squalr.View
{
    using Squalr.Engine.Common.DataStructures;
    using Squalr.Engine.Projects.Items;
    using Squalr.Source.ProjectExplorer;
    using Squalr.Source.ProjectExplorer.ProjectItems;
    using Squalr.Source.PropertyViewer;
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using System.Reflection;
    using System.Windows;
    using System.Windows.Controls;
    using System.Windows.Input;

    /// <summary>
    /// Interaction logic for Settings.xaml.
    /// </summary>
    public partial class ProjectExplorer : UserControl
    {
        private static readonly PropertyInfo IsSelectionChangeActiveProperty = typeof(TreeView).GetProperty("IsSelectionChangeActive", BindingFlags.NonPublic | BindingFlags.Instance);

        /// <summary>
        /// Initializes a new instance of the <see cref="ProjectExplorer" /> class.
        /// </summary>
        public ProjectExplorer()
        {
            this.InitializeComponent();

            this.ProjectItemCache = new TtlCache<ProjectItemView>();

            // This works, but can be offloaded to a helper class, or perhaps rolled into the viewmodel itself.
            // Should be modified to support keyboard ctrl/shift+arrow stuff.
            // It's shit, but it's a great place to start.
            ProjectExplorer.AllowMultiSelection(this.ProjectExplorerTreeView);
        }

        private TtlCache<ProjectItemView> ProjectItemCache { get; set; }

        /// <summary>
        /// Modifies a <see cref="TreeView"/> to support multi-select.
        /// </summary>
        /// <param name="treeView">The <see cref="TreeView"/> to grant multi-select behavior.</param>
        public static void AllowMultiSelection(TreeView treeView)
        {
            if (IsSelectionChangeActiveProperty == null)
            {
                return;
            }

            treeView.SelectedItemChanged += (a, b) =>
            {
                if (ProjectExplorerViewModel.GetInstance().SelectedProjectItems == null)
                {
                    ProjectExplorerViewModel.GetInstance().SelectedProjectItems = new List<ProjectItemView>();
                }

                Boolean isShiftSelecting = Keyboard.IsKeyDown(Key.LeftShift) || Keyboard.IsKeyDown(Key.RightShift);
                Boolean isControlSelecting = Keyboard.IsKeyDown(Key.LeftCtrl) || Keyboard.IsKeyDown(Key.RightCtrl);

                if (isShiftSelecting)
                {
                    // Suppress selection change notification, select all selected items, then restore selection change notifications
                    Object isSelectionChangeActive = IsSelectionChangeActiveProperty.GetValue(treeView, null);

                    IsSelectionChangeActiveProperty.SetValue(treeView, true, null);
                    ProjectExplorer.ShiftSelect(treeView);

                    IsSelectionChangeActiveProperty.SetValue(treeView, isSelectionChangeActive, null);
                }
                else if (isControlSelecting)
                {
                    ProjectExplorer.ReselectPriorSelectedItems(treeView);
                    ProjectExplorer.ToggleSelection(treeView);
                }
                else
                {
                    ProjectExplorer.NormalSelect(treeView);
                    ProjectExplorer.ToggleSelection(treeView);
                }

                PropertyViewerViewModel.GetInstance().SetTargetObjects(ProjectExplorerViewModel.GetInstance().SelectedProjectItems?.ToArray());
            };
        }

        /// <summary>
        /// Helper function for ctrl+selecting multiple items in a <see cref="TreeView"/>.
        /// </summary>
        /// <param name="treeView">The <see cref="TreeView"/> being selected.</param>
        private static void ReselectPriorSelectedItems(TreeView treeView)
        {
            // Suppress selection change notification, select all selected items, then restore selection change notifications
            Object isSelectionChangeActive = IsSelectionChangeActiveProperty.GetValue(treeView, null);

            IsSelectionChangeActiveProperty.SetValue(treeView, true, null);
            ProjectExplorerViewModel.GetInstance().SelectedProjectItems?.ForEach(item => item.IsSelected = true);
            
            IsSelectionChangeActiveProperty.SetValue(treeView, isSelectionChangeActive, null);
        }

        /// <summary>
        /// Helper function for shift+selecting multiple items in a <see cref="TreeView"/>.
        /// </summary>
        /// <param name="treeView">The <see cref="TreeView"/> being selected.</param>
        private static void ShiftSelect(TreeView treeView)
        {
            ProjectItemView clickedTreeViewItem = treeView.SelectedItem as ProjectItemView;
            ProjectItemView selectedItem = ProjectExplorerViewModel.GetInstance().SelectedProjectItems?.FirstOrDefault();
            DirectoryItemView root = ProjectExplorerViewModel.GetInstance().ProjectRoot?.FirstOrDefault();

            if (root == null || clickedTreeViewItem == null || selectedItem == null)
            {
                return;
            }

            Boolean isSelecting = (root == selectedItem) || (root == clickedTreeViewItem);

            ProjectExplorerViewModel.GetInstance().SelectedProjectItems?.Clear();
            ProjectExplorer.SelectRange(root, selectedItem, clickedTreeViewItem, ref isSelecting);
        }

        /// <summary>
        /// Selects a range of project items between a start and end project item, based on what directories are expanded and collapsed between them.
        /// </summary>
        /// <param name="currentDirectory">The current directory being evaluated for recursion. Initially this should be the project root.</param>
        /// <param name="rangeStart">The first project item view in the selection range.</param>
        /// <param name="rangeEnd">The last project item view in the selection range.</param>
        /// <param name="isSelecting">A value indicating whether selection is still occuring for the recursive call.</param>
        /// <returns>A value indicating whether selection is complete, which happens when encountering the start and end range project view items.</returns>
        private static Boolean SelectRange(DirectoryItemView currentDirectory, ProjectItemView rangeStart, ProjectItemView rangeEnd, ref Boolean isSelecting)
        {
            Boolean selectionComplete = false;

            if (currentDirectory.ChildItems == null)
            {
                return selectionComplete;
            }

            foreach (ProjectItem projectItem in currentDirectory.ChildItems)
            {
                ProjectItemView projectItemView = projectItem.MappedView as ProjectItemView;
                DirectoryItemView directoryItemView = projectItemView as DirectoryItemView;
                Boolean selectionStarted = false;

                if (!isSelecting)
                {
                    isSelecting = (projectItemView == rangeStart) || (projectItemView == rangeEnd);
                    selectionStarted = true;
                }

                if (projectItemView != null)
                {
                    projectItemView.IsSelected = isSelecting;

                    if (isSelecting)
                    {
                        ProjectExplorerViewModel.GetInstance().SelectedProjectItems?.Add(projectItemView);
                    }
                }

                if (directoryItemView != null && directoryItemView.IsExpanded)
                {
                    selectionComplete = ProjectExplorer.SelectRange(directoryItemView, rangeStart, rangeEnd, ref isSelecting);
                }

                if (!selectionStarted && isSelecting && ((projectItemView == rangeStart) || (projectItemView == rangeEnd)))
                {
                    isSelecting = false;
                    selectionComplete = true;
                    break;
                }
            }

            return selectionComplete;
        }

        /// <summary>
        /// Performs a normal single select on a <see cref="TreeView"/>, deselecting all other items.
        /// </summary>
        /// <param name="treeView">The <see cref="TreeView"/> being selected.</param>
        private static void NormalSelect(TreeView treeView)
        {
            // Feselect all selected items except the current one
            ProjectItemView clickedTreeViewItem = treeView.SelectedItem as ProjectItemView;
            ProjectExplorerViewModel.GetInstance().SelectedProjectItems?.ForEach(item => item.IsSelected = item == clickedTreeViewItem);
            ProjectExplorerViewModel.GetInstance().SelectedProjectItems?.Clear();
        }


        // WIP Drag n drop

        private static void ToggleSelection(TreeView treeView)
        {
            ProjectItemView clickedTreeViewItem = treeView.SelectedItem as ProjectItemView;

            if (clickedTreeViewItem == null)
            {
                return;
            }

            if (!ProjectExplorerViewModel.GetInstance().SelectedProjectItems?.Contains(clickedTreeViewItem) ?? false)
            {
                ProjectExplorerViewModel.GetInstance().SelectedProjectItems?.Add(clickedTreeViewItem);
            }
            else
            {
                clickedTreeViewItem.IsSelected = false;
                ProjectExplorerViewModel.GetInstance().SelectedProjectItems?.Remove(clickedTreeViewItem);
            }
        }

        private void ValueMouseDown(Object sender, MouseButtonEventArgs e)
        {
            ProjectItemView hitResult = (sender as FrameworkElement)?.DataContext as ProjectItemView;

            if (hitResult == null)
            {
                return;
            }

            if (this.ProjectItemCache.Contains(hitResult))
            {
                ProjectExplorerViewModel.GetInstance().EditProjectItemValueCommand.Execute(hitResult);
            }
            else
            {
                this.ProjectItemCache.Invalidate();
                this.ProjectItemCache.Add(hitResult, TimeSpan.FromMilliseconds(System.Windows.Forms.SystemInformation.DoubleClickTime));
            }
        }

        private void NameMouseDown(Object sender, MouseButtonEventArgs e)
        {
            ProjectItemView hitResult = (sender as FrameworkElement)?.DataContext as ProjectItemView;

            if (hitResult == null)
            {
                return;
            }

            if (this.ProjectItemCache.Contains(hitResult))
            {
                ProjectExplorerViewModel.GetInstance().RenameProjectItemCommand.Execute(hitResult);
            }
            else
            {
                this.ProjectItemCache.Invalidate();
                this.ProjectItemCache.Add(hitResult, TimeSpan.FromMilliseconds(System.Windows.Forms.SystemInformation.DoubleClickTime));
            }
        }

        Point _startPoint;
        bool _IsDragging = false;

        void TemplateTreeView_PreviewMouseMove(object sender, MouseEventArgs e)
        {
            if (e.LeftButton == MouseButtonState.Pressed ||
                e.RightButton == MouseButtonState.Pressed && !_IsDragging)
            {
                Point position = e.GetPosition(null);
                if (Math.Abs(position.X - _startPoint.X) >
                        SystemParameters.MinimumHorizontalDragDistance ||
                    Math.Abs(position.Y - _startPoint.Y) >
                        SystemParameters.MinimumVerticalDragDistance)
                {
                    StartDrag(e);
                }
            }
        }

        void TemplateTreeView_PreviewMouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            _startPoint = e.GetPosition(null);
        }

        private void StartDrag(MouseEventArgs e)
        {
            _IsDragging = true;
            Object temp = this.ProjectExplorerTreeView.SelectedItem;

            if (temp == null)
            {
                return;
            }

            DataObject data = null;

            data = new DataObject("inadt", temp);

            if (data != null)
            {
                DragDropEffects dde = DragDropEffects.Move;

                if (e.RightButton == MouseButtonState.Pressed)
                {
                    dde = DragDropEffects.All;
                }

                DragDropEffects de = DragDrop.DoDragDrop(this.ProjectExplorerTreeView, data, dde);
            }
            _IsDragging = false;
        }

        private void ProjectExplorerTreeView_Drop(object sender, DragEventArgs e)
        {

        }
    }
    //// End class
}
//// End namespace