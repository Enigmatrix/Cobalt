using System;

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