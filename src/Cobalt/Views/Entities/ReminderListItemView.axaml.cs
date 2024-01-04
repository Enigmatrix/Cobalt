using System;
using System.Windows.Input;
using Avalonia;
using Cobalt.Common.ViewModels.Entities;
using FluentAvalonia.UI.Controls;
using ReactiveUI;

namespace Cobalt.Views.Entities;

public partial class ReminderListItemView : SettingsExpanderItem, IViewFor<EditableReminderViewModel>
{
    public static readonly StyledProperty<EditableReminderViewModel?> ViewModelProperty = AvaloniaProperty
        .Register<ReminderListItemView, EditableReminderViewModel?>(nameof(ViewModel));

    public static readonly StyledProperty<TimeSpan?> UsageLimitProperty = AvaloniaProperty
        .Register<ReminderListItemView, TimeSpan?>(nameof(UsageLimit));

    public static readonly StyledProperty<ICommand?> DeleteCommandProperty = AvaloniaProperty
        .Register<ReminderListItemView, ICommand?>(nameof(DeleteCommand));

    public static readonly StyledProperty<object?> DeleteCommandParameterProperty = AvaloniaProperty
        .Register<ReminderListItemView, object?>(nameof(DeleteCommandParameter));

    public ReminderListItemView()
    {
        InitializeComponent();

        this.WhenActivated(disposables => { });
        this.GetObservable(ViewModelProperty).Subscribe(OnViewModelChanged);
    }

    protected override Type StyleKeyOverride { get; } = typeof(SettingsExpanderItem);

    public TimeSpan? UsageLimit
    {
        get => GetValue(UsageLimitProperty);
        set => SetValue(UsageLimitProperty, value);
    }

    public ICommand? DeleteCommand
    {
        get => GetValue(DeleteCommandProperty);
        set => SetValue(DeleteCommandProperty, value);
    }

    public object? DeleteCommandParameter
    {
        get => GetValue(DeleteCommandParameterProperty);
        set => SetValue(DeleteCommandParameterProperty, value);
    }

    /// <summary>
    ///     The ViewModel.
    /// </summary>
    public EditableReminderViewModel? ViewModel
    {
        get => GetValue(ViewModelProperty);
        set => SetValue(ViewModelProperty, value);
    }

    object? IViewFor.ViewModel
    {
        get => ViewModel;
        set => ViewModel = (EditableReminderViewModel?)value;
    }

    protected override void OnDataContextChanged(EventArgs e)
    {
        base.OnDataContextChanged(e);
        ViewModel = DataContext as EditableReminderViewModel;
    }

    private void OnViewModelChanged(object? value)
    {
        if (value == null)
            ClearValue(DataContextProperty);
        else if (DataContext != value) DataContext = value;
    }
}