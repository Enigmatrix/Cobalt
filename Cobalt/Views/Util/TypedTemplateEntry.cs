using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
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
