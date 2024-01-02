using System;
using System.Reactive.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Data;
using Avalonia.Input;
using Avalonia.Interactivity;
using Cobalt.Common.ViewModels;
using CommunityToolkit.Mvvm.ComponentModel;
using ReactiveUI;
using ReactiveUI.Validation.Abstractions;
using ReactiveUI.Validation.Contexts;
using ReactiveUI.Validation.Extensions;
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

    public static readonly DirectProperty<DurationPicker, DurationPickerViewModel?> ViewModelProperty =
        AvaloniaProperty.RegisterDirect<DurationPicker, DurationPickerViewModel?>(
            nameof(ViewModel),
            o => o.ViewModel,
            (o, v) => o.ViewModel = v);

    private TimeSpan? _duration;
    private IDisposable? _durationBind;
    private DurationPickerViewModel? _viewModel;

    public DurationPicker()
    {
        InitializeComponent();
    }

    public DurationPickerViewModel? ViewModel
    {
        get => _viewModel;
        set => SetAndRaise(ViewModelProperty, ref _viewModel, value);
    }

    public TimeSpan? Duration
    {
        get => _duration;
        set => SetAndRaise(DurationProperty, ref _duration, value);
    }

    protected override void UpdateDataValidation(AvaloniaProperty property, BindingValueType state, Exception? error)
    {
        if (property == DurationProperty)
            DataValidationErrors.SetError(this, error);
    }

    protected override void OnLoaded(RoutedEventArgs e)
    {
        ViewModel = new DurationPickerViewModel();
        _durationBind = ViewModel.WhenAnyValue(self => self.Duration).Subscribe(x => { Duration = x; });

        base.OnLoaded(e);
    }

    protected override void OnUnloaded(RoutedEventArgs e)
    {
        base.OnUnloaded(e);

        ViewModel?.Dispose();
        _durationBind?.Dispose();
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
        if (e.Key == Key.Enter || e.Key == Key.Escape) SwitchToDisplay();
    }

    private void SwitchToDisplay()
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