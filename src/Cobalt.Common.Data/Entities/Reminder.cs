namespace Cobalt.Common.Data.Entities;

public class Reminder
{
    public long Id { get; set; }
    public Alert Alert { get; set; }
    public double Threshold { get; set; }
    public string Message { get; set; }
}
