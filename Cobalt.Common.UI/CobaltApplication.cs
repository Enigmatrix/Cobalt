using System;
using System.Collections.Generic;
using System.Reflection;
using System.Text;
using System.Windows;
using Cobalt.Common.IoC;
using Splat;

namespace Cobalt.Common.UI
{
    public class CobaltApplication : Application
    {
        public CobaltApplication() 
        {
            IoCService.Instance = new IoCService(x =>
                x.RegisterForReactiveUI(Assembly.GetEntryAssembly()));
            Locator.CurrentMutable = new AutofacDependencyResolver(IoCService.Instance.Container);
        }
    }
}
