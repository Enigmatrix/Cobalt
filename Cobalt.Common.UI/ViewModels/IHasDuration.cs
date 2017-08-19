using System;

namespace Cobalt.Common.UI.ViewModels
{
    public interface IHasDuration
    {
        TimeSpan Duration { get; set; }
    }
}