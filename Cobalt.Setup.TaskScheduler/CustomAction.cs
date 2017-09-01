using System;
using System.Diagnostics;
using System.IO;
using Microsoft.Deployment.WindowsInstaller;
using Microsoft.Win32.TaskScheduler;

namespace Cobalt.Setup.TaskScheduler
{
    public class CustomActions
    {
        [CustomAction]
        public static ActionResult InstallCobaltEngineToTaskScheduler(Session session)
        {
            Log("start");

            try
            {
                var installLocation = session.CustomActionData["INSTALLFOLDER"];

                using (var ts = new TaskService())
                {
                    Setup(installLocation, "Cobalt.Engine", ts);
                    Setup(installLocation, "Cobalt.TaskbarNotifier", ts);
                }
            }
            catch (Exception e)
            {
                Log($"Register Failure: {e}\n Message:{e.Message}\n \nStacktrace:{e.StackTrace}\n");
                return ActionResult.Failure;
            }

            return ActionResult.Success;
        }

        private static void Setup(string installLocation, string prog, TaskService ts)
        {
            var task = ts.NewTask();

            task.Triggers.Add(new LogonTrigger());
            task.Actions.Add(
                new ExecAction($"{installLocation}{prog}.exe", "", installLocation));

            task.Principal.RunLevel = TaskRunLevel.Highest;
            task.Principal.LogonType = TaskLogonType.InteractiveToken;
            //users group
            task.Principal.GroupId = "S-1-5-32-545";

            task.Settings.MultipleInstances = TaskInstancesPolicy.IgnoreNew;
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

            ts.RootFolder.RegisterTaskDefinition(prog, task);
        }

        private static void Log(string msg)
        {
            File.AppendAllText(@"C:\Users\enigm\wixlog.txt", msg + "\r\n");
        }
    }
}