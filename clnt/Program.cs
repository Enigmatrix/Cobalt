using System;
using Cobalt.Common.Transmission;
using Cobalt.Common.Transmission.Messages;

namespace clnt
{
    public class Program
    {
        public static void Main(string[] args)
        {
            var client = new TransmissionClient();
            var c = 0;
            client.MessageReceived += (s, e) =>
            {
                var asg = e.Message as AppSwitchMessage;
                //Console.WriteLine($"{asg.PreviousAppUsage.App.Path} ran for {asg.PreviousAppUsage.Duration}");
                Console.WriteLine($"[{c++}]: {asg.NewApp.Path}\n");
            };
        }
    }
}