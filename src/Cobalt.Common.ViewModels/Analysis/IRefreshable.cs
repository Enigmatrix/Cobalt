namespace Cobalt.Common.ViewModels.Analysis;

/// <summary>
///     Represents a refreshable object
/// </summary>
public interface IRefreshable
{
    /// <summary>
    ///     Asynchronously refresh this object
    /// </summary>
    public Task Refresh();
}