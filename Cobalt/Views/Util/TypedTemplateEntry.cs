using System;
using System.Collections.ObjectModel;
using System.Windows;

namespace Cobalt.Views.Util
{
    public class TypedTemplateEntry
    {
        public Type Type { get; set; }
        public DataTemplate Template { get; set; }
    }

    public class TypedTemplateEntries : Collection<TypedTemplateEntry>
    {
    }
}