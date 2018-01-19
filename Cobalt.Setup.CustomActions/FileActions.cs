using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Microsoft.Deployment.WindowsInstaller;

namespace Cobalt.Setup.CustomActions
{
    public class FileActions
    {
        [CustomAction]
        public static ActionResult DeleteRemnants(Session session)
        {
            var installFolder = Util.GetInstallFolder(session);
            Directory.Delete(installFolder, true);
            return ActionResult.Success;
        }


    }
}
