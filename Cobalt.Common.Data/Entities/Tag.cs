using System;
using System.Collections.Generic;
using System.Text;

namespace Cobalt.Common.Data.Entities
{
    public class Tag : Entity
    {
        public string Name { get; set; }
        public string ForegroundColor { get; set; }
        public string BackgroundColor { get; set; }
    }
}
