using System.Windows;
using System.Windows.Controls;

namespace Cobalt.Views.Pages
{
    public class PageView : UserControl
    {
        public static readonly DependencyProperty TitleProperty =
            DependencyProperty.Register("Title", typeof(string), typeof(PageView), new PropertyMetadata(""));

        public string Title
        {
            get => (string) GetValue(TitleProperty);
            set => SetValue(TitleProperty, value);
        }
    }
}