using System.IO;
using Microsoft.Deployment.WindowsInstaller;

namespace Cobalt.Setup.CustomActions
{
    public class FileActions
    {
        [CustomAction]
        public static ActionResult DeleteRemnants(Session session)
        {
            var installFolder = Util.GetInstallFolder(session);
            Util.StopCobalt();
            Directory.Delete(installFolder, true);
            return ActionResult.Success;
        }
    }
}