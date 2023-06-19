namespace Cobalt.Common.ViewModels.Interactions;

public class InteractionContext<TInput, TOutput>
{
    private TOutput? _output;

    public InteractionContext(TInput input)
    {
        Input = input;
    }

    public TInput Input { get; }

    public void SetOutput(TOutput output)
    {
        _output = output;
    }

    public TOutput? GetOutput()
    {
        return _output;
    }
}