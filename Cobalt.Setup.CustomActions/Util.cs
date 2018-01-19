using Microsoft.Deployment.WindowsInstaller;

namespace Cobalt.Setup.CustomActions
{
    public class Util
    {
        public static string GetInstallFolder(Session session)
        {
            return session.CustomActionData["INSTALLFOLDER"];
        }
    }
}