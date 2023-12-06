using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels.Pages;

public abstract class PageViewModelBase : ViewModelBase, IActivatableViewModel
{
    protected PageViewModelBase(IDbContextFactory<QueryContext> contexts)
    {
        Contexts = contexts;
    }

    public abstract string Name { get; }
    protected IDbContextFactory<QueryContext> Contexts { get; }
    public ViewModelActivator Activator { get; } = new();
}