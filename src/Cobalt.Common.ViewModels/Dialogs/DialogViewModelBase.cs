using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels.Dialogs;

public abstract class DialogViewModelBase<TInput, TOutput> : ObservableValidator
{
    protected TInput Input = default!;
    protected TOutput? Output = default!;

    public virtual void SetInput(TInput input)
    {
        Input = input;
    }

    public virtual TOutput GetOutput()
    {
        return Output ?? throw new InvalidOperationException("Output not initialized");
    }
}