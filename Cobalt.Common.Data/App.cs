using System;

namespace Cobalt.Common.Data
{
    public class App : Entity
    {
        public string Name { get; set; }
        public string Path { get; set; }
        public IObservable<Tag> Tags { get; set; }
    }
}