using Cobalt.Common.Data.Entities;

namespace Cobalt.Common.Data;

public interface IHasEntity
{
    public IEntity Inner { get; }
}