using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels.Entities;

public partial class WithDuration<T> : ObservableObject, IHasDuration, IHasEntity
    where T : IEntity
{
    [ObservableProperty] private TimeSpan _duration;

    [ObservableProperty] private T _entity;

    public WithDuration(T inner, TimeSpan duration)
    {
        Entity = inner;
        Duration = duration;
    }

    public IEntity Inner => Entity;
}