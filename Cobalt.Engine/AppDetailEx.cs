using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Util;

namespace Cobalt.Engine
{
    public static class AppDetailEx
    {
        public static App WithDetails(this AppResource res, string appPath)
        {
            var (icon, color) = res.GetAppIconAndColor(appPath);
            return new App
            {
                Icon = Observable.Return(icon),
                Name = res.GetAppName(appPath),
                Path = appPath,
                Color = color
            };
        }
    }
}