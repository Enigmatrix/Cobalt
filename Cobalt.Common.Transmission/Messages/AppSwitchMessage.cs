using Cobalt.Common.Data;

namespace Cobalt.Common.Transmission.Messages
{
    public class AppSwitchMessage : MessageBase
    {
        public AppSwitchMessage(AppUsage prev, App newApp)
        {
            PreviousAppUsage = prev;
            NewApp = newApp;
        }

        public AppSwitchMessage()
        {
        }

        public AppUsage PreviousAppUsage { get; set; }
        public App NewApp { get; set; }
    }
}