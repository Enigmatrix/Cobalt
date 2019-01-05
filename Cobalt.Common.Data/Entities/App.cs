using System;
using System.Collections.Generic;
using System.Text;

namespace Cobalt.Common.Data.Entities
{
    public class App : Entity
    {
        public string Name { get; set; }
        public string Color { get; set; }
        public string Path { get; set; }
        public Lazy<byte[]> Icon { get; set; }
    }
}
