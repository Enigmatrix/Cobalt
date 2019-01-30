using System.Linq;
using System.Reflection;
using Autofac;
using ReactiveUI;

namespace Cobalt.Common.UI
{

    /// <summary>
    /// Registration extentions to register ReactiveUI-related components with Autofac.
    /// </summary>
    public static class RegistrationExtensions
    {
        /// <summary>
        /// Register all types that implement generic interface IViewFor{T}
        /// </summary>
        /// <param name="builder">Container builder</param>
        /// <param name="assemblies">Assemblies to be scanned</param>
        public static void RegisterViews(this ContainerBuilder builder, params Assembly[] assemblies) =>
            builder.RegisterAssemblyTypes(assemblies)
                .Where(t => t.GetTypeInfo()
                    .ImplementedInterfaces.Any(
                        i => i.IsGenericType && i.GetGenericTypeDefinition() == typeof(IViewFor<>)))
                .AsImplementedInterfaces();


        /// <summary>
        /// Register all types that have "ViewModel" in their name
        /// IScreen implementation is ignored since it must be registered as a singleton
        /// <seealso cref="RegisterScreen"/>
        /// </summary>
        /// <param name="builder">Container builder</param>
        /// <param name="assemblies">Assemblies to be scanned</param>
        public static void RegisterViewModels(this ContainerBuilder builder, params Assembly[] assemblies) =>
            builder.RegisterAssemblyTypes(assemblies)
                .Where(t => t.GetTypeInfo().Name.Contains("ViewModel") && t.GetTypeInfo().ImplementedInterfaces.All(x => x != typeof(IScreen)))
                .AsSelf()
                .AsImplementedInterfaces();

        /// <summary>
        /// Register IScreen implementations as singletons.
        /// Actually, you should only have one of those. We allow sending many assemblies to make <see cref="RegisterForReactiveUI"/> to work.
        /// </summary>
        /// <param name="builder">Container builder</param>
        /// <param name="assemblies">Assemblies to be scanned</param>
        public static void RegisterScreen(this ContainerBuilder builder, params Assembly[] assemblies) =>
            builder.RegisterAssemblyTypes(assemblies)
                .Where(t => t.GetTypeInfo().ImplementedInterfaces.Any(x => x == typeof(IScreen)))
                .AsSelf()
                .As<IScreen>()
                .SingleInstance();

        /// <summary>
        /// Performs assembly scanning for views, view models and the screen.
        /// View models are registered by convention and must have the "ViewName" in their class names.
        /// You can call this method only to register all required components.
        /// </summary>
        /// <param name="builder">Container builder</param>
        /// <param name="assemblies">Assemblies to be scanned</param>
        public static void RegisterForReactiveUI(this ContainerBuilder builder, params Assembly[] assemblies)
        {
            builder.RegisterViews(assemblies);
            builder.RegisterViewModels(assemblies);
            builder.RegisterScreen(assemblies);
        }
    }}
