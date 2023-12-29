using System.ComponentModel;
using CommunityToolkit.Mvvm.ComponentModel;
using ReactiveUI;

namespace Cobalt.Common.ViewModels;

/// <summary>
///     Combination of <see cref="ObservableObject" /> and <see cref="IReactiveObject" />
/// </summary>
public class ReactiveObservableObject : ObservableObject, IReactiveObject
{
    /*
     * Note the calls to SubscribeProperty* and the RaiseProperty* calls. Both need to present.
     * Furthermore, even if we do not need to directly never call RaiseProperty* ourselves, we should anyway.
     * ReactiveUI seems to attach interceptors to those methods to check whether they are called. This is documented
     * nowhere. The ExtendedState attached by SubscribeProperty* comes into play somehow.
     */

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