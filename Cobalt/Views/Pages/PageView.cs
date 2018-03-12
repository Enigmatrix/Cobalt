using System.Windows;
using System.Windows.Controls;

namespace Cobalt.Views.Pages
{
    public class PageView : UserControl
    {
        public static readonly DependencyProperty TitleProperty =
            DependencyProperty.Register("Title", typeof(object), typeof(PageView), new PropertyMetadata(""));

        public object Title
        {
            get => GetValue(TitleProperty);
            set => SetValue(TitleProperty, value);
        }
    }
}