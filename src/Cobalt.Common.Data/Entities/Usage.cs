using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

/// <summary>
///     A continuous usage of an <see cref="App" /> during an <see cref="Session" />
/// </summary>
public class Usage : IEntity, IHasDuration
{
    public required Session Session { get; set; }

    [Column(nameof(Start))] public required long StartTicks { get; set; }

    [Column(nameof(End))] public required long EndTicks { get; set; }

    [NotMapped] public DateTime Start => new(StartTicks);

    [NotMapped] public DateTime End => new(EndTicks);

    public long Id { get; set; }

    public TimeSpan Duration => End - Start;
}