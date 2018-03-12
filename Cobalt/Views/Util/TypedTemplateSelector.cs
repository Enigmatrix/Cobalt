using System.Windows;
using System.Windows.Controls;

namespace Cobalt.Views.Util
{
    public class TypedTemplateSelector : DataTemplateSelector
    {
        public TypedTemplateEntries Entries { get; set; }

        public override DataTemplate SelectTemplate(object item, DependencyObject container)
        {
            if (Entries == null || item == null)
                return base.SelectTemplate(item, container);
            foreach (var kv in Entries)
                if (kv.Type.IsInstanceOfType(item))
                    return kv.Template;
            return base.SelectTemplate(item, container);
        }
    }
}