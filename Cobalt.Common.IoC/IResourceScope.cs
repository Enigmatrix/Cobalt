using System;

namespace Cobalt.Common.IoC
{
    public interface IResourceScope
    {
        void Dispose();
        void Manage(IDisposable dis);
        IResourceScope Subscope();
        T Resolve<T>();
    }
}