namespace Cobalt.Common.ViewModels.Interactions;

public class Interaction<TInput, TOutput>
{
    private readonly List<Func<InteractionContext<TInput, TOutput>, ValueTask>> _handlers;

    public Interaction()
    {
        _handlers = new List<Func<InteractionContext<TInput, TOutput>, ValueTask>>();
    }

    public Action RegisterHandler(Func<InteractionContext<TInput, TOutput>, ValueTask> handler)
    {
        _handlers.Add(handler);
        return () => _handlers.Remove(handler);
    }

    public async ValueTask<TOutput?> Handle(TInput input)
    {
        var ctx = new InteractionContext<TInput, TOutput>(input);
        foreach (var handler in _handlers.AsEnumerable().Reverse())
        {
            await handler(ctx);
            var output = ctx.GetOutput();
            if (output != null) return output;
        }

        return default;
    }
}