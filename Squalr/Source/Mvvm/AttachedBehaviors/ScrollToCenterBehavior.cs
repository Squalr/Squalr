namespace Squalr.Source.Mvvm.AttachedBehaviors
{
    using System;
    using System.ComponentModel;
    using System.Windows;
    using System.Windows.Controls;
    using System.Windows.Controls.Primitives;

    public static class ScrollToCenterBehavior
    {
        public static readonly DependencyProperty ScrollToCenterProperty = DependencyProperty.RegisterAttached(
            "ScrollToCenter",
            typeof(Boolean),
            typeof(ScrollToCenterBehavior),
            new UIPropertyMetadata(false, OnScrollToCenterPropertyChanged));

        public static Boolean GetScrollToCenter(DependencyObject obj)
        {
            return (Boolean)obj.GetValue(ScrollToCenterProperty);
        }

        public static void SetScrollToCenter(DependencyObject obj, Boolean value)
        {
            obj.SetValue(ScrollToCenterProperty, value);
        }

        private static void OnScrollToCenterPropertyChanged(DependencyObject dpo, DependencyPropertyChangedEventArgs e)
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
                    DataGrid dataGrid = itemsControl as DataGrid;
                    dataGrid?.ScrollIntoView(dataGrid.Items[dataGrid.Items.Count / 2]);
                    itemsControl.ItemContainerGenerator.StatusChanged -= eventHandler;
                }
            });

            itemsControl.ItemContainerGenerator.StatusChanged += eventHandler;
        }
    }
    //// End class
}
//// End namespace