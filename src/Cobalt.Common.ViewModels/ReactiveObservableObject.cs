using System.ComponentModel;
using CommunityToolkit.Mvvm.ComponentModel;
using ReactiveUI;

namespace Cobalt.Common.ViewModels;

public class ReactiveObservableObject : ObservableObject, IReactiveObject
{
    public ReactiveObservableObject()
    {
        this.SubscribePropertyChangingEvents();
        this.SubscribePropertyChangedEvents();
    }

    public void RaisePropertyChanging(PropertyChangingEventArgs args)
    {
        base.OnPropertyChanging(args);
    }

    public void RaisePropertyChanged(PropertyChangedEventArgs args)
    {
        base.OnPropertyChanged(args);
    }

    protected override void OnPropertyChanging(PropertyChangingEventArgs e)
    {
        this.RaisePropertyChanging(e.PropertyName);
    }

    protected override void OnPropertyChanged(PropertyChangedEventArgs e)
    {
        this.RaisePropertyChanged(e.PropertyName);
    }
}