namespace Squalr.Source.Mvvm.AttachedBehaviors
{
    using Squalr.Source.Utils.Extensions;
    using System;
    using System.ComponentModel;
    using System.Windows;
    using System.Windows.Controls;
    using System.Windows.Controls.Primitives;

    public static class ScrollToTopBehavior
    {
        public static readonly DependencyProperty ScrollToTopProperty = DependencyProperty.RegisterAttached(
            "ScrollToTop",
            typeof(Boolean),
            typeof(ScrollToTopBehavior),
            new UIPropertyMetadata(false, OnScrollToTopPropertyChanged));

        public static Boolean GetScrollToTop(DependencyObject obj)
        {
            return (Boolean)obj.GetValue(ScrollToTopProperty);
        }

        public static void SetScrollToTop(DependencyObject obj, Boolean value)
        {
            obj.SetValue(ScrollToTopProperty, value);
        }

        private static void OnScrollToTopPropertyChanged(DependencyObject dpo, DependencyPropertyChangedEventArgs e)
        {
            ItemsControl itemsControl = dpo as ItemsControl;

            if (itemsControl != null)
            {
                DependencyPropertyDescriptor dependencyPropertyDescriptor =
                        DependencyPropertyDescriptor.FromProperty(ItemsControl.ItemsSourceProperty, typeof(ItemsControl));

                if (dependencyPropertyDescriptor != null)
                {
                    if ((Boolean)e.NewValue == true)
                    {
                        dependencyPropertyDescriptor.AddValueChanged(itemsControl, ItemsSourceChanged);
                    }
                    else
                    {
                        dependencyPropertyDescriptor.RemoveValueChanged(itemsControl, ItemsSourceChanged);
                    }
                }
            }
        }

        private static void ItemsSourceChanged(Object sender, EventArgs e)
        {
            ItemsControl itemsControl = sender as ItemsControl;
            EventHandler eventHandler = null;

            eventHandler = new EventHandler(delegate
            {
                if (itemsControl.ItemContainerGenerator.Status == GeneratorStatus.ContainersGenerated)
                {
                    ScrollViewer scrollViewer = itemsControl.GetVisualChild<ScrollViewer>() as ScrollViewer;
                    scrollViewer.ScrollToTop();
                    itemsControl.ItemContainerGenerator.StatusChanged -= eventHandler;
                }
            });

            itemsControl.ItemContainerGenerator.StatusChanged += eventHandler;
        }
    }
    //// End class
}
//// End namespace