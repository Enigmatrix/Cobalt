using System;
using Autofac;

namespace Cobalt.Common.IoC
{
    //wrapper around ILifetimeScope, making scope interactions more ioc-agnostic
    public class ResourceScope : IDisposable, IResourceScope
    {
        private readonly ILifetimeScope _scope;

        public ResourceScope(ILifetimeScope scope)
        {
            _scope = scope;
        }

        public void Dispose()
        {
            _scope.Dispose();
        }

        public IResourceScope Subscope()
        {
            return new ResourceScope(_scope.BeginLifetimeScope());
        }

        public T Resolve<T>()
        {
            return _scope.Resolve<T>();
        }

        public void Manage(IDisposable dis)
        {
            _scope.Disposer.AddInstanceForDisposal(dis);
        }
    }
}