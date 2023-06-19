using System;
using Avalonia.Controls;

namespace Cobalt.Views;

public static class ViewBaseExtensions
{
    public static void WhenActivated(this Control control, Func<Action> activation)
    {
        Action cleanup = default!;

        void Loaded(object? _, EventArgs args)
        {
            cleanup = activation();
        }

        void Unloaded(object? _, EventArgs args)
        {
            cleanup();
            control.Loaded -= Loaded;
            control.Unloaded -= Unloaded;
        }

        control.Loaded += Loaded;
        control.Unloaded += Unloaded;
    }

    /*
    public static void WhenActivated(this Visual visual, Func<Action> activation)
    {
        Action cleanup = default!;
        void Attached(object? _, EventArgs args)
        {
            cleanup = activation();
        }

        void Detached(object? _, EventArgs args)
        {
            cleanup();
            visual.AttachedToVisualTree -= Attached;
            visual.DetachedFromVisualTree -= Detached;
        }

        visual.AttachedToVisualTree += Attached;
        visual.DetachedFromVisualTree += Detached;
    }
    */
}