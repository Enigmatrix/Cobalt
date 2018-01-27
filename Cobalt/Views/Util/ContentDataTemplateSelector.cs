using System.Windows;
using System.Windows.Controls;

namespace Cobalt.Views.Util
{
    public class ContentDataTemplateSelector : DataTemplateSelector
    {
        public override DataTemplate SelectTemplate(object item, DependencyObject container)
        {
            var res = container as FrameworkElement;
            return (string) item == "Custom"
                ? (DataTemplate) res.FindResource("CustomDateRangeTemplate")
                : (DataTemplate) res.FindResource("TextDateRangeTemplate");
        }
    }
}