namespace Cobalt.Common.ViewModels.Entities;

public record AlertDurationViewModel(
    AlertViewModel Alert,
    TimeSpan Duration) : IHaveDuration;