namespace Cobalt.Common.Data.Entities;

/// <summary>
///     Continuous session of <see cref="Cobalt.Common.Data.Entities.App" /> usage.
/// </summary>
public class Session : IEntity, IHasName
{
    public required string Title { get; set; }
    public required App App { get; set; }
    public List<Usage> Usages { get; } = new();
    public long Id { get; set; }

    public string Name => Title;
}