using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels.Entities;

public abstract partial class EntityViewModelBase : ViewModelBase
{
    [ObservableProperty] private long _id;
}