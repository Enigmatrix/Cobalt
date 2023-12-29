using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Data;

namespace Cobalt.Controls;

/// <summary>
///     Exact same as a regular <see cref="ComboBox" />, but both SelectedItem and SelectedValue trigger the Data
///     Validation
/// </summary>
public class ExtendedComboBox : ComboBox
{
    static ExtendedComboBox()
    {
        SelectedValueProperty.OverrideMetadata<ExtendedComboBox>(
            new StyledPropertyMetadata<object?>(defaultBindingMode: BindingMode.TwoWay, enableDataValidation: true));
    }

    protected override Type StyleKeyOverride { get; } = typeof(ComboBox);

    protected override void UpdateDataValidation(AvaloniaProperty property, BindingValueType state, Exception? error)
    {
        if (property == SelectedItemProperty || property == SelectedValueProperty)
            DataValidationErrors.SetError(this, error);
    }
}