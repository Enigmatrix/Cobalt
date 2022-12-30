namespace Cobalt.Common.Data.Models;

public class App
{
    public long Id { get; set; } = default!;
    // public bool Initialized { get; set; } // TODO not be public
    public string Name { get; set; } = default!;
    public string Description { get; set; } = default!; 
    public string Company { get; set; } = default!;
    public string Color { get; set; } = default!;
    // public int AppIdentityTag { get; set; }
    // public string AppIdentityText0 { get; set; }

    // TODO icon
}