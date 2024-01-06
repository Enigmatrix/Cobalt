using System;
using Avalonia;
using Avalonia.Controls;

namespace Cobalt.Controls;

public partial class DurationTextBlock : TextBlock
{
    public static DirectProperty<DurationTextBlock, TimeSpan?> DurationProperty =
        AvaloniaProperty.RegisterDirect<DurationTextBlock, TimeSpan?>(nameof(Duration), o => o.Duration,
            (o, v) => o.Duration = v, enableDataValidation: true);

    private TimeSpan? _duration;

    public DurationTextBlock()
    {
        InitializeComponent();
    }

    public TimeSpan? Duration
    {
        get => _duration;
        set => SetAndRaise(DurationProperty, ref _duration, value);
    }
}