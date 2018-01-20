using System;
using System.Diagnostics;
using Microsoft.Deployment.WindowsInstaller;

namespace Cobalt.Setup.CustomActions
{
    public class Util
    {
        public static string GetInstallFolder(Session session)
        {
            try
            {
                return session.CustomActionData["INSTALLFOLDER"];
            }
            catch (Exception)
            {
                return session["INSTALLFOLDER"];
            }
        }

        public static void StopCobalt()
        {
            foreach (var process in Process.GetProcessesByName("Cobalt"))
            {
                process.Kill();
                process.WaitForExit();
            }
        }
    }
}