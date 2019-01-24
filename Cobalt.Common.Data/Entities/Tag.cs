using System;

namespace Cobalt.Common.Data.Entities
{
    public class Tag : Entity
    {
        public string Name { get; set; }
        public string ForegroundColor { get; set; }
        public string BackgroundColor { get; set; }
        public IObservable<App> Apps { get; set; }
    }
}