using System.Collections.Generic;

namespace Cobalt.Common.Data
{
    public class App : Entity
    {
        public string Name { get; set; }
        public string Path { get; set; }
        public List<Tag> Tags { get; set; }
    }
}