using ReactiveUI;

namespace Cobalt.Common.ViewModels.Pages;

public abstract class PageViewModelBase : ViewModelBase, IActivatableViewModel
{
    public abstract string Name { get; }
    public ViewModelActivator Activator { get; } = new();
}