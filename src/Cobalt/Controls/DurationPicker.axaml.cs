using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Metadata;
using Avalonia.Controls.Primitives;
using Avalonia.Data;
using Avalonia.Interactivity;

namespace Cobalt.Controls;

[TemplatePart("PART_DurationTextBox", typeof(TextBox))]
[TemplatePart("PART_DurationDisplay", typeof(Button))]
public class DurationPicker : TemplatedControl
{
    public static readonly StyledProperty<TimeSpan?> DurationProperty =
        AvaloniaProperty.Register<ComboBox, TimeSpan?>(nameof(Duration), enableDataValidation: true,
            defaultBindingMode: BindingMode.TwoWay);

    private Button? _display;

    private TextBox? _textBox;

    public TimeSpan? Duration
    {
        get => GetValue(DurationProperty);
        set => SetValue(DurationProperty, value);
    }

    protected override void OnApplyTemplate(TemplateAppliedEventArgs e)
    {
        if (_textBox != null) _textBox.LostFocus -= TextBoxLostFocus;
        if (_display != null) _display.Click -= DisplayClicked;

        _textBox = e.NameScope.Get<TextBox>("PART_DurationTextBox");
        _display = e.NameScope.Get<Button>("PART_DurationDisplay");

        _textBox.LostFocus += TextBoxLostFocus;
        _display.Click += DisplayClicked;
    }

    private void TextBoxLostFocus(object? sender, RoutedEventArgs e)
    {
        _textBox!.IsVisible = false;
        _display!.IsVisible = true;
    }

    private void DisplayClicked(object? sender, RoutedEventArgs e)
    {
        _textBox!.IsVisible = true;
        _display!.IsVisible = false;
        _textBox.Focus();
    }
}