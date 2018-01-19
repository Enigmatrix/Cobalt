using System;
using System.Diagnostics;
using Microsoft.Deployment.WindowsInstaller;
using Microsoft.Win32.TaskScheduler;

namespace Cobalt.Setup.CustomActions
{
    public class CustomActions
    {
        private static readonly string[] TaskNames = { "Cobalt.Engine", "Cobalt.TaskbarNotifier" };

        [CustomAction]
        public static ActionResult InstallTasks(Session session) => RunOnTaskNames(session, InstallTask);
        [CustomAction]
        public static ActionResult LaunchTasks(Session session) => RunOnTaskNames(session, LaunchTask);
        [CustomAction]
        public static ActionResult DeleteTasks(Session session) => RunOnTaskNames(session, DeleteTask);

        private static ActionResult RunOnTaskNames(Session session, Action<string, string, TaskService> func)
        {

            var installLocation = GetInstallLocation(session);

            using (var ts = new TaskService())
            {
                foreach (var taskName in TaskNames)
                {
                    func(installLocation, taskName, ts);
                }
            }
            return ActionResult.Success;
        }

        private static string GetInstallLocation(Session session)
        {
           return session["INSTALLFOLDER"];
        }

        private static void InstallTask(string installLocation, string prog, TaskService ts)
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

        private static void DeleteTask(string installLocation, string taskName, TaskService ts)
        {
            ts.FindTask(taskName)?.Stop();
            ts.RootFolder.DeleteTask(taskName, exceptionOnNotExists:false);
        }

        private static void LaunchTask(string installLocation, string taskName, TaskService ts)
        {
            ts.FindTask(taskName)?.Run();
        }
    }
}