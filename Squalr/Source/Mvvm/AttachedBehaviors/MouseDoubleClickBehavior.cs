namespace Squalr.Source.Mvvm.AttachedBehaviors
{
    using Squalr.Engine.Common.DataStructures;
    using Squalr.Source.ProjectExplorer.ProjectItems;
    using System;
    using System.Windows;
    using System.Windows.Controls;
    using System.Windows.Input;

    public static class MouseDoubleClickBehavior
    {
        private static readonly DependencyProperty CommandProperty = DependencyProperty.RegisterAttached(
            "Command",
            typeof(ICommand),
            typeof(MouseDoubleClickBehavior),
            new UIPropertyMetadata(CommandChanged));

        private static readonly DependencyProperty CommandParameterProperty = DependencyProperty.RegisterAttached(
            "CommandParameter",
            typeof(Object),
            typeof(MouseDoubleClickBehavior),
            new UIPropertyMetadata(null));

        private static readonly TtlCache<Object> ClickedControls = new TtlCache<Object>(TimeSpan.FromMilliseconds(500));

        public static void SetCommand(DependencyObject target, ICommand value)
        {
            target.SetValue(CommandProperty, value);
        }

        public static void SetCommandParameter(DependencyObject target, Object value)
        {
            target.SetValue(CommandParameterProperty, value);
        }

        public static Object GetCommandParameter(DependencyObject target)
        {
            return target.GetValue(CommandParameterProperty);
        }

        public static TreeViewItem ContainerFromItem(this TreeView treeView, Object item)
        {
            TreeViewItem containerThatMightContainItem = (TreeViewItem)treeView.ItemContainerGenerator.ContainerFromItem(item);

            if (containerThatMightContainItem != null)
            {
                return containerThatMightContainItem;
            }
            else
            {
                return ContainerFromItem(treeView.ItemContainerGenerator, treeView.Items, item);
            }
        }

        private static void CommandChanged(DependencyObject target, DependencyPropertyChangedEventArgs e)
        {
            Control control = target as Control;

            if (control != null)
            {
                if ((e.NewValue != null) && (e.OldValue == null))
                {
                    control.PreviewMouseLeftButtonUp += OnMouseUp;
                }
                else if ((e.NewValue == null) && (e.OldValue != null))
                {
                    control.PreviewMouseLeftButtonUp -= OnMouseUp;
                }
            }
        }

        private static void OnMouseUp(Object sender, MouseButtonEventArgs e)
        {
            Control control = sender as Control;
            DirectoryItemView directoryItemView = control.DataContext as DirectoryItemView;

            if (directoryItemView != null)
            {
                return;
            }

            Object commandParameter = control?.GetValue(CommandParameterProperty);

            if (commandParameter != null)
            {
                if (MouseDoubleClickBehavior.ClickedControls.Contains(commandParameter))
                {
                    // Ideally the double click event would fire on mouse down instead of up, but some timing issues make this problematic
                    ICommand command = (ICommand)control.GetValue(CommandProperty);
                    command.Execute(commandParameter);
                }
                else
                {
                    MouseDoubleClickBehavior.ClickedControls.Invalidate();
                    MouseDoubleClickBehavior.ClickedControls.Add(commandParameter);
                }
            }
        }

        private static TreeViewItem ContainerFromItem(ItemContainerGenerator parentItemContainerGenerator, ItemCollection itemCollection, Object item)
        {
            foreach (Object curChildItem in itemCollection)
            {
                TreeViewItem parentContainer = (TreeViewItem)parentItemContainerGenerator.ContainerFromItem(curChildItem);
                TreeViewItem containerThatMightContainItem = (TreeViewItem)parentContainer.ItemContainerGenerator.ContainerFromItem(item);

                if (containerThatMightContainItem != null)
                {
                    return containerThatMightContainItem;
                }

                TreeViewItem recursionResult = ContainerFromItem(parentContainer.ItemContainerGenerator, parentContainer.Items, item);

                if (recursionResult != null)
                {
                    return recursionResult;
                }
            }

            return null;
        }
    }
    //// End class
}
//// End namespace