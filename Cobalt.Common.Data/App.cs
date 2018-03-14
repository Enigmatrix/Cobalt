using System;

namespace Cobalt.Common.Data
{
    public class App : Entity
    {
        public string Name { get; set; }
        public string Path { get; set; }
        public string Color { get; set; }
        public IObservable<byte[]> Icon { get; set; }
        public IObservable<Tag> Tags { get; set; }
    }
}