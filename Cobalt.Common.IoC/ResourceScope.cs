using System;
using Autofac;

namespace Cobalt.Common.IoC
{
    //wrapper around ILifetimeScope, making scope interactions more ioc-agnostic
    public class ResourceScope : IDisposable, IResourceScope
    {
        private readonly ILifetimeScope _scope;
        private bool _disposed;

        public ResourceScope(ILifetimeScope scope)
        {
            _scope = scope;
        }

        public void Dispose()
        {
            _disposed = true;
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

        public T Resolve<T>(Type type)
        {
            return (T)_scope.Resolve(type);
        }

        public void Manage(IDisposable dis)
        {
            if (_disposed) return;
            _scope?.Disposer.AddInstanceForDisposal(dis);
        }
    }
}