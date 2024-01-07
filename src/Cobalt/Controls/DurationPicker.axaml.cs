using System;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Data;
using Avalonia.Input;
using Avalonia.Interactivity;
using TimeSpanParserUtil;

namespace Cobalt.Controls;

// TODO right now, this control has 2 validation levels, one at the textbox and one at this control layer
// move the one at the textbox to the one at the control layer. The one at the textbox gives red border + red text
// but the one at the control layer only gives red text, it looks incongruous.
public partial class DurationPicker : UserControl
{
    public static readonly DirectProperty<DurationPicker, TimeSpan?> DurationProperty =
        AvaloniaProperty.RegisterDirect<DurationPicker, TimeSpan?>(
            nameof(Duration),
            o => o.Duration,
            (o, v) => o.Duration = v,
            defaultBindingMode: BindingMode.TwoWay, enableDataValidation: true);

    public static readonly DirectProperty<DurationPicker, string?> TextProperty =
        AvaloniaProperty.RegisterDirect<DurationPicker, string?>(
            nameof(Text),
            o => o.Text,
            (o, v) => o.Text = v,
            defaultBindingMode: BindingMode.TwoWay,
            enableDataValidation: true);

    private static readonly string InvalidDuration = "Invalid duration";

    private TimeSpan? _duration;
    private string? _text;

    public DurationPicker()
    {
        InitializeComponent();
    }

    public TimeSpan? Duration
    {
        get => _duration;
        set => SetAndRaise(DurationProperty, ref _duration, value);
    }

    public string? Text
    {
        get => _text;
        set => SetAndRaise(TextProperty, ref _text, value);
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);

        if (change.Property == TextProperty && change.NewValue != null)
        {
            var text = (string)change.NewValue;

            if (TimeSpanParser.TryParse(text, out var dur))
            {
                Duration = dur;
                RemoveError(this, InvalidDuration);
            }
            else
            {
                AddError(this, InvalidDuration);
            }
        }
        else if (change.Property == DurationProperty && change.NewValue == null)
        {
            Text = null;
        }
    }

    private static void RemoveError(Control control, object error)
    {
        var errors = DataValidationErrors.GetErrors(control);
        if (errors != null) DataValidationErrors.SetErrors(control, errors.Except([error]));
    }

    private static void AddError(Control control, object error)
    {
        var errors = DataValidationErrors.GetErrors(control);
        var existingErrors = Enumerable.Empty<object>();
        if (errors != null) existingErrors = errors.Except([error]);
        DataValidationErrors.SetErrors(control, existingErrors.Append(error));
    }

    protected override void UpdateDataValidation(AvaloniaProperty property, BindingValueType state, Exception? error)
    {
        if (property == DurationProperty)
            DataValidationErrors.SetError(this, error);
    }

    private void Display_OnClick(object? sender, RoutedEventArgs e)
    {
        Display.IsVisible = false;
        TextBox.IsVisible = true;
        TextBox.Focus();
    }

    private void TextBox_OnLostFocus(object? sender, RoutedEventArgs e)
    {
        SwitchToDisplay();
    }

    private void TextBox_OnKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key is Key.Enter or Key.Escape) SwitchToDisplay();
    }

    private void SwitchToDisplay()
    {
        RemoveError(this, InvalidDuration);
        Text = null;
        Display.IsVisible = true;
        TextBox.IsVisible = false;
    }
}