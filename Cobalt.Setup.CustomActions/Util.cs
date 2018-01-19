using System;
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
    }
}