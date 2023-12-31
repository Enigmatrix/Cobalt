using System;
using System.Reactive.Linq;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Cobalt.Common.ViewModels;
using CommunityToolkit.Mvvm.ComponentModel;
using ReactiveUI;
using ReactiveUI.Validation.Abstractions;
using ReactiveUI.Validation.Contexts;
using ReactiveUI.Validation.Extensions;
using TimeSpanParserUtil;

namespace Cobalt.Controls;

public partial class DurationPicker : UserControl
{
    public DurationPicker()
    {
        InitializeComponent();
        // TODO dispose!
        DataContext = new DurationPickerViewModel();
    }

    private void Display_OnClick(object? sender, RoutedEventArgs e)
    {
        Display.IsVisible = false;
        TextBox.IsVisible = true;
        TextBox.Focus();
    }

    private void TextBox_OnLostFocus(object? sender, RoutedEventArgs e)
    {
        Display.IsVisible = true;
        TextBox.IsVisible = false;
    }
}

public partial class DurationPickerViewModel : ReactiveObservableObject, IValidatableViewModel, IDisposable
{
    private readonly IDisposable _propertyBinding;
    [ObservableProperty] private TimeSpan? _duration;
    [ObservableProperty] private string? _text;

    public DurationPickerViewModel()
    {
        this.ValidationRule(self => self.Text, text => TimeSpanParser.TryParse(text, out _), "Invalid duration");
        _propertyBinding = this.WhenAnyValue(self => self.Text)
            .Where(text => TimeSpanParser.TryParse(text, out _))
            .Select(text => TimeSpanParser.Parse(text!))
            .BindTo(this, self => self.Duration);
    }

    public void Dispose()
    {
        _propertyBinding.Dispose();
        ValidationContext.Dispose();
    }

    public ValidationContext ValidationContext { get; } = new();
}