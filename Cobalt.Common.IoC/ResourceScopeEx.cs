using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Cobalt.Common.IoC
{
    public static class ResourceScopeEx
    {
        public static void ManageUsing(this IDisposable dis, IResourceScope scope)
        {
            scope.Manage(dis);
        }
    }
}
