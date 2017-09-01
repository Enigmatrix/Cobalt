using System;
using System.Diagnostics;
using Microsoft.Deployment.WindowsInstaller;
using Microsoft.Win32.TaskScheduler;

namespace Cobalt.Setup.TaskScheduler
{
    public class CustomActions
    {
        [CustomAction]
        public static ActionResult InstallCobaltEngineToTaskScheduler(Session session)
        {
            //File.AppendAllText(@"C:\Users\enigm\wixlog.txt", "Start\n");
            var installLocation = session.CustomActionData["INSTALLFOLDER"];
            using (var ts = new TaskService())
            {
                var task = ts.NewTask();

                task.Triggers.Add(new LogonTrigger());
                task.Actions.Add(
                    new ExecAction($"{installLocation}Cobalt.Engine.exe", "", installLocation));

                task.Principal.LogonType = TaskLogonType.InteractiveToken;
                task.Principal.RunLevel = TaskRunLevel.Highest;
                task.Principal.UserId = $@"{Environment.UserDomainName}\{Environment.UserName}";

                task.Settings.MultipleInstances =TaskInstancesPolicy.IgnoreNew;
                task.Settings.DisallowStartIfOnBatteries = false;
                task.Settings.StopIfGoingOnBatteries = false;
                task.Settings.AllowHardTerminate = false;
                task.Settings.StartWhenAvailable = true;
                task.Settings.RunOnlyIfNetworkAvailable = false;
                task.Settings.IdleSettings.StopOnIdleEnd = false;
                task.Settings.IdleSettings.RestartOnIdle = false;
                task.Settings.AllowDemandStart = true;
                task.Settings.Hidden = false;
                task.Settings.Enabled = true;
                task.Settings.RunOnlyIfIdle = false;
                task.Settings.WakeToRun = false;
                task.Settings.Priority = ProcessPriorityClass.Normal;


                //File.AppendAllText(@"C:\Users\enigm\wixlog.txt", "Registering...\n");
                try
                {
                    ts.RootFolder.RegisterTaskDefinition("Cobalt.Engine", task);
                }
                catch (Exception)
                {
                    //File.AppendAllText(@"C:\Users\enigm\wixlog.txt", $"Register Failure: {e}\n Message:{e.Message}\n \nStacktrace:{e.StackTrace}\n");
                    return ActionResult.Failure;
                }
            }
            //File.AppendAllText(@"C:\Users\enigm\wixlog.txt", "Success\n");

            return ActionResult.Success;
        }
    }
}
