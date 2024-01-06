using System;
using Avalonia.Controls.Primitives;
using Avalonia.Input;

namespace Cobalt.Controls;

public class KeylessToggleButton : ToggleButton
{
    protected override Type StyleKeyOverride { get; } = typeof(ToggleButton);

    // ignore space
    protected override void OnKeyDown(KeyEventArgs e)
    {
        if (e.Key == Key.Space)
        {
            e.Handled = false;
            return;
        }

        base.OnKeyDown(e);
    }

    /*protected override void OnKeyUp(KeyEventArgs e)
    {
        // handle enter etc
        base.OnKeyDown(e);
        base.OnKeyUp(e);
    }*/
}