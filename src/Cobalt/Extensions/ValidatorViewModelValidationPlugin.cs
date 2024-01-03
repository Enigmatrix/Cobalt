using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Diagnostics.CodeAnalysis;
using System.Linq;
using System.Reactive.Linq;
using Avalonia.Data;
using Avalonia.Data.Core.Plugins;
using DynamicData;
using ReactiveUI.Validation.Abstractions;
using ReactiveUI.Validation.Components.Abstractions;
using ReactiveUI.Validation.States;

namespace Cobalt.Extensions;

/// <summary>
///     Validation Plugin for <see cref="IValidatableViewModel" />, based off
///     <see
///         href="https://github.com/AvaloniaUI/Avalonia/blob/21ad94b98085767674204e7da71dd2b403759c38/src/Avalonia.Base/Data/Core/Plugins/IndeiValidationPlugin.cs#L13">
///         the
///         INDE plugin
///     </see>
/// </summary>
public class ValidatorViewModelValidationPlugin : IDataValidationPlugin
{
    [RequiresUnreferencedCode("DataValidationPlugin might require unreferenced code.")]
    public bool Match(WeakReference<object?> reference, string memberName)
    {
        reference.TryGetTarget(out var target);

        return target is IValidatableViewModel;
    }

    [RequiresUnreferencedCode("DataValidationPlugin might require unreferenced code.")]
    public IPropertyAccessor Start(WeakReference<object?> reference, string propertyName,
        IPropertyAccessor inner)
    {
        return new Validator(reference, propertyName, inner);
    }


    private class Validator
        (WeakReference<object?> reference, string name, IPropertyAccessor inner) : DataValidationBase(inner)
    {
        private const bool ExclusivePropertyName = true;
        private IDisposable? _validationStatesSub;

        private static IValidationState[] InitialValidationStates { get; } = [ValidationState.Valid];

        protected override void SubscribeCore()
        {
            var target = GetReferenceTarget();

            if (target != null)
            {
                // ref: https://github.com/reactiveui/ReactiveUI.Validation/blob/540fafebaeebeecd322bd5685caa1f3c70f9ac6d/src/ReactiveUI.Validation/Extensions/ValidationContextExtensions.cs#L51
                var validationStatesForProperty = target.ValidationContext
                    .Validations
                    .GetWeakCollectionChangeSet<ReadOnlyObservableCollection<IValidationComponent>,
                        IValidationComponent>()
                    .ToCollection()
                    .Select(validations => validations
                        .OfType<IPropertyValidationComponent>()
                        .Where(validation => validation.ContainsPropertyName(name, ExclusivePropertyName))
                        .Select(validation => validation.ValidationStatusChange)
                        .CombineLatest()
                        .StartWith(InitialValidationStates))
                    .Switch();


                _validationStatesSub =
                    validationStatesForProperty.Subscribe(states =>
                        PublishValue(CreateBindingNotification(Value, states)));
            }

            base.SubscribeCore();
        }

        protected override void UnsubscribeCore()
        {
            var target = GetReferenceTarget();

            if (target != null) _validationStatesSub?.Dispose();

            base.UnsubscribeCore();
        }

        protected override void InnerValueChanged(object? value)
        {
            PublishValue(CreateBindingNotification(value));
        }

        private BindingNotification CreateBindingNotification(object? value, IList<IValidationState>? states = null)
        {
            List<string>? errors;
            if (states == null) // exists for InnerValueChanged
            {
                var target = GetReferenceTarget();
                errors = target?.ValidationContext.Validations
                    .OfType<IPropertyValidationComponent>()
                    .Where(validation =>
                        validation.ContainsPropertyName(name, ExclusivePropertyName) && !validation.IsValid)
                    .SelectMany(x => x.Text ?? Enumerable.Empty<string>())
                    .ToList();
            }
            else
            {
                errors = states.Where(state => !state.IsValid).SelectMany(state => state.Text).ToList();
            }

            if (errors?.Count > 0)
                return new BindingNotification(
                    GenerateException(errors),
                    BindingErrorType.DataValidationError,
                    value);

            return new BindingNotification(value);
        }

        private IValidatableViewModel? GetReferenceTarget()
        {
            reference.TryGetTarget(out var target);

            return target as IValidatableViewModel;
        }

        private static Exception GenerateException(IList<string>? errors)
        {
            if (errors!.Count == 1)
                return new DataValidationException(errors[0]);
            return new AggregateException(
                errors.Select(x => new DataValidationException(x)));
        }
    }
}