﻿using System;

namespace Cobalt.Common.IoC
{
    public static class ResourceScopeEx
    {
        public static void ManageUsing(this IDisposable dis, IResourceScope scope)
        {
            scope.Manage(dis);
        }

        public static T ManagedBy<T>(this T dis, IResourceScope scope) where T : IDisposable
        {
            dis.ManageUsing(scope);
            return dis;
        }
    }
}