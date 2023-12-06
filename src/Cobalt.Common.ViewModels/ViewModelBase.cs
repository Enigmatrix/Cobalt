using CommunityToolkit.Mvvm.ComponentModel;
using ReactiveUI;

namespace Cobalt.Common.ViewModels;

/// <summary>
///     Base class for all ViewModels.
/// </summary>
/// <remarks>
///     We derive from CommunityToolkit's <see cref="ObservableObject" /> instead of ReactiveUI's
///     <see cref="ReactiveObject" />
///     otherwise the CommunityToolkit source generators will complain.
/// </remarks>
public abstract class ViewModelBase : ObservableObject
{
}