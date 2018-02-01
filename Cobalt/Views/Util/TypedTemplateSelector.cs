using System.Windows;
using System.Windows.Controls;

namespace Cobalt.Views.Util
{
    public class TypedTemplateSelector : DataTemplateSelector
    {
        public TypedTemplateEntries Entries { get; set; }

        public override DataTemplate SelectTemplate(object item, DependencyObject container)
        {
            if (Entries == null)
                return base.SelectTemplate(item, container);
            foreach (var kv in Entries)
                if (kv.Type == item?.GetType())
                    return kv.Template;
            return base.SelectTemplate(item, container);
        }
    }
}