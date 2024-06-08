using Cobalt.Common.ViewModels.Analysis;

namespace Cobalt.Common.ViewModels;

/// <summary>
///     Trait interface for classes with a <see cref="Image" />
/// </summary>
public interface IHasIcon
{
    Query<byte[]?> Image { get; }
}